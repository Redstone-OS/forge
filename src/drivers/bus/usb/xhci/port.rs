//! # xHCI Port Management
//!
//! Gerenciamento de portas USB do xHCI.

use super::regs::*;
use super::types::*;
use crate::drivers::bus::usb::host::PortStatus;
use crate::drivers::bus::usb::types::UsbSpeed;

/// Lê o status de uma porta.
///
/// ## STUB:
/// Não acessa hardware real.
pub fn get_port_status(_base: u64, _port: u8) -> PortStatus {
    crate::kwarn!("(xHCI Port) get_port_status() stub");

    // TODO: Ler PORTSC

    PortStatus::default()
}

/// Realiza reset de uma porta.
///
/// ## STUB:
/// Não acessa hardware real.
pub fn reset_port(_base: u64, _port: u8) -> bool {
    crate::kwarn!("(xHCI Port) reset_port() stub");

    // TODO:
    // 1. Escrever PR bit em PORTSC
    // 2. Aguardar reset completar
    // 3. Verificar PRC bit

    true
}

/// Habilita power em uma porta.
pub fn power_on(_base: u64, _port: u8) {
    crate::kwarn!("(xHCI Port) power_on() stub");
    // TODO: Escrever PP bit
}

/// Limpa flags de mudança de uma porta.
pub fn clear_port_changes(_base: u64, _port: u8) {
    crate::kwarn!("(xHCI Port) clear_port_changes() stub");
    // TODO: Escrever CSC, PEC, etc para limpar
}

/// Converte speed do xHCI para UsbSpeed.
pub fn xhci_speed_to_usb(speed: u8) -> UsbSpeed {
    match speed {
        SPEED_LOW => UsbSpeed::Low,
        SPEED_FULL => UsbSpeed::Full,
        SPEED_HIGH => UsbSpeed::High,
        SPEED_SUPER => UsbSpeed::Super,
        SPEED_SUPER_PLUS => UsbSpeed::SuperPlus,
        _ => UsbSpeed::Full,
    }
}
