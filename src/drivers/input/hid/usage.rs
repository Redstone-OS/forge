//! # HID Usage Tables
//!
//! Contém definições para HID Usage Pages e Usages conforme a especificação USB HID.

/// HID Usage Pages
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsagePage {
    Undefined = 0x00,
    GenericDesktop = 0x01,
    SimulationControls = 0x02,
    VRControls = 0x03,
    SportControls = 0x04,
    GameControls = 0x05,
    KeyboardKeypad = 0x07,
    Led = 0x08,
    Button = 0x09,
    Ordinal = 0x0A,
    Telephony = 0x0B,
    Consumer = 0x0C,
    Digitizer = 0x0D,
    Unicode = 0x10,
    AlphanumericDisplay = 0x14,
    MedicalInstruments = 0x40,
}

/// Generic Desktop Usages
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericDesktopUsage {
    Pointer = 0x01,
    Mouse = 0x02,
    Joystick = 0x04,
    Gamepad = 0x05,
    Keyboard = 0x06,
    Keypad = 0x07,
    MultiAxisController = 0x08,
    X = 0x30,
    Y = 0x31,
    Z = 0x32,
    Wheel = 0x38,
    Dial = 0x37,
    SystemSleep = 0x82,
    SystemWakeUp = 0x83,
}

pub struct HidUsage {
    pub page: UsagePage,
    pub usage: u16,
}
