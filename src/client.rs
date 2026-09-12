use reqwest::header;
use serde::Serialize;
use serde::de::DeserializeOwned;

use std::fmt;
use std::future::Future;
use std::net::Ipv4Addr;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::Modes;
use crate::constants::{Area, Mode};
use crate::errors::{Error, Result};
use crate::resources::{ApiResponse, devices, panel, response};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Bounds the TCP and TLS handshake, which `REQUEST_TIMEOUT` alone would let
/// consume the whole budget.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The panel serves full HTML pages on failure, which are large and of little
/// diagnostic value beyond their first few lines.
const MAX_ERROR_BODY_BYTES: usize = 512;

// The panel can discard idle sessions without notifying the client.
const TOKEN_TTL: Duration = Duration::from_secs(60);

/// The path the panel serves once a session has expired.
const LOGIN_PATH: &str = "/action/login";

/// An asynchronous client for the LUPUSEC HTTP API.
///
/// The client owns a connection pool and caches the panel's session token, so a
/// single instance should be shared rather than created per request. Every
/// method takes `&self`, so an `Arc<Client>` can be used from multiple tasks.
///
/// Every request is retried once if the panel reports a session timeout or an
/// unauthorized error, both of which it returns transiently; the cached token is
/// dropped before the retry.
pub struct Client {
    http: reqwest::Client,
    username: String,
    password: String,
    base_url: reqwest::Url,
    token: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    value: String,
    expires_at: Instant,
}

/// Redacts the credentials, so a `Client` can sit in a `Debug` application
/// state without leaking them into logs.
impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `try_lock`, because formatting must not block or panic just because
        // another task holds the token.
        let token = match self.token.try_lock() {
            Ok(slot) if slot.is_some() => "<redacted>",
            Ok(_) => "<unset>",
            Err(_) => "<locked>",
        };

        f.debug_struct("Client")
            .field("base_url", &self.base_url.as_str())
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("token", &token)
            .finish()
    }
}

impl Client {
    /// Construct a client for the panel at `ip_address`.
    ///
    /// The client accepts self-signed TLS certificates because LUPUSEC panels
    /// ship with self-signed certs by default. Certificates are therefore not
    /// authenticated at all: only use this on a network you trust.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be built.
    pub fn new(username: &str, password: &str, ip_address: Ipv4Addr) -> Result<Client> {
        let base_url = format!("https://{ip_address}/action/")
            .parse()
            .expect("a well-formed IPv4 address yields a valid base URL");

        Client::with_base_url(username, password, base_url)
    }

    fn with_base_url(username: &str, password: &str, base_url: reqwest::Url) -> Result<Client> {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            // The panel is a fixed address on the local network, so a proxy is
            // never wanted. Without this, a `HTTPS_PROXY` in the environment
            // would route the credentials through a third party — and because
            // certificates are not validated, silently so.
            .no_proxy()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()?;

        Ok(Client {
            http,
            username: username.into(),
            password: password.into(),
            base_url,
            token: Mutex::new(None),
        })
    }

    /// Get the status of the alarm panel.
    ///
    /// # Errors
    ///
    /// Returns an error if the panel cannot be reached, rejects the
    /// credentials, or sends a response that cannot be understood.
    pub async fn get_status(&self) -> Result<Modes> {
        self.get::<panel::Condition>("panelCondGet").await
    }

    /// Change the mode of the given area.
    ///
    /// # Errors
    ///
    /// Returns an error if the panel cannot be reached, rejects the
    /// credentials, or refuses the mode change.
    pub async fn change_mode(&self, area: Area, mode: Mode) -> Result {
        let payload = &[("mode", mode as u8), ("area", area as u8)];

        self.post::<response::Response, _>("panelCondPost", payload)
            .await?;

        Ok(())
    }

    /// List all devices managed by the alarm panel.
    ///
    /// # Errors
    ///
    /// Returns an error if the panel cannot be reached, rejects the
    /// credentials, or reports a device type this crate does not know.
    pub async fn list_devices(&self) -> Result<Vec<devices::Device>> {
        self.get::<devices::List>("deviceListGet").await
    }

    async fn get<D>(&self, action: &str) -> Result<D::Output>
    where
        D: ApiResponse + DeserializeOwned,
    {
        self.retrying(|| async move { parse::<D>(self.send_get(action).await?).await })
            .await
    }

    async fn post<D, F>(&self, action: &str, form: &F) -> Result<D::Output>
    where
        D: ApiResponse + DeserializeOwned,
        F: Serialize + ?Sized,
    {
        self.retrying(|| async move {
            let token = self.token().await?;
            parse::<D>(self.send_post(action, form, &token).await?).await
        })
        .await
    }

