//! # AHCI / SATA Driver
//!
//! Driver para controladores Serial ATA modernos via interface AHCI.

pub mod regs;
pub mod structs;

use super::traits::{BlockDevice, BlockError};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct AhciDriver;

impl Driver for AhciDriver {
    fn name(&self) -> &'static str {
        "AHCI SATA Controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização AHCI
        // 1. Obter endereço ABAR do PCI BAR5
        // 2. Habilitar AHCI Mode (GHC.AE)
        // 3. Enumerar portas implementadas (PI)
        crate::kinfo!("(Storage/AHCI) Controlador SATA detectado.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub struct AhciDisk {
    port_index: u8,
}

impl BlockDevice for AhciDisk {
    fn name(&self) -> &str {
        "sda"
    }

    fn block_size(&self) -> usize {
        512
    }

    fn total_blocks(&self) -> u64 {
        0 // TODO: Ler de IDENTIFY
    }

    fn read_block(&self, _lba: u64, _buf: &mut [u8]) -> Result<(), BlockError> {
        // TODO: Enviar Command Header via DMA ring
        Err(BlockError::NotReady)
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        Err(BlockError::ReadOnly)
    }
}
