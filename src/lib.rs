// #![deny(warnings)]

mod resources;

use regex::Regex;
use reqwest::header;
use serde::{self, Deserialize};

use std::error::Error;
use std::net::Ipv4Addr;
use std::result;

pub use self::resources::{Area, Mode};
use self::resources::{DeviceKind, Status};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) }
}

pub type Result<T = ()> = result::Result<T, Box<dyn Error>>;

const X_TOKEN: &'static str = "x-token";

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    username: String,
    password: String,
    ip_address: Ipv4Addr,
    // TODO
    pub token: Option<String>,
}

impl Client {
    pub fn new(username: &str, password: &str, ip_address: Ipv4Addr) -> Client {
        Client {
            client: reqwest::Client::new(),
            username: username.into(),
            password: password.into(),
            ip_address: ip_address.into(),
            token: None,
        }
    }

    pub fn list_devices(&self) -> Result<Vec<Device>> {
        let url = self.url("deviceListGet")?;

        let mut response = self
            .client
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        if !response.status().is_success() {
            return err!("{}", response.text()?);
        }

        let response = response.text()?;
        let response = Regex::new(r"\u0009")?.replace_all(&response, " ");
        let device_list: DeviceList = serde_json::from_str(&response)?;

        Ok(device_list.list)
    }

    pub fn get_status(&self) -> Result<((Area, Mode), (Area, Mode))> {
        let url = self.url("panelCondGet")?;

        let response = self
            .client
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        let status: PanelCondStatus = ApiResponse::from_response_into(response)?;

        Ok((
            (Area::Area1, status.forms.pcondform1.mode),
            (Area::Area2, status.forms.pcondform2.mode),
        ))
    }

    pub fn change_mode(&self, area: Area, mode: Mode) -> Result {
        let url = self.url("panelCondPost")?;

        // TODO
        let token = self.token.clone().unwrap();

        let response = self
            .client
            .post(url)
            .form(&[("mode", mode as u8), ("area", area as u8)])
            .basic_auth(&self.username, Some(&self.password))
            .header(X_TOKEN, header::HeaderValue::from_str(&token)?)
            .send()?;

        ApiResponse::from_response(response)?.message()?;

        Ok(())
    }

    fn url(&self, path: &str) -> Result<reqwest::Url> {
        Ok(format!("https://{}/action/{}", self.ip_address, path).parse()?)
    }

    pub fn get_token(&self) -> Result<String> {
        let url = self.url("tokenGet")?;

        let response = self
            .client
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        ApiResponse::from_response(response)?.message()
    }
}

#[derive(Deserialize, Debug)]
struct DeviceList {
    #[serde(rename = "senrows")]
    list: Vec<Device>,
}

#[derive(Deserialize, Debug)]
pub struct Device {
    sid: String,
    #[serde(rename = "type")]
    kind: DeviceKind,
    name: String,
    area: Area,
    #[serde(rename = "cond_ok")]
    cond: Status,
    #[serde(rename = "battery_ok")]
    battery: Status,
    #[serde(rename = "tamper_ok")]
    tamper: Status,
}

#[derive(Deserialize, Debug)]
struct PanelCondStatus {
    forms: Forms,
}

#[derive(Deserialize, Debug)]
struct Forms {
    pcondform1: CondForm,
    pcondform2: CondForm,
}

#[derive(Deserialize, Debug)]
struct CondForm {
    mode: Mode,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    result: Status,
    message: String,
}

impl ApiResponse {
    fn from_response(mut response: reqwest::Response) -> Result<Self> {
        if !response.status().is_success() {
            return err!("A: {}", response.text()?);
        }

        Ok(response.json()?)
    }

    fn from_response_into<D>(mut response: reqwest::Response) -> Result<D>
    where
        D: serde::de::DeserializeOwned,
    {
        if !response.status().is_success() {
            return err!("B: {}", response.text()?);
        }

        let response = response.text()?;
        let response = Regex::new(r"\u0009")?.replace_all(&response, " ");

        Ok(serde_json::from_str(&response)?)
    }

    fn message(self) -> Result<String> {
        if let Status::Error = self.result {
            return err!("C: {}", self.message);
        }

        Ok(self.message)
    }
}
