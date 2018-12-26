use std::fmt;

use crate::enum_number;

enum_number!(
/// Possible modes of an area
Mode {
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
});

enum_number!(
/// Areas of an alarm panel
Area {
    /// Area 1
    Area1 = 1,

    /// Area 2
    Area2 = 2,
});

enum_number!(
/// Possible status of an API response
Status {
    /// Error
    Error = 0,

    /// Ok
    Ok = 1,
});

enum_number!(
/// Enumeration of Lupusec Alarm & Smarthome devices (incomplete)
DeviceKind {
    /// Remote control V1
    RemoteControlV1 = 2,

    /// Remote control V2
    RemoteControlV2= 31,

    /// Nachtschalter
    Nachtschalter = 3,

    /// Door contact
    DoorContact = 4,

    /// Water sensor
    WaterSensor = 5,

    /// Panic Button V1
    PanicButtonV1 = 6,

    /// Panic Button V2
    PanicButtonV2 = 7,

    /// Motion sensor
    MotionSensor = 9,

    /// CO Detector V1
    CoDetectorV1 = 12,

    /// CO Detector V2
    CoDetectorV2 = 13,

    /// Heat detector V1
    HeatDetectorV1 = 14,

    /// Heat detector V2
    HeatDetectorV2 = 58,

    /// Tag Reader
    TagReader = 16,

    /// Temperature sensor V1
    TemperatureSensorV1 = 20,

    /// Temperature sensor V2
    TemperatureSensorV2 = 75,

    /// Medical Emergency Controller
    MedicalEmergencyController = 21,

    /// Small indoor siren
    SmallIndoorSiren = 22,

    /// Siren
    Siren = 23,

    /// PowerSwitch
    PowerSwitch = 24,

    /// Repeater
    Repeater = 26,

    /// PIR network camera
    PIRNetworkCamera = 27,

    /// Keypad V1
    KeypadV1 = 17,

    /// Keypad V2
    KeypadV2 = 37,

    /// Glas breaking sensor
    GlasBreakingSensorV1 = 19,

    /// Glas breaking sensor
    GlasBreakingSensorV2 = 39,

    /// Indoor siren
    IndoorSiren = 45,

    /// Outdoor Siren
    OutdoorSiren = 46,

    /// Power Switch Meter
    PowerSwitchMeter = 48,

    /// UPIC
    UPIC = 52,

    /// Smoke detector V1
    SmokeDetectorV1 = 54,

    /// Smoke detector V2
    SmokeDetectorV2 = 67,

    /// Smoke / Heat detector
    SmokeAndHeatDetector = 11,

    /// Remote Switch
    RemoteSwitch = 61,

    /// Dimmer
    Dimmer = 66,

    /// Radiator thermostat V1
    RadiatorThermostatV1   = 73,

    /// Radiator thermostat V2
    RadiatorThermostatV2   = 79,

    /// Hue
    Hue = 74,

    /// Shutter relay
    ShutterRelay = 76,

    /// Light sensor
    LightSensor = 78,


    /// Smart Switch
    SmartSwitch = 81,

    /// Vibration sensor
    VibrationSensor = 93,
});
