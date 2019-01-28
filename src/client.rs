use reqwest::header;
use serde::Serialize;

use std::net::Ipv4Addr;

use crate::constants::{Area, Mode};
use crate::errors::{Error, Result};
use crate::resources::{devices, panel, response, ApiResponse};
use crate::Modes;

const X_TOKEN: &str = "x-token";

/// Holds the credentials and a session token
#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    username: String,
    password: String,
    ip_address: Ipv4Addr,
    token: Option<String>,
}

impl Client {
    /// Construct a client
    pub fn new(username: &str, password: &str, ip_address: Ipv4Addr) -> Client {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("could not build reqwest::Client");

        Client {
            client,
            username: username.into(),
            password: password.into(),
            ip_address,
            token: None,
        }
    }

    /// Get the status of the Alarm Panel
    pub fn get_status(&self) -> Result<Modes> {
        self.get::<panel::Condition>("panelCondGet", true)?.ok()
    }

    /// Change the mode of the given area
    pub fn change_mode(&mut self, area: Area, mode: Mode) -> Result {
        let payload = &[("mode", mode as u8), ("area", area as u8)];

        self.post::<_, response::Response>("panelCondPost", payload, true)?
            .ok()?;

        Ok(())
    }

    /// List all devices managed by the alarm panel
    pub fn list_devices(&self) -> Result<Vec<devices::Device>> {
        self.get::<devices::List>("deviceListGet", true)?.ok()
    }

    // Private

    fn url(&self, path: &str) -> Result<reqwest::Url> {
        Ok(format!("https://{}/action/{}", self.ip_address, path).parse()?)
    }

    fn get<T>(&self, action: &str, retry: bool) -> Result<T>
    where
        T: ApiResponse + serde::de::DeserializeOwned,
    {
        let response = self
            .client
            .get(self.url(action)?)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        match from_response_into(response) {
            Err(ref error) if error.is_session_timeout() && retry => self.get(action, false),
            other => other,
        }
    }

    fn post<T, D>(&mut self, action: &str, form: &T, retry: bool) -> Result<D>
    where
        T: Serialize + ?Sized,
        D: ApiResponse + serde::de::DeserializeOwned,
    {
        let url = self.url(action)?;

        let token = match &self.token {
            Some(token) => token.clone(),
            None => {
                let token = self.get_token()?;
                self.token = Some(token.clone());
                token
            }
        };

        let response = self
            .client
            .post(url)
            .form(form)
            .basic_auth(&self.username, Some(&self.password))
            .header(X_TOKEN, header::HeaderValue::from_str(&token)?)
            .send()?;

        match from_response_into(response) {
            Err(ref error) if error.is_session_timeout() && retry => {
                self.token = None;
                self.post(action, form, false)
            }
            other => other,
        }
    }

    fn get_token(&self) -> Result<String> {
        Ok(self.get::<response::Response>("tokenGet", true)?.ok()?)
    }
}

fn from_response_into<D>(mut response: reqwest::Response) -> Result<D>
where
    D: ApiResponse + serde::de::DeserializeOwned,
{
    if !response.status().is_success() {
        return Err(Error::Panel(format!(
            "{}: {}",
            response.status(),
            response.text()?
        )));
    }

    let response = response.text()?.replace("\u{009}", "");
    let model = serde_json::from_str(&response)?;

    Ok(model)
}
