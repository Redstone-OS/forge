//! # Device Buffer
//!
//! Buffer genérico para dispositivos.

/// Buffer de dispositivo
pub struct DeviceBuffer {
    pub virt: u64,
    pub phys: u64,
    pub size: usize,
}
