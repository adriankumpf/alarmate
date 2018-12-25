use std::fmt;

#[macro_use]
mod utils;

enum_number!(Mode {
    Disarmed = 0,
    Armed = 1,
    Home1 = 2,
    Home2 = 3,
    Home3 = 4,
});

enum_number!(Area {
    Area1 = 1,
    Area2 = 2,
});

enum_number!(Status {
    Error = 0,
    Ok = 1,
});

enum_number!(DeviceKind {
    Fernbedienung = 2, // 31
    Nachtschalter = 3,
    Tuerkontakt = 4,
    Wassersensor = 5,
    PanicButton = 6, // 7
    Bewegungsmelder = 9,
    RauchUndHitzemelder = 11,
    Gasmelder = 12,
    COMelder = 13,
    Hitzemelder = 14, // 58
    TagReader = 16,
    Glasbruchsensor = 19,
    Temperatursensor = 20, // 75
    MedAlarmmelder = 21,
    MiniInnensirene = 22,
    Sirene = 23,
    PowerSwitch = 24,
    Repeater = 26,
    PIRKamera = 27,
    DrahtloserSensoreingang = 33,
    Keypad = 37, // 17
    Glasbruchmelder = 39,
    Innensirene = 45,
    Aussensirene = 46,
    PowerSwitchMeter = 48,
    UPIC = 52,
    Raumsensor = 54,
    RemoteSwitch = 61,
    Dimmer = 66,
    Rauchmelder = 67,
    Heizungsthermostat = 73,
    Hue = 74,
    Rollladenrelais = 76,
    Lichtsensor = 78,
    Heizkoerperthermostat = 79,
    SmartSwitch = 81,
    Erschuetterungssensor = 93,
});
