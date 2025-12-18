//! /dev/console - Console do sistema (TTY do kernel)
//!
//! Dispositivo essencial do kernel para output de panic e debug.

use crate::fs::devfs::char_device::CharDevice;
use crate::fs::devfs::device::{Device, DeviceNumber, DeviceType};

/// /dev/console device
pub struct ConsoleDevice {
    dev: DeviceNumber,
}

impl ConsoleDevice {
    /// Cria um novo /dev/console
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(5, 1),
        }
    }
}

impl Device for ConsoleDevice {
    fn name(&self) -> &str {
        "console"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implementar leitura do console
        // - Integrar com driver de teclado
        // - Buffer de entrada
        Err("Console read not implemented")
    }

    fn write(&self, buf: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implementar escrita no console
        // - Integrar com driver VGA/framebuffer
        // - Por enquanto, usar serial

        // Stub: imprime via serial (se disponível)
        for &byte in buf {
            // TODO: Chamar serial_write(byte)
            let _ = byte;
        }

        Ok(buf.len())
    }
}

impl CharDevice for ConsoleDevice {}

impl Default for ConsoleDevice {
    fn default() -> Self {
        Self::new()
    }
}
