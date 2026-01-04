//! # VirtIO Block Driver
//!
//! Driver para discos paravirtualizados de alta performance.

pub mod request;
pub mod virtqueue;

use super::traits::{BlockDevice, BlockError};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct VirtioBlkDriver;

impl Driver for VirtioBlkDriver {
    fn name(&self) -> &'static str {
        "VirtIO Block Storage"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização VirtIO
        // 1. Negociar features (VIRTIO_BLK_F_*)
        // 2. Configurar virtqueues
        // 3. Status |= DRIVER_OK
        crate::kinfo!("(Storage/VirtIO) Disco virtual detectado.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub struct VirtioDisk;

impl BlockDevice for VirtioDisk {
    fn name(&self) -> &str {
        "vda"
    }

    fn block_size(&self) -> usize {
        512
    }

    fn total_blocks(&self) -> u64 {
        1024 * 1024 // Fake size
    }

    fn read_block(&self, _lba: u64, _buf: &mut [u8]) -> Result<(), BlockError> {
        // TODO: Enviar request para a queue de requests
        Err(BlockError::NotReady)
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        Err(BlockError::NotReady)
    }
}
