use serde::{Deserialize, Serialize};

use crate::constants::{Area, DeviceKind, State, Status};
use crate::errors::Result;
use crate::resources::ApiResponse;

/// Holds information about a Lupusic Alarm / Smarthome device
#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    sid: String,
    #[serde(rename = "type")]
    kind: DeviceKind,
    name: String,
    area: Area,
    #[serde(rename = "status_ex")]
    state: State,
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

impl ApiResponse for List {
    type Type = Vec<Device>;

    fn ok(self) -> Result<Self::Type> {
        Ok(self.list)
    }
}
