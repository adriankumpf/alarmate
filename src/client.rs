use reqwest::header;
use serde::Serialize;
use serde::de::DeserializeOwned;

use std::future::Future;
use std::net::Ipv4Addr;
use std::sync::Mutex;
use std::time::Duration;

use crate::Modes;
use crate::constants::{Area, Mode};
use crate::errors::{Error, Result};
use crate::resources::{ApiResponse, devices, panel, response};

/// How long to wait for a complete response before giving up.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// How long to wait for the TCP and TLS handshake with the panel.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Upper bound on the response body retained in [`Error::UnexpectedResponse`].
///
/// The panel serves full HTML pages on failure, which are large and of little
/// diagnostic value beyond their first few lines.
const MAX_ERROR_BODY: usize = 512;

/// An asynchronous client for the LUPUSEC HTTP API.
///
/// The client owns a connection pool and caches the panel's session token, so a
/// single instance should be shared rather than created per request. Every
/// method takes `&self`, so an `Arc<Client>` can be used from multiple tasks.
pub struct Client {
    http: reqwest::Client,
    username: String,
    password: String,
    base_url: reqwest::Url,
    token: Mutex<Option<String>>,
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
    /// Retries once if the panel reports a session timeout or an unauthorized
    /// error, both of which it returns transiently.
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
    /// Retries once if the panel reports a session timeout or an unauthorized
    /// error, both of which it returns transiently.
    ///
    /// # Errors
    ///
    /// Returns an error if the panel cannot be reached, rejects the
    /// credentials, or refuses the mode change.
    pub async fn change_mode(&self, area: Area, mode: Mode) -> Result {
        let payload = &[("mode", mode as u8), ("area", area as u8)];

        self.post::<_, response::Response>("panelCondPost", payload)
            .await?;

        Ok(())
    }

    /// List all devices managed by the alarm panel.
    ///
    /// Retries once if the panel reports a session timeout or an unauthorized
    /// error, both of which it returns transiently.
    ///
    /// # Errors
    ///
    /// Returns an error if the panel cannot be reached, rejects the
    /// credentials, or reports a device type this crate does not know.
    pub async fn list_devices(&self) -> Result<Vec<devices::Device>> {
        self.get::<devices::List>("deviceListGet").await
    }

    async fn get<T>(&self, action: &str) -> Result<T::Output>
    where
        T: ApiResponse + DeserializeOwned,
    {
        self.send_retrying::<T, _>(|| async move { self.send_get(action).await })
            .await
    }

    async fn post<T, D>(&self, action: &str, form: &T) -> Result<D::Output>
    where
        T: Serialize + ?Sized,
        D: ApiResponse + DeserializeOwned,
    {
        self.send_retrying::<D, _>(|| async move {
            let token = self.token().await?;
            self.send_post(action, form, &token).await
        })
        .await
    }

