//! /dev/tty* - Terminais (TTY)
//!
//! Dispositivos essenciais do kernel para I/O de terminal.

use crate::fs::devfs::char_device::CharDevice;
use crate::fs::devfs::device::{Device, DeviceNumber, DeviceType};

/// /dev/tty - Terminal atual
pub struct TtyDevice {
    dev: DeviceNumber,
}

impl TtyDevice {
    /// Cria um novo /dev/tty
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(5, 0),
        }
    }
}

impl Device for TtyDevice {
    fn name(&self) -> &str {
        "tty"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implementar leitura do TTY atual
        Err("tty read not implemented")
    }

    fn write(&self, _buf: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implementar escrita no TTY atual
        Err("tty write not implemented")
    }
}

impl CharDevice for TtyDevice {}

/// /dev/ttyS0 - Serial port 0
pub struct TtyS0Device {
    dev: DeviceNumber,
}

impl TtyS0Device {
    /// Cria um novo /dev/ttyS0
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(4, 64),
        }
    }
}

impl Device for TtyS0Device {
    fn name(&self) -> &str {
        "ttyS0"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implementar leitura da serial
        // - Integrar com driver UART (16550)
        Err("ttyS0 read not implemented")
    }

    fn write(&self, _buf: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implementar escrita na serial
        // - Integrar com driver UART (16550)
        Err("ttyS0 write not implemented")
    }
}

impl CharDevice for TtyS0Device {}
