use reqwest::header;
use serde::Serialize;

use std::net::Ipv4Addr;

use crate::constants::{Area, Mode};
use crate::err;
use crate::resources::{devices, panel, result};
use crate::Result;

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
        Client {
            client: reqwest::Client::new(),
            username: username.into(),
            password: password.into(),
            ip_address,
            token: None,
        }
    }

    /// Get the status of the Alarm Panel
    pub fn get_status(&self) -> Result<((Area, Mode), (Area, Mode))> {
        Ok(self.get::<panel::Status>("panelCondGet")?.inner())
    }

    /// Change the mode of the given area
    pub fn change_mode(&mut self, area: Area, mode: Mode) -> Result {
        let payload = &[("mode", mode as u8), ("area", area as u8)];

        self.post::<_, result::Result>("panelCondPost", payload)?
            .ok()?;

        Ok(())
    }

    /// List all devices managed by the alarm panel
    pub fn list_devices(&self) -> Result<Vec<devices::Device>> {
        Ok(self.get::<devices::List>("deviceListGet")?.inner())
    }

    // Private

    fn url(&self, path: &str) -> Result<reqwest::Url> {
        Ok(format!("https://{}/action/{}", self.ip_address, path).parse()?)
    }

    fn get<T>(&self, action: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .client
            .get(self.url(action)?)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        Ok(from_response_into(response)?)
    }

    fn post<T, D>(&mut self, action: &str, form: &T) -> Result<D>
    where
        T: Serialize + ?Sized,
        D: serde::de::DeserializeOwned,
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

        Ok(from_response_into(response)?)
    }

    fn get_token(&self) -> Result<String> {
        Ok(self.get::<result::Result>("tokenGet")?.ok()?)
    }
}

fn from_response_into<D>(mut response: reqwest::Response) -> Result<D>
where
    D: serde::de::DeserializeOwned,
{
    if !response.status().is_success() {
        return err!("B: {:?}: {}", response.status(), response.text()?);
    }

    let response = response.text()?.replace("\u{009}", "");

    Ok(serde_json::from_str(&response)?)
}
