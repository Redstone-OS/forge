//! /dev/null - Descarta tudo que é escrito, retorna EOF ao ler
//!
//! Dispositivo essencial do kernel.

use crate::fs::devfs::char_device::CharDevice;
use crate::fs::devfs::device::{Device, DeviceNumber, DeviceType};

/// /dev/null device
pub struct NullDevice {
    dev: DeviceNumber,
}

impl NullDevice {
    /// Cria um novo /dev/null
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(1, 3),
        }
    }
}

impl Device for NullDevice {
    fn name(&self) -> &str {
        "null"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // Sempre retorna EOF (0 bytes lidos)
        Ok(0)
    }

    fn write(&self, buf: &[u8]) -> Result<usize, &'static str> {
        // Descarta tudo, sempre sucesso
        Ok(buf.len())
    }
}

impl CharDevice for NullDevice {}

impl Default for NullDevice {
    fn default() -> Self {
        Self::new()
    }
}
