use reqwest::header;
use serde::Serialize;

use std::net::Ipv4Addr;

use crate::Modes;
use crate::constants::{Area, Mode};
use crate::errors::{Error, Result};
use crate::resources::{ApiResponse, devices, panel, response};

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
    pub async fn get_status(&self) -> Result<Modes> {
        self.get::<panel::Condition>("panelCondGet").await?.into_result()
    }

    /// Change the mode of the given area
    pub async fn change_mode(&mut self, area: Area, mode: &Mode) -> Result {
        let payload = &[("mode", *mode as u8), ("area", area as u8)];

        self.post::<_, response::Response>("panelCondPost", payload)
            .await?
            .into_result()?;

        Ok(())
    }

    /// List all devices managed by the alarm panel
    pub async fn list_devices(&self) -> Result<Vec<devices::Device>> {
        self.get::<devices::List>("deviceListGet").await?.into_result()
    }

    // Private

    fn url(&self, path: &str) -> Result<reqwest::Url> {
        Ok(format!("https://{}/action/{}", self.ip_address, path)
            .parse()
            .expect("URL must always be valid"))
    }

    async fn get<'s, 'a: 's, T>(&'s self, action: &'a str) -> Result<T>
    where
        T: ApiResponse + serde::de::DeserializeOwned,
    {
        for i in 0..2 {
            let res = self
                .client
                .get(self.url(action)?)
                .basic_auth(&self.username, Some(&self.password))
                .send()
                .await?;

            match from_response_into(res).await {
                Err(ref error) if error.is_session_timeout() && i == 0 => continue,
                other => return other,
            }
        }

        unreachable!()
    }

    async fn post<'s, 'a: 's, T, D>(&'s mut self, action: &'a str, form: &'a T) -> Result<D>
    where
        T: Serialize + Sized,
        D: ApiResponse + serde::de::DeserializeOwned,
    {
        for i in 0..2 {
            let url = self.url(action)?;

            let token = match &self.token {
                Some(token) => token.clone(),
                None => {
                    let token = self.get_token().await?;
                    self.token = Some(token.clone());
                    token
                }
            };

            let res = self
                .client
                .post(url)
                .form(&form)
                .basic_auth(&self.username, Some(&self.password))
                .header("x-token", header::HeaderValue::from_str(&token)?)
                .send()
                .await?;

            match from_response_into(res).await {
                Err(ref error) if error.is_session_timeout() && i == 0 => {
                    self.token = None;
                    continue;
                }
                other => return other,
            }
        }

        unreachable!()
    }

    async fn get_token(&self) -> Result<String> {
        self.get::<response::Response>("tokenGet").await?.into_result()
    }
}

async fn from_response_into<D>(res: reqwest::Response) -> Result<D>
where
    D: ApiResponse + serde::de::DeserializeOwned,
{
    let status = res.status();
    let body = res.text().await?;

    if !status.is_success() {
        let error = if let reqwest::StatusCode::UNAUTHORIZED = status {
            Error::SessionTimeout
        } else {
            Error::UnexpectedResponse { status, body }
        };

        return Err(error);
    }

    match serde_json::from_str(&body.replace('\u{009}', "")) {
        Err(_) if body.contains("/action/login") => Err(Error::SessionTimeout),
        Err(_) => Err(Error::UnexpectedResponse { status, body }),
        Ok(model) => Ok(model),
    }
}
