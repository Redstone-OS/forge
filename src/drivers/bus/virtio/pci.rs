//! # VirtIO PCI Transport
//!
//! Implementação da especificação VirtIO sobre o barramento PCI.
//! Suporta o layout moderno (V1) e prepara retrocompatibilidade.

use super::transport::{VirtioError, VirtioTransport};
use super::types::VirtioDeviceId;
use crate::drivers::bus::pci::access;
use crate::drivers::bus::pci::device::PciDeviceInfo;

/// Layout da estrutura Virtio PCI Modern (V1)
pub struct VirtioPciTransport {
    pub pci: PciDeviceInfo,
    pub common_cfg: u64,
    pub notify_base: u64,
    pub notify_off_multiplier: u32,
    pub isr_cfg: u64,
    pub device_cfg: u64,
}

impl VirtioPciTransport {
    pub fn new(pci: PciDeviceInfo) -> Option<Self> {
        // 1. Verificar capabilities PCI para encontrar as estruturas VirtIO
        // TODO: Iterar sobre PCI Capabilities para mapear MMIO config
        // Por enquanto, placeholder funcional

        Some(Self {
            pci,
            common_cfg: 0,
            notify_base: 0,
            notify_off_multiplier: 0,
            isr_cfg: 0,
            device_cfg: 0,
        })
    }
}

impl VirtioTransport for VirtioPciTransport {
    fn device_id(&self) -> VirtioDeviceId {
        // No PCI, o Subsystem Device ID do VirtIO indica a função
        // TODO: Ler do config space
        VirtioDeviceId::BlockDevice
    }

    fn read_status(&self) -> u8 {
        0
    }
    fn write_status(&self, _status: u8) {}
    fn read_device_features(&self) -> u64 {
        0
    }
    fn write_driver_features(&self, _features: u64) {}

    fn setup_queue(&self, _idx: u16, _size: u16, _desc: u64, _avail: u64, _used: u64) {
        // Escrita via MMIO nos registros do common_cfg
    }

    fn notify_queue(&self, _idx: u16) {
        // Escrita via MMIO no notify_base
    }

    fn read_config(&self, _offset: usize, _size: usize) -> u64 {
        0
    }
}
