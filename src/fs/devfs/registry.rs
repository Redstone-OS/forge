//! Device Registry - Registro global de dispositivos

use super::device::{Device, DeviceNumber};
use super::operations::OpenFlags;

/// Número máximo de dispositivos
const MAX_DEVICES: usize = 256;

// Entrada no registro de dispositivos
struct DeviceEntry {
    name: [u8; 32],
    name_len: usize,
    device_number: DeviceNumber,
    // Ponteiro para o dispositivo (Box quando implementar alloc)
    // device: Option<Box<dyn Device>>,
}

/// Registro de dispositivos
pub struct DeviceRegistry {
    /// Array de dispositivos registrados
    devices: [Option<DeviceEntry>; MAX_DEVICES],
    /// Contador de dispositivos
    count: usize,
}

impl DeviceRegistry {
    /// Cria um novo registro
    pub const fn new() -> Self {
        const NONE: Option<DeviceEntry> = None;
        Self {
            devices: [NONE; MAX_DEVICES],
            count: 0,
        }
    }

    /// Registra um dispositivo
    pub fn register(
        &mut self,
        _name: &'static str,
        _dev: DeviceNumber,
    ) -> Result<(), &'static str> {
        if self.count >= MAX_DEVICES {
            return Err("Device registry full");
        }

        // TODO: Implementar quando tiver Box/alloc
        // self.devices[self.count] = Some(DeviceEntry { dev, name, device });
        // self.count += 1;

        Ok(())
    }

    /// Remove um dispositivo
    pub fn unregister(&mut self, _dev: DeviceNumber) -> Result<(), &'static str> {
        // TODO: Implementar
        Err("Not implemented")
    }

    /// Busca um dispositivo por nome
    pub fn lookup(&self, _name: &str) -> Option<DeviceNumber> {
        // TODO: Implementar
        None
    }

    /// Busca um dispositivo por device number
    pub fn lookup_by_dev(&self, _dev: DeviceNumber) -> Option<&'static str> {
        // TODO: Implementar
        None
    }

    /// Abre um dispositivo
    pub fn open(&self, _path: &str, _flags: OpenFlags) -> Result<usize, &'static str> {
        // TODO: Implementar
        Err("Not implemented")
    }

    /// Fecha um dispositivo
    pub fn close(&self, _fd: usize) -> Result<(), &'static str> {
        // TODO: Implementar
        Err("Not implemented")
    }

    /// Lê de um dispositivo
    pub fn read(&self, _fd: usize, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implementar
        Err("Not implemented")
    }

    /// Escreve em um dispositivo
    pub fn write(&self, _fd: usize, _buf: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implementar
        Err("Not implemented")
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
