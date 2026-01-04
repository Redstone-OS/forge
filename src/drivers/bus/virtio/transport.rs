//! # VirtIO Transport Abstraction
//!
//! Define como o sistema fala com o hardware VirtIO,
//! independente se é via PCI ou MMIO.

use super::types::VirtioDeviceId;

pub trait VirtioTransport: Send + Sync {
    /// Obtém o ID do dispositivo VirtIO (ex: 2 para Block)
    fn device_id(&self) -> VirtioDeviceId;

    /// Lê o status atual do dispositivo
    fn read_status(&self) -> u8;

    /// Escreve um novo status no dispositivo
    fn write_status(&self, status: u8);

    /// Lê as features suportadas pelo dispositivo
    fn read_device_features(&self) -> u64;

    /// Escreve as features aceitas pelo driver
    fn write_driver_features(&self, features: u64);

    /// Configura uma Virtqueue no hardware
    fn setup_queue(
        &self,
        queue_index: u16,
        size: u16,
        desc_addr: u64,
        avail_addr: u64,
        used_addr: u64,
    );

    /// Notifica o hardware que há novos dados na fila
    fn notify_queue(&self, queue_index: u16);

    /// Lê do espaço de configuração específico do dispositivo (Device Specific Config)
    fn read_config(&self, offset: usize, size: usize) -> u64;
}

#[derive(Debug)]
pub enum VirtioError {
    HardwareNotResponded,
    FeatureNegotiationFailed,
    InvalidDevice,
    NoMemory,
}
