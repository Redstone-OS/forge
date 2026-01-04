//! # NVMe Driver
//!
//! Driver para SSDs Non-Volatile Memory Express sobre PCIe.
//!
//! O NVMe é o padrão moderno para armazenamento de alto desempenho, substituindo o AHCI.
//! Ele utiliza filas de submissão e conclusão (SQ/CQ) em memória compartilhada.

use super::traits::{BlockDevice, BlockError};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct NvmeDriver;

impl Driver for NvmeDriver {
    fn name(&self) -> &'static str {
        "NVMe SSD Controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização NVMe
        // 1. Mapear BAR0 (Registradores 64-bit)
        // 2. Desabilitar controller (CC.EN = 0)
        // 3. Configurar Admin Queue (ASQ/ACQ)
        // 4. Habilitar controller (CC.EN = 1)
        // 5. Identificar Controller e Namespaces
        crate::kinfo!("(Storage/NVMe) Controlador NVMe detectado (stub).");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub struct NvmeNamespace {
    ns_id: u32,
    blocks: u64,
}

impl BlockDevice for NvmeNamespace {
    fn name(&self) -> &str {
        "nvme0n1"
    }

    fn block_size(&self) -> usize {
        4096 // NVMe geralmente usa 4K
    }

    fn total_blocks(&self) -> u64 {
        self.blocks
    }

    fn read_block(&self, _lba: u64, _buf: &mut [u8]) -> Result<(), BlockError> {
        // TODO: Enviar comando Read NVM para a I/O Submission Queue
        Err(BlockError::NotReady)
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        Err(BlockError::NotReady)
    }
}
