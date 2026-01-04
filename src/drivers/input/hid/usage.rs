//! # HID Usage Pages and Usages
//!
//! Define as Usage Pages e Usages padrão do HID.
//!
//! Usages identificam o propósito semântico de cada campo no report.

// =============================================================================
// USAGE PAGES
// =============================================================================

/// Undefined.
pub const USAGE_PAGE_UNDEFINED: u16 = 0x00;
/// Generic Desktop (mouse, keyboard, joystick).
pub const USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
/// Simulation Controls.
pub const USAGE_PAGE_SIMULATION: u16 = 0x02;
/// VR Controls.
pub const USAGE_PAGE_VR: u16 = 0x03;
/// Sport Controls.
pub const USAGE_PAGE_SPORT: u16 = 0x04;
/// Game Controls.
pub const USAGE_PAGE_GAME: u16 = 0x05;
/// Generic Device Controls.
pub const USAGE_PAGE_GENERIC_DEVICE: u16 = 0x06;
/// Keyboard/Keypad.
pub const USAGE_PAGE_KEYBOARD: u16 = 0x07;
/// LEDs.
pub const USAGE_PAGE_LED: u16 = 0x08;
/// Button.
pub const USAGE_PAGE_BUTTON: u16 = 0x09;
/// Ordinal.
pub const USAGE_PAGE_ORDINAL: u16 = 0x0A;
/// Telephony.
pub const USAGE_PAGE_TELEPHONY: u16 = 0x0B;
/// Consumer.
pub const USAGE_PAGE_CONSUMER: u16 = 0x0C;
/// Digitizer.
pub const USAGE_PAGE_DIGITIZER: u16 = 0x0D;
/// Haptics.
pub const USAGE_PAGE_HAPTICS: u16 = 0x0E;
/// Physical Input Device.
pub const USAGE_PAGE_PID: u16 = 0x0F;
/// Unicode.
pub const USAGE_PAGE_UNICODE: u16 = 0x10;
/// Sensor.
pub const USAGE_PAGE_SENSOR: u16 = 0x20;

// =============================================================================
// GENERIC DESKTOP USAGES
// =============================================================================

/// Pointer.
pub const USAGE_POINTER: u16 = 0x01;
/// Mouse.
pub const USAGE_MOUSE: u16 = 0x02;
/// Joystick.
pub const USAGE_JOYSTICK: u16 = 0x04;
/// Game Pad.
pub const USAGE_GAME_PAD: u16 = 0x05;
/// Keyboard.
pub const USAGE_KEYBOARD: u16 = 0x06;
/// Keypad.
pub const USAGE_KEYPAD: u16 = 0x07;
/// Multi-axis Controller.
pub const USAGE_MULTI_AXIS: u16 = 0x08;
/// Tablet PC System Controls.
pub const USAGE_TABLET_PC: u16 = 0x09;

/// X axis.
pub const USAGE_X: u16 = 0x30;
/// Y axis.
pub const USAGE_Y: u16 = 0x31;
/// Z axis.
pub const USAGE_Z: u16 = 0x32;
/// Rx (rotation X).
pub const USAGE_RX: u16 = 0x33;
/// Ry (rotation Y).
pub const USAGE_RY: u16 = 0x34;
/// Rz (rotation Z).
pub const USAGE_RZ: u16 = 0x35;
/// Slider.
pub const USAGE_SLIDER: u16 = 0x36;
/// Dial.
pub const USAGE_DIAL: u16 = 0x37;
/// Wheel.
pub const USAGE_WHEEL: u16 = 0x38;
/// Hat Switch.
pub const USAGE_HAT_SWITCH: u16 = 0x39;

// =============================================================================
// DIGITIZER USAGES
// =============================================================================

/// Digitizer.
pub const USAGE_DIGITIZER: u16 = 0x01;
/// Pen.
pub const USAGE_PEN: u16 = 0x02;
/// Light Pen.
pub const USAGE_LIGHT_PEN: u16 = 0x03;
/// Touch Screen.
pub const USAGE_TOUCH_SCREEN: u16 = 0x04;
/// Touch Pad.
pub const USAGE_TOUCH_PAD: u16 = 0x05;

