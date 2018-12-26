use serde::Deserialize;

use crate::constants::{Area, DeviceKind, Status};

/// Holds information about a Lupusic Alarm / Smarthome device
#[derive(Deserialize, Debug)]
pub struct Device {
    sid: String,
    #[serde(rename = "type")]
    kind: DeviceKind,
    name: String,
    area: Area,
    #[serde(rename = "cond_ok")]
    condition: Status,
    #[serde(rename = "battery_ok")]
    battery: Status,
    #[serde(rename = "tamper_ok")]
    tamper: Status,
}

#[derive(Deserialize)]
pub struct List {
    #[serde(rename = "senrows")]
    list: Vec<Device>,
}

impl List {
    pub fn inner(self) -> Vec<Device> {
        self.list
    }
}
