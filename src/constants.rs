use num_enum::TryFromPrimitive;
use strum::{Display, EnumString};

/// Possible modes of an area
#[derive(Clone, Copy, Debug, Eq, PartialEq, Display, EnumString, TryFromPrimitive)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "build-binary", derive(clap::ValueEnum))]
#[repr(u8)]
pub enum Mode {
    /// Disarmed
    Disarmed = 0,

    /// Armed
    Armed = 1,

    /// Home 1
    Home1 = 2,

    /// Home 2
    Home2 = 3,

    /// Home 3
    Home3 = 4,
}

/// Areas of an alarm panel
#[derive(Clone, Copy, Debug, Eq, PartialEq, Display, EnumString, TryFromPrimitive)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "build-binary", derive(clap::ValueEnum))]
#[repr(u8)]
pub enum Area {
    /// Area 1
    Area1 = 1,

    /// Area 2
    Area2 = 2,
}

/// Possible status of an API response
#[derive(Clone, Copy, Debug, Eq, PartialEq, Display, EnumString, TryFromPrimitive)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "build-binary", derive(clap::ValueEnum))]
#[repr(u8)]
pub enum Status {
    /// Error
    Error = 0,

    /// Ok
    Ok = 1,
}

/// Possible states of a binary sensor
#[derive(Clone, Copy, Debug, Eq, PartialEq, Display, EnumString, TryFromPrimitive)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "build-binary", derive(clap::ValueEnum))]
#[repr(u8)]
pub enum State {
    /// Closed
    Closed = 0,

    /// Open
    Open = 1,
}

/// Enumeration of Lupusec Alarm & Smarthome devices (incomplete)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Display, EnumString, TryFromPrimitive)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "build-binary", derive(clap::ValueEnum))]
#[repr(u8)]
pub enum DeviceKind {
    /// Remote Control
    RemoteControl1 = 2,

    /// Night Switch
    NightSwitch = 3,

    /// Door Contact
    DoorContact = 4,

    /// water sensor
    WaterSensor = 5,

    /// Panic Button
    PanicButton1 = 6,

    /// Panic Button
    PanicButton2 = 7,

    /// KHL
    KHL = 8,

    /// Motion Detector
    MotionDetector = 9,

    /// outdoor motion detector
    OutdoorMotionDetector = 10,

    /// Smoke / Heat detector
    SmokedetectorAndHeatDetector = 11,

    /// GAS Detector
    GasDetector = 12,

    /// CO Detector
    CoDetector = 13,

    /// heat detector
    HeatDetector1 = 14,

    /// Keypad
    Keypad1 = 15,

    /// Tag Reader
    TagReader1 = 16,

    /// Keypad
    Keypad2 = 17,

    /// Keypad
    Keypad3 = 18,

    /// glass break sensor,
    GlassBreakSensor = 19,

    /// Temperature sensor
    TemperatureSensor1 = 20,

    /// Med Alarm Detectors
    MedAlarmDetectors = 21,

    /// Mini indoor siren / status indicator
    MiniindoorsirenAndStatusIndicator = 22,

    /// Siren
    Siren = 23,

    /// Power Switch
    PowerSwitch1 = 24,

    /// Power Switch
    PowerSwitch2 = 25,

    /// Repeater
    Repeater = 26,

    /// PIR Camera
    PirCamera = 27,

    /// Out View
    OutView = 29,

    /// Remote Control
    RemoteControl2 = 31,

    /// PCT
    PCT = 32,

    /// Sensor Input
    SensorInput = 33,

    /// Sudden Motion Sensor
    SuddenMotionSensor1 = 34,

    /// NT
    NT = 35,

    /// UT
    UT = 36,

    /// Keypad
    Keypad4 = 37,

    /// Tag Reader
    TagReader2 = 38,

    /// Glass break detector
    GlassBreakDetector = 39,

    /// Temperature sensor
    TemperatureSensor2 = 40,

    /// Temperature sensor
    TemperatureSensor3 = 41,