    /// Send a request, retrying it once if the panel's session state is stale.
    ///
    /// The panel also returns 401 transiently when its session state is
    /// confused, so an unauthorized response is retried like an expired one.
    /// The cost is that genuinely wrong credentials are tried twice.
    ///
    /// Either way the session as a whole is suspect, so the cached token is
    /// dropped before retrying. GET requests do not use the token, but leaving
    /// a dead one behind would make the next POST fail.
    ///
    /// `send` is an `Fn` returning a named future rather than an `AsyncFn`:
    /// `AsyncFn` carries a higher-ranked lifetime, which makes the resulting
    /// future impossible to prove `Send` and breaks callers that need one — an
    /// axum handler, for instance.
    async fn send_retrying<D, F>(&self, send: impl Fn() -> F) -> Result<D::Output>
    where
        D: ApiResponse + DeserializeOwned,
        F: Future<Output = Result<reqwest::Response>>,
    {
        match parse::<D>(send().await?).await {
            Err(Error::SessionTimeout | Error::Unauthorized) => {
                *self.token.lock().expect("token lock poisoned") = None;
                parse::<D>(send().await?).await
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

    /// Return the cached session token, requesting a new one if there is none.
    async fn token(&self) -> Result<String> {
        if let Some(token) = self.token.lock().expect("token lock poisoned").clone() {
            return Ok(token);
        }

        // Deliberately not routed through `send_retrying`: the caller is already
        // wrapped in it, and it re-runs the whole request — this fetch included
        // — after a session timeout. Nesting the two would square the number of
        // requests a single call can make.
        let token = parse::<response::Response>(self.send_get("tokenGet").await?).await?;
        *self.token.lock().expect("token lock poisoned") = Some(token.clone());

        Ok(token)
    }
}

async fn parse<D>(res: reqwest::Response) -> Result<D::Output>
where
    D: ApiResponse + DeserializeOwned,
{
    // reqwest has already followed the panel's redirect, so the final URL is the
    // most direct evidence that the session expired.
    if res.url().path().ends_with("/action/login") {
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
        // The panel serves the /action/login page instead of JSON once the
        // session has expired. Report that as a timeout so the caller can retry
        // with a fresh session, rather than as a confusing serde error.
        Err(_) if body.contains("/action/login") => Err(Error::SessionTimeout),
        Err(e) => Err(e.into()),
        Ok(model) => Ok(model),
    }
}

/// Deserialize a panel response, working around the raw tabs it emits.
///
/// `serde_json` accepts tabs between tokens but rejects them inside string
/// values, so a body that fails to parse is retried with the tabs removed. That
/// costs a copy only on the bodies that actually need it, and it does mean a tab
/// inside e.g. a device name is dropped rather than preserved.
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
    match body.char_indices().nth(MAX_ERROR_BODY) {
        Some((end, _)) => format!("{}… ({} bytes total)", &body[..end], body.len()),
        None => body.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
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

    #[test]
    fn parse_body_truncates_long_error_bodies() {
        let body = "x".repeat(MAX_ERROR_BODY * 2);
        let result: Result<response::Response> =
            parse_body(reqwest::StatusCode::INTERNAL_SERVER_ERROR, &body);

        let Err(Error::UnexpectedResponse { body, .. }) = result else {
            panic!("expected an unexpected-response error");
        };
        assert!(body.len() < MAX_ERROR_BODY * 2);
        assert!(body.contains("1024 bytes total"));
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

    #[tokio::test]
    async fn get_retries_on_session_timeout() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>/action/login</html>"))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "forms": {
                    "pcondform1": { "mode": 0 },
                    "pcondform2": { "mode": 1 }
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let modes = client(&server).get_status().await.unwrap();
        assert_eq!(modes.area1, Mode::Disarmed);
        assert_eq!(modes.area2, Mode::Armed);
    }

    #[tokio::test]
    async fn post_retries_on_session_timeout() {
        let server = MockServer::start().await;

        // Once for the initial attempt, once after the timeout drops the token.
        Mock::given(method("GET"))
            .and(path("/action/tokenGet"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": 1, "message": "tok123"})),
            )
            .expect(2)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>/action/login</html>"))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": 1, "message": "ok"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let result = client(&server)
            .change_mode(Area::Area1, Mode::Disarmed)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_retries_on_unauthorized() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "forms": {
                    "pcondform1": { "mode": 0 },
                    "pcondform2": { "mode": 1 }
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let modes = client(&server).get_status().await.unwrap();
        assert_eq!(modes.area1, Mode::Disarmed);
        assert_eq!(modes.area2, Mode::Armed);
    }

    #[tokio::test]
    async fn post_retries_on_unauthorized() {
        let server = MockServer::start().await;

        // Once for the initial attempt, once after the 401 drops the token.
        Mock::given(method("GET"))
            .and(path("/action/tokenGet"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": 1, "message": "tok123"})),
            )
            .expect(2)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/action/panelCondPost"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": 1, "message": "ok"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let result = client(&server)
            .change_mode(Area::Area1, Mode::Disarmed)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_timeout_drops_the_cached_token() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>/action/login</html>"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/action/panelCondGet"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "forms": { "pcondform1": { "mode": 0 }, "pcondform2": { "mode": 0 } }
            })))
            .mount(&server)
            .await;

        let client = client(&server);
        *client.token.lock().unwrap() = Some("stale".into());
        client.get_status().await.unwrap();

        assert_eq!(*client.token.lock().unwrap(), None);
    }
}
