use reqwest::header;
use serde::Serialize;

use std::net::Ipv4Addr;

use crate::Modes;
use crate::constants::{Area, Mode};
use crate::errors::{Error, Result};
use crate::resources::{ApiResponse, devices, panel, response};

/// Holds the credentials and a session token
pub struct Client {
    client: reqwest::Client,
    username: String,
    password: String,
    base_url: reqwest::Url,
    token: Option<String>,
}

impl Client {
    /// Construct a client.
    ///
    /// The client accepts self-signed TLS certificates because LUPUSEC panels
    /// ship with self-signed certs by default.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be built (e.g.
    /// due to malformed proxy environment variables).
    pub fn new(username: &str, password: &str, ip_address: Ipv4Addr) -> Result<Client> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let base_url = format!("https://{ip_address}/action/")
            .parse()
            .expect("base URL should be valid for well-formed IPv4");

        Ok(Client {
            client,
            username: username.into(),
            password: password.into(),
            base_url,
            token: None,
        })
    }

    /// Get the status of the Alarm Panel.
    ///
    /// Automatically retries once if the panel reports a session timeout.
    pub async fn get_status(&mut self) -> Result<Modes> {
        self.get::<panel::Condition>("panelCondGet").await
    }

    /// Change the mode of the given area.
    ///
    /// Automatically retries once if the panel reports a session timeout,
    /// clearing the cached token before the retry.
    pub async fn change_mode(&mut self, area: Area, mode: Mode) -> Result {
        let payload = &[("mode", mode as u8), ("area", area as u8)];

        self.post::<_, response::Response>("panelCondPost", payload)
            .await?;

        Ok(())
    }

    /// List all devices managed by the alarm panel.
    ///
    /// Automatically retries once if the panel reports a session timeout.
    pub async fn list_devices(&mut self) -> Result<Vec<devices::Device>> {
        self.get::<devices::List>("deviceListGet").await
    }

    fn url(&self, path: &str) -> reqwest::Url {
        self.base_url
            .join(path)
            .expect("action path should be a valid relative URL segment")
    }

    async fn get<T>(&mut self, action: &str) -> Result<T::Type>
    where
        T: ApiResponse + serde::de::DeserializeOwned,
    {
        let res = self.send_get(action).await?;
        match parse_and_convert::<T>(res).await {
            Err(ref e) if e.is_session_timeout() => {}
            other => return other,
        }
        parse_and_convert::<T>(self.send_get(action).await?).await
    }

    async fn post<T, D>(&mut self, action: &str, form: &T) -> Result<D::Type>
    where
        T: Serialize,
        D: ApiResponse + serde::de::DeserializeOwned,
    {
        let token = self.get_or_fetch_token().await?;
        let res = self.send_post(action, form, &token).await?;
        match parse_and_convert::<D>(res).await {
            Err(ref e) if e.is_session_timeout() => {}
            other => return other,
        }
        self.token = None;
        let token = self.get_or_fetch_token().await?;
        parse_and_convert::<D>(self.send_post(action, form, &token).await?).await
    }

    async fn send_get(&self, action: &str) -> Result<reqwest::Response> {
        Ok(self
            .client
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
        Ok(self
            .client
            .post(self.url(action))
            .form(form)
            .basic_auth(&self.username, Some(&self.password))
            .header("x-token", header::HeaderValue::from_str(token)?)
            .send()
            .await?)
    }

    /// Fetch the cached token or request a new one from the panel.
    async fn get_or_fetch_token(&mut self) -> Result<String> {
        if let Some(ref token) = self.token {
            return Ok(token.clone());
        }

        let token = self.get_token().await?;
        self.token = Some(token.clone());
        Ok(token)
    }

    async fn get_token(&mut self) -> Result<String> {
        self.get::<response::Response>("tokenGet").await
    }
}

async fn parse_response<D>(res: reqwest::Response) -> Result<D>
where
    D: ApiResponse + serde::de::DeserializeOwned,
{
    let status = res.status();
    let body = res.text().await?;
    parse_body(status, &body)
}

async fn parse_and_convert<D>(res: reqwest::Response) -> Result<D::Type>
where
    D: ApiResponse + serde::de::DeserializeOwned,
{
    parse_response::<D>(res).await?.into_result()
}

fn parse_body<D>(status: reqwest::StatusCode, body: &str) -> Result<D>
where
    D: ApiResponse + serde::de::DeserializeOwned,
{
    if !status.is_success() {
        let error = if status == reqwest::StatusCode::UNAUTHORIZED {
            Error::Unauthorized
        } else {
            Error::UnexpectedResponse {
                status,
                body: body.to_owned(),
            }
        };

        return Err(error);
    }

    // The panel redirects to /action/login when the session has expired,
    // returning an HTML page instead of JSON. Detect this before reporting
    // the serde error so callers can retry with a fresh session.
    match serde_json::from_str(&body.replace('\u{009}', "")) {
        Err(_) if body.contains("/action/login") => Err(Error::SessionTimeout),
        Err(e) => Err(e.into()),
        Ok(model) => Ok(model),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parse_body_session_timeout() {
        let body = r#"<html>/action/login</html>"#;
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, body);
        assert!(result.unwrap_err().is_session_timeout());
    }

    #[test]
    fn parse_body_invalid_json() {
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, "not json");
        assert!(matches!(result.unwrap_err(), Error::Deserialize(_)));
    }

    #[test]
    fn parse_body_strips_tabs() {
        let body = "{\t\"result\":\t1,\t\"message\":\t\"ok\"\t}";
        let result: Result<response::Response> = parse_body(reqwest::StatusCode::OK, body);
        assert!(result.is_ok());
    }
}