    /// Run an operation, retrying it once if the panel's session state is stale.
    ///
    /// The panel returns 401 transiently when its session state is confused, so
    /// an unauthorized response is retried like an expired one. The cost is that
    /// genuinely wrong credentials are tried twice.
    ///
    /// Either way the whole session is suspect, so the cached token is dropped
    /// before retrying — including for GETs, which do not send it themselves but
    /// would otherwise leave a dead token behind for the next POST.
    ///
    /// The retried unit is the entire operation, token fetch included, so a
    /// session that expires between fetching the token and using it recovers.
    ///
    /// `op` is an `Fn` returning a named future rather than an `AsyncFn`:
    /// `AsyncFn` carries a higher-ranked lifetime, which makes the resulting
    /// future impossible to prove `Send` and breaks callers that need one — an
    /// axum handler, for instance.
    async fn retrying<T, F>(&self, op: impl Fn() -> F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match op().await {
            Err(Error::SessionTimeout | Error::Unauthorized) => {
                *self.token_slot() = None;
                op().await
            }
            other => other,
        }
    }

    fn url(&self, path: &str) -> reqwest::Url {
        self.base_url
            .join(path)
            .expect("action path should be a valid relative URL segment")
    }

    async fn send_get(&self, action: &str) -> Result<reqwest::Response> {
        Ok(self
            .http
            .get(self.url(action))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?)
    }

    async fn send_post<T: Serialize + ?Sized>(
        &self,
        action: &str,
        form: &T,
        token: &str,
    ) -> Result<reqwest::Response> {
        let mut token = header::HeaderValue::from_str(token)?;
        token.set_sensitive(true);

        Ok(self
            .http
            .post(self.url(action))
            .form(form)
            .basic_auth(&self.username, Some(&self.password))
            .header("x-token", token)
            .send()
            .await?)
    }

    fn token_slot(&self) -> MutexGuard<'_, Option<CachedToken>> {
        self.token.lock().expect("token lock poisoned")
    }

    fn cached_token(&self) -> Option<String> {
        let slot = self.token_slot();
        let cached = slot.as_ref()?;

        (Instant::now() < cached.expires_at).then(|| cached.value.clone())
    }

    /// Tasks that miss the cache concurrently will each fetch a token; the last
    /// one wins. Not worth serializing for a handful of requests.
    async fn token(&self) -> Result<String> {
        if let Some(token) = self.cached_token() {
            return Ok(token);
        }

        // Not routed through `retrying`: the caller already retries this fetch,
        // and nesting the two would multiply the requests one call can make.
        let token = parse::<response::Response>(self.send_get("tokenGet").await?).await?;
        *self.token_slot() = Some(CachedToken {
            value: token.clone(),
            expires_at: Instant::now() + TOKEN_TTL,
        });

        Ok(token)
    }
}

async fn parse<D>(res: reqwest::Response) -> Result<D::Output>
where
    D: ApiResponse + DeserializeOwned,
{
    // An expired session lands on the login page. reqwest has already followed
    // the redirect, so the final URL says so directly; `parse_body` covers the
    // panels that serve the page without redirecting.
    if res.url().path().ends_with(LOGIN_PATH) {
        return Err(Error::SessionTimeout);
    }

    let status = res.status();
    let body = res.text().await?;

    parse_body::<D>(status, &body)?.into_result()
}

fn parse_body<D: DeserializeOwned>(status: reqwest::StatusCode, body: &str) -> Result<D> {
    if !status.is_success() {
        return Err(match status {
            reqwest::StatusCode::UNAUTHORIZED => Error::Unauthorized,
            status => Error::UnexpectedResponse {
                status,
                body: truncate(body),
            },
        });
    }

    match parse_json(body) {
        // Report the login page as a timeout so the caller retries with a fresh
        // session, rather than as a confusing serde error.
        Err(_) if body.contains(LOGIN_PATH) => Err(Error::SessionTimeout),
        Err(e) => Err(e.into()),
        Ok(model) => Ok(model),
    }
}

/// Deserialize a panel response, working around the raw tabs it emits.
///
/// `serde_json` accepts tabs between tokens but rejects them inside string
/// values, so a body that fails to parse is retried with the tabs removed. A tab
/// inside e.g. a device name is therefore dropped rather than preserved.
fn parse_json<D: DeserializeOwned>(body: &str) -> serde_json::Result<D> {
    serde_json::from_str(body).or_else(|e| {
        if body.contains('\t') {
            serde_json::from_str(&body.replace('\t', ""))
        } else {
            Err(e)
        }
    })
}

