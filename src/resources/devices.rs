use serde::{Deserialize, Serialize};

use crate::constants::{Area, DeviceKind, State, Status};
use crate::errors::Result;
use crate::resources::ApiResponse;

/// Holds information about a Lupusic Alarm / Smarthome device
#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    /// The sensor ID
    pub sid: String,
    /// The device kind
    #[serde(rename = "type")]
    pub kind: DeviceKind,
    /// The device name
    pub name: String,
    /// The area the device belongs to
    pub area: Area,
    /// The device state
    #[serde(rename = "status_ex")]
    pub state: State,
    /// The device condition
    #[serde(rename = "cond_ok")]
    pub condition: Status,
    /// The battery status
    #[serde(rename = "battery_ok")]
    pub battery: Status,
    /// The tamper status
    #[serde(rename = "tamper_ok")]
    pub tamper: Status,
}

#[derive(Deserialize)]
pub(crate) struct List {
    #[serde(rename = "senrows")]
    list: Vec<Device>,
}

impl ApiResponse for List {
    type Type = Vec<Device>;

    fn into_result(self) -> Result<Self::Type> {
        Ok(self.list)
    }
}