    /// Temperature sensor
    TemperatureSensor4 = 42,

    /// WTGGPS
    WTGGPS = 43,

    /// Dialer
    Dialer = 44,

    /// indoor siren
    IndoorSiren = 45,

    /// outdoor siren
    OutdoorSiren = 46,

    /// HRRs
    HRRs = 47,

    /// Power Switch meters
    PowerSwitchMeters = 48,

    /// WTRV
    WTRV = 49,

    /// Power Meter
    PowerMeter = 50,

    /// Thermostat (Danfoss)
    ThermostatDanfoss = 51,

    /// UPIC
    UPIC = 52,

    /// Dimmer
    Dimmer1 = 53,

    /// Room sensor
    RoomSensor = 54,

    /// Radon sensor
    RadonSensor = 55,

    /// thermostat (Horstmann)
    ThermostatHorstmann = 56,

    /// Door Lock
    DoorLock = 57,

    /// heat detector
    HeatDetector2 = 58,

    /// Sudden Motion Sensor
    SuddenMotionSensor2 = 59,

    /// Sudden Motion Sensor
    SuddenMotionSensor3 = 60,

    /// Remote Switch
    RemoteSwitch = 61,

    /// Heat Meter
    HeatMeter = 62,

    /// Water Meter
    WaterMeter = 63,

    /// gas meters
    GasMeters = 64,

    /// Dimmer
    Dimmer2 = 66,

    /// Smoke Detector
    SmokeDetector = 67,

    /// Thermostat (Elko)
    ThermostatElko = 68,

    /// IP Camera
    IpCamera = 69,

    /// Door Lock (Secure)
    DoorlockSecure = 70,

    /// Thermostat (RCS)
    ThermostatRcs_ = 71,

    /// Door Lock (Yale)
    DoorlockYale = 72,

    /// Thermostat
    Thermostat = 73,

    /// Hue
    Hue = 74,

    /// Temperature Sensor
    TemperatureSensor5 = 75,

    /// Shutter
    Shutter = 76,

    /// Lightsensor
    Lightsensor = 78,

    /// Radiator Thermostat
    RadiatorThermostat = 79,

    /// Awning
    Awning = 80,

    /// Smart Switch
    SmartSwitch = 81,

    /// Shocksensor
    Shocksensor = 93,
}

impl_numeric_serde!(Mode, Area, Status, State, DeviceKind);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_deserialize_from_number() {
        let mode: Mode = serde_json::from_str("0").unwrap();
        assert_eq!(mode, Mode::Disarmed);
    }

    #[test]
    fn mode_deserialize_from_string() {
        let mode: Mode = serde_json::from_str("\"2\"").unwrap();
        assert_eq!(mode, Mode::Home1);
    }

    #[test]
    fn mode_display() {
        assert_eq!(Mode::Armed.to_string(), "Armed");
    }

    #[test]
    fn mode_from_str() {
        assert_eq!("disarmed".parse::<Mode>().unwrap(), Mode::Disarmed);
        assert_eq!("ARMED".parse::<Mode>().unwrap(), Mode::Armed);
    }

    #[test]
    fn mode_from_str_unknown() {
        assert!("unknown".parse::<Mode>().is_err());
    }

    #[test]
    fn area_deserialize_from_number() {
        let area: Area = serde_json::from_str("1").unwrap();
        assert_eq!(area, Area::Area1);
    }

    #[test]
    fn device_kind_unknown_value() {
        let result: std::result::Result<DeviceKind, _> = serde_json::from_str("999");
        assert!(result.is_err());
    }

    #[test]
    fn state_roundtrip() {
        let state: State = serde_json::from_str("0").unwrap();
        assert_eq!(state, State::Closed);
        assert_eq!(state.to_string(), "Closed");
    }

    #[test]
    fn status_roundtrip() {
        let status: Status = serde_json::from_str("1").unwrap();
        assert_eq!(status, Status::Ok);
        assert_eq!(status.to_string(), "Ok");
    }
}
