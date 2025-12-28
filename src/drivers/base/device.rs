//! Abstração de dispositivo

use super::driver::DeviceType;

/// ID de dispositivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(pub u64);

/// Representação de um dispositivo
pub struct Device {
    pub id: DeviceId,
    pub name: [u8; 32],
    pub device_type: DeviceType,
    pub bus_type: BusType,
    pub driver: Option<&'static dyn super::driver::Driver>,
}

/// Tipo de barramento
#[derive(Debug, Clone, Copy)]
pub enum BusType {
    Platform,   // Dispositivos integrados
    Pci,
    Usb,
    Acpi,
}

impl Device {
    pub fn new(id: DeviceId, name: &str) -> Self {
        let mut name_buf = [0u8; 32];
        let len = name.len().min(31);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        
        Self {
            id,
            name: name_buf,
            device_type: DeviceType::Unknown,
            bus_type: BusType::Platform,
            driver: None,
        }
    }
}
