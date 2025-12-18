//! Character Device - Dispositivos de caractere

use super::device::{Device, DeviceNumber, DeviceType};

/// Trait para dispositivos de caractere
pub trait CharDevice: Device {
    /// Lê um byte do dispositivo
    fn read_byte(&self) -> Result<u8, &'static str> {
        let mut buf = [0u8; 1];
        self.read(&mut buf)?;
        Ok(buf[0])
    }

    /// Escreve um byte no dispositivo
    fn write_byte(&self, byte: u8) -> Result<(), &'static str> {
        let buf = [byte];
        self.write(&buf)?;
        Ok(())
    }
}

/// Dispositivo de caractere base
pub struct BaseCharDevice {
    name: &'static str,
    dev: DeviceNumber,
}

impl BaseCharDevice {
    /// Cria um novo dispositivo de caractere
    pub const fn new(name: &'static str, major: u32, minor: u32) -> Self {
        Self {
            name,
            dev: DeviceNumber::new(major, minor),
        }
    }
}

impl Device for BaseCharDevice {
    fn name(&self) -> &str {
        self.name
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }
}

impl CharDevice for BaseCharDevice {}
