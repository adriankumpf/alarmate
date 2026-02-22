use serde::{Deserialize, Serialize};

use crate::constants::{Area, DeviceKind, State, Status};
use crate::errors::Result;
use crate::resources::ApiResponse;

/// Holds information about a Lupusec Alarm / Smarthome device
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_device_json() -> serde_json::Value {
        serde_json::json!({
            "sid": "RF:001",
            "type": 4,
            "name": "Front Door",
            "area": 1,
            "status_ex": 0,
            "cond_ok": 1,
            "battery_ok": 1,
            "tamper_ok": 1
        })
    }

    #[test]
    fn deserialize_known_device() {
        let device: Device = serde_json::from_value(sample_device_json()).unwrap();
        assert_eq!(device.sid, "RF:001");
        assert_eq!(device.kind, DeviceKind::DoorContact);
        assert_eq!(device.state, State::Closed);
        assert_eq!(device.condition, Status::Ok);
    }

    #[test]
    fn unknown_device_type_fails() {
        let json = serde_json::json!({
            "sid": "RF:002",
            "type": 999,
            "name": "Unknown",
            "area": 1,
            "status_ex": 0,
            "cond_ok": 1,
            "battery_ok": 1,
            "tamper_ok": 1
        });
        assert!(serde_json::from_value::<Device>(json).is_err());
    }

    #[test]
    fn tolerant_list_empty() {
        let json = serde_json::json!({ "senrows": [] });
        let list: List = serde_json::from_value(json).unwrap();
        assert!(list.into_result().unwrap().is_empty());
    }
}
