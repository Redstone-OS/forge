//! # Touchpad Global State
//!
//! Gerenciamento do estado estático e instâncias globais para o driver de touchpad.

use super::hid::{FingerData, HidDescriptor, TouchpadState, TouchpadType};
use super::i2c::I2cController;
use crate::sync::Spinlock;

/// Estado global de movimento e botões
pub static TOUCHPAD_STATE: Spinlock<TouchpadState> = Spinlock::new(TouchpadState {
    finger_count: 0,
    fingers: [FingerData {
        contact_id: 0,
        tip: false,
        in_range: false,
        x: 0,
        y: 0,
        pressure: 0,
    }; 5],
    button_left: false,
    button_right: false,
    cursor_x: 640,
    cursor_y: 400,
    delta_x: 0,
    delta_y: 0,
    screen_width: 1280,
    screen_height: 800,
});

/// Controlador I2C ativo
pub static I2C: Spinlock<Option<I2cController>> = Spinlock::new(None);

/// Endereço I2C do dispositivo detectado
pub static TOUCHPAD_ADDR: Spinlock<u8> = Spinlock::new(0);

/// Descriptor HID carregado do hardware
pub static HID_DESC: Spinlock<Option<HidDescriptor>> = Spinlock::new(None);

/// Fabricante detectado
pub static TOUCHPAD_TYPE: Spinlock<TouchpadType> = Spinlock::new(TouchpadType::Unknown);