fn truncate(body: &str) -> String {
    if body.len() <= MAX_ERROR_BODY_BYTES {
        return body.to_owned();
    }

    let mut end = MAX_ERROR_BODY_BYTES;
    while !body.is_char_boundary(end) {
        end -= 1;
    }

    format!("{}… ({} bytes total)", &body[..end], body.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(server: &MockServer) -> Client {
        let base_url = format!("{}/action/", server.uri()).parse().unwrap();
        Client::with_base_url("user", "pass", base_url).unwrap()
    }

    /// Consumers put the client in shared state and call it from handlers that
    /// require `Send` futures, so guard both properties at compile time.
    #[test]
    fn client_and_its_futures_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        fn assert_send<T: Send>(_: T) {}

        assert_send_sync::<Client>();
        assert_send_sync::<Error>();

        let client = Client::new("user", "pass", "192.168.1.1".parse().unwrap()).unwrap();
        assert_send(client.get_status());
        assert_send(client.list_devices());
        assert_send(client.change_mode(Area::Area1, Mode::Disarmed));
    }

    #[test]
    fn debug_redacts_the_credentials() {
        let client = Client::new("user", "hunter2", "192.168.1.1".parse().unwrap()).unwrap();
        *client.token_slot() = Some(CachedToken {
            value: "tok123".into(),
            expires_at: Instant::now() + TOKEN_TTL,
        });

        let debug = format!("{client:?}");
        assert!(!debug.contains("hunter2"), "{debug}");
        assert!(!debug.contains("tok123"), "{debug}");
        assert!(debug.contains("192.168.1.1"), "{debug}");
    }

    #[test]
    fn url_construction() {
        let client = Client::new("user", "pass", "192.168.1.1".parse().unwrap()).unwrap();
        let url = client.url("panelCondGet");
        assert_eq!(url.as_str(), "https://192.168.1.1/action/panelCondGet");
    }

    #[test]
    fn parse_body_valid_json() {
        let body = r#"{"result": 1, "message": "token123"}"#;
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, body);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_body_unauthorized() {
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::UNAUTHORIZED, "");
        assert!(matches!(result.unwrap_err(), Error::Unauthorized));
    }

    #[test]
    fn parse_body_unexpected_response() {
        let result: Result<response::Response> =
            parse_body(reqwest::StatusCode::INTERNAL_SERVER_ERROR, "oops");
        assert!(matches!(
            result.unwrap_err(),
            Error::UnexpectedResponse { .. }
        ));
    }

    fn unexpected_response_body(body: &str) -> String {
        let result: Result<response::Response> =
            parse_body(reqwest::StatusCode::INTERNAL_SERVER_ERROR, body);

        let Err(Error::UnexpectedResponse { body, .. }) = result else {
            panic!("expected an unexpected-response error");
        };
        body
    }

    #[test]
    fn parse_body_truncates_long_error_bodies() {
        let sent = "x".repeat(MAX_ERROR_BODY_BYTES * 2);
        let retained = unexpected_response_body(&sent);

        assert!(retained.starts_with(&"x".repeat(MAX_ERROR_BODY_BYTES)));
        assert!(retained.contains(&format!("{} bytes total", sent.len())));
    }

    /// The bound is in bytes, so it has to be walked back to a char boundary
    /// rather than slicing through a multi-byte character.
    #[test]
    fn parse_body_truncates_on_a_char_boundary() {
        let sent = "ä".repeat(MAX_ERROR_BODY_BYTES);
        let retained = unexpected_response_body(&sent);

        let kept = retained.split('…').next().unwrap();
        assert!(kept.len() <= MAX_ERROR_BODY_BYTES);
        assert!(kept.chars().all(|c| c == 'ä'));
    }

    #[test]
    fn parse_body_session_timeout() {
        let body = "<html>/action/login</html>";
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, body);
        assert!(matches!(result.unwrap_err(), Error::SessionTimeout));
    }

    #[test]
    fn parse_body_invalid_json() {
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, "not json");
        assert!(matches!(result.unwrap_err(), Error::Deserialize(_)));
    }

    #[test]
    fn parse_body_accepts_tabs_between_tokens() {
        let body = "{\t\"result\":\t1,\t\"message\":\t\"ok\"\t}";
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, body);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_body_recovers_from_tabs_inside_strings() {
        let body = "{\"result\":1,\"message\":\"Hall\tDoor\"}";
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, body);
        assert_eq!(result.unwrap().into_result().unwrap(), "HallDoor");
    }

    fn login_page() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_string("<html>/action/login</html>")
    }

    fn panel_status() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "forms": {
                "pcondform1": { "mode": 0 },
                "pcondform2": { "mode": 1 }
            }
        }))
    }

    fn ok_message(message: &str) -> ResponseTemplate {
        ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({"result": 1, "message": message}))
    }

    /// Mounts `first` for the initial GET and a valid status for everything
    /// after, then asserts the retry recovered. Returns the client so callers
    /// can inspect what the retry left behind.
    async fn assert_get_retries(server: &MockServer, first: ResponseTemplate) -> Client {
        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(first)
            .up_to_n_times(1)
            .expect(1)
            .mount(server)
            .await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(panel_status())
            .expect(1)
            .mount(server)
            .await;

        let client = client(server);
        let modes = client.get_status().await.unwrap();
        assert_eq!(modes.area1, Mode::Disarmed);
        assert_eq!(modes.area2, Mode::Armed);

        client
    }

    /// As above for POST. `tokens` is how many times `tokenGet` is expected —
    /// once for the initial attempt, once after the failure drops the token.
    async fn assert_post_retries(server: &MockServer, first: ResponseTemplate, tokens: u64) {
        Mock::given(method("GET"))
            .and(path("/action/tokenGet"))
            .respond_with(ok_message("tok123"))
            .expect(tokens)
            .mount(server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .respond_with(first)
            .up_to_n_times(1)
            .expect(1)
            .mount(server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .respond_with(ok_message("ok"))
            .expect(1)
            .mount(server)
            .await;

        client(server)
            .change_mode(Area::Area1, Mode::Disarmed)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_retries_on_session_timeout() {
        let server = MockServer::start().await;
        assert_get_retries(&server, login_page()).await;
    }

    #[tokio::test]
    async fn get_retries_on_unauthorized() {
        let server = MockServer::start().await;
        assert_get_retries(&server, ResponseTemplate::new(401)).await;
    }

    #[tokio::test]
    async fn post_retries_on_session_timeout() {
        let server = MockServer::start().await;
        assert_post_retries(&server, login_page(), 2).await;
    }

    #[tokio::test]
    async fn post_retries_on_unauthorized() {
        let server = MockServer::start().await;
        assert_post_retries(&server, ResponseTemplate::new(401), 2).await;
    }

    #[tokio::test]
    async fn a_token_is_reused_within_its_ttl() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/action/tokenGet"))
            .respond_with(ok_message("tok123"))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .and(header("x-token", "tok123"))
            .respond_with(ok_message("ok"))
            .expect(2)
            .mount(&server)
            .await;

        let client = client(&server);
        client
            .change_mode(Area::Area1, Mode::Disarmed)
            .await
            .unwrap();
        let expires_at = client.token_slot().as_ref().unwrap().expires_at;

        client.change_mode(Area::Area1, Mode::Armed).await.unwrap();
        assert_eq!(client.token_slot().as_ref().unwrap().expires_at, expires_at);
    }

    #[tokio::test]
    async fn an_expired_token_is_refetched() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/action/tokenGet"))
            .respond_with(ok_message("fresh"))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .and(header("x-token", "fresh"))
            .respond_with(ok_message("ok"))
            .expect(1)
            .mount(&server)
            .await;

        let client = client(&server);
        *client.token_slot() = Some(CachedToken {
            value: "stale".into(),
            expires_at: Instant::now(),
        });

        client
            .change_mode(Area::Area1, Mode::Disarmed)
            .await
            .unwrap();
    }

    /// A GET does not send the token, but it must still drop it — otherwise the
    /// next POST would present one the panel has already forgotten.
    #[tokio::test]
    async fn get_timeout_drops_the_cached_token() {
        let server = MockServer::start().await;

        let client = assert_get_retries(&server, login_page()).await;
        assert!(client.token_slot().is_none());
    }

    /// The redirect is detected by the final URL rather than the body, so the
    /// login page here deliberately does not mention the login path.
    #[tokio::test]
    async fn retries_when_the_panel_redirects_to_login() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", "/action/login"))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/action/login"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>please sign in</html>"))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(panel_status())
            .expect(1)
            .mount(&server)
            .await;

        let modes = client(&server).get_status().await.unwrap();
        assert_eq!(modes.area1, Mode::Disarmed);
    }

    /// A session that expires between fetching the token and using it must
    /// still recover: the retried unit includes the token fetch.
    #[tokio::test]
    async fn post_retries_when_the_token_fetch_times_out() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/action/tokenGet"))
            .respond_with(login_page())
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/action/tokenGet"))
            .respond_with(ok_message("tok123"))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .respond_with(ok_message("ok"))
            .expect(1)
            .mount(&server)
            .await;

        client(&server)
            .change_mode(Area::Area1, Mode::Disarmed)
            .await
            .unwrap();
    }
}
