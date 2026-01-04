//! # Touchpad Protocols
//!
//! Despachante de relatórios para diferentes fabricantes de touchpad.

pub mod elan;
pub mod synaptics;

use super::hid::TouchpadType;

pub fn process_report(tp_type: TouchpadType, data: &[u8]) {
    match tp_type {
        TouchpadType::Synaptics => synaptics::process_report(data),
        TouchpadType::Elan => elan::process_report(data),
        TouchpadType::Unknown => {}
    }
}