/// Tip Pressure.
pub const USAGE_TIP_PRESSURE: u16 = 0x30;
/// Barrel Pressure.
pub const USAGE_BARREL_PRESSURE: u16 = 0x31;
/// In Range.
pub const USAGE_IN_RANGE: u16 = 0x32;
/// Touch.
pub const USAGE_TOUCH: u16 = 0x33;
/// Untouch.
pub const USAGE_UNTOUCH: u16 = 0x34;
/// Tap.
pub const USAGE_TAP: u16 = 0x35;
/// Tip Switch.
pub const USAGE_TIP_SWITCH: u16 = 0x42;
/// Secondary Tip Switch.
pub const USAGE_SECONDARY_TIP_SWITCH: u16 = 0x43;
/// Barrel Switch.
pub const USAGE_BARREL_SWITCH: u16 = 0x44;
/// Eraser.
pub const USAGE_ERASER: u16 = 0x45;
/// Tablet Pick.
pub const USAGE_TABLET_PICK: u16 = 0x46;
/// Contact Identifier.
pub const USAGE_CONTACT_ID: u16 = 0x51;
/// Contact Count.
pub const USAGE_CONTACT_COUNT: u16 = 0x54;
/// Contact Count Maximum.
pub const USAGE_CONTACT_COUNT_MAX: u16 = 0x55;

// =============================================================================
// CONSUMER USAGES
// =============================================================================

/// Consumer Control.
pub const USAGE_CONSUMER_CONTROL: u16 = 0x01;
/// Power.
pub const USAGE_POWER: u16 = 0x30;
/// Sleep.
pub const USAGE_SLEEP: u16 = 0x32;
/// Menu.
pub const USAGE_MENU: u16 = 0x40;
/// Volume.
pub const USAGE_VOLUME: u16 = 0xE0;
/// Volume Increment.
pub const USAGE_VOLUME_UP: u16 = 0xE9;
/// Volume Decrement.
pub const USAGE_VOLUME_DOWN: u16 = 0xEA;
/// Mute.
pub const USAGE_MUTE: u16 = 0xE2;
/// Play/Pause.
pub const USAGE_PLAY_PAUSE: u16 = 0xCD;
/// Stop.
pub const USAGE_STOP: u16 = 0xB7;
/// Scan Next Track.
pub const USAGE_SCAN_NEXT: u16 = 0xB5;
/// Scan Previous Track.
pub const USAGE_SCAN_PREV: u16 = 0xB6;

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Retorna nome de uma Usage Page.
pub fn usage_page_name(page: u16) -> &'static str {
    match page {
        USAGE_PAGE_GENERIC_DESKTOP => "Generic Desktop",
        USAGE_PAGE_SIMULATION => "Simulation",
        USAGE_PAGE_VR => "VR",
        USAGE_PAGE_SPORT => "Sport",
        USAGE_PAGE_GAME => "Game",
        USAGE_PAGE_KEYBOARD => "Keyboard",
        USAGE_PAGE_LED => "LED",
        USAGE_PAGE_BUTTON => "Button",
        USAGE_PAGE_CONSUMER => "Consumer",
        USAGE_PAGE_DIGITIZER => "Digitizer",
        USAGE_PAGE_SENSOR => "Sensor",
        _ => "Unknown",
    }
}

/// Retorna nome de um Usage (Generic Desktop).
pub fn generic_desktop_usage_name(usage: u16) -> &'static str {
    match usage {
        USAGE_POINTER => "Pointer",
        USAGE_MOUSE => "Mouse",
        USAGE_JOYSTICK => "Joystick",
        USAGE_GAME_PAD => "Game Pad",
        USAGE_KEYBOARD => "Keyboard",
        USAGE_KEYPAD => "Keypad",
        USAGE_X => "X",
        USAGE_Y => "Y",
        USAGE_Z => "Z",
        USAGE_WHEEL => "Wheel",
        _ => "Unknown",
    }
}
