//! Trait base para drivers

use super::device::Device;

/// Tipo de dispositivo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    Block,
    Char,
    Network,
    Input,
    Display,
    Timer,
    Bus,
    Unknown,
}

/// Erro de driver
#[derive(Debug, Clone, Copy)]
pub enum DriverError {
    NotSupported,
    NotFound,
    InitFailed,
    BusError,
    Timeout,
    IoError,
}

/// Trait que todo driver deve implementar
pub trait Driver: Send + Sync {
    /// Nome do driver
    fn name(&self) -> &'static str;
    
    /// Tipo de dispositivo
    fn device_type(&self) -> DeviceType;
    
    /// Chamado quando dispositivo é detectado
    fn probe(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// Chamado quando dispositivo é removido
    fn remove(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// Chamado durante suspend
    fn suspend(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }
    
    /// Chamado durante resume
    fn resume(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }
}
