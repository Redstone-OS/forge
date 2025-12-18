//! /dev/zero - Retorna zeros infinitos
//!
//! Dispositivo essencial do kernel.

use crate::fs::devfs::char_device::CharDevice;
use crate::fs::devfs::device::{Device, DeviceNumber, DeviceType};

/// /dev/zero device
pub struct ZeroDevice {
    dev: DeviceNumber,
}

impl ZeroDevice {
    /// Cria um novo /dev/zero
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(1, 5),
        }
    }
}

impl Device for ZeroDevice {
    fn name(&self) -> &str {
        "zero"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        // Preenche o buffer com zeros
        for byte in buf.iter_mut() {
            *byte = 0;
        }
        Ok(buf.len())
    }

    fn write(&self, buf: &[u8]) -> Result<usize, &'static str> {
        // Descarta tudo (como /dev/null)
        Ok(buf.len())
    }
}

impl CharDevice for ZeroDevice {}

impl Default for ZeroDevice {
    fn default() -> Self {
        Self::new()
    }
}
