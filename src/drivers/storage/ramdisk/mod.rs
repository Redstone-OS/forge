//! # Ramdisk Driver
//!
//! Driver de disco em memória volátil.

use super::traits::{BlockDevice, BlockError};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Tamanho padrão do bloco
const BLOCK_SIZE: usize = 512;

pub struct RamdiskDriver {
    disk: Arc<RamdiskDevice>,
}

impl RamdiskDriver {
    pub fn new(size_mb: usize) -> Self {
        Self {
            disk: Arc::new(RamdiskDevice::new(size_mb)),
        }
    }
}

impl Driver for RamdiskDriver {
    fn name(&self) -> &'static str {
        "Ramdisk Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // Ramdisk é virtual, não precisa de hardware probe real
        // Mas registramos sua existência
        crate::kinfo!("(Storage/Ramdisk) Disco em memória ativo.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

/// O dispositivo de bloco real
pub struct RamdiskDevice {
    data: Spinlock<Vec<u8>>,
    size: usize,
}

impl RamdiskDevice {
    pub fn new(size_mb: usize) -> Self {
        let size_bytes = size_mb * 1024 * 1024;
        let mut vec = Vec::with_capacity(size_bytes);
        vec.resize(size_bytes, 0);

        Self {
            data: Spinlock::new(vec),
            size: size_bytes,
        }
    }
}

impl BlockDevice for RamdiskDevice {
    fn name(&self) -> &str {
        "ramdisk0"
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn total_blocks(&self) -> u64 {
        (self.size / BLOCK_SIZE) as u64
    }

    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        let offset = (lba as usize) * BLOCK_SIZE;
        if offset + buf.len() > self.size {
            return Err(BlockError::InvalidBlock);
        }

        let data = self.data.lock();
        buf.copy_from_slice(&data[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError> {
        let offset = (lba as usize) * BLOCK_SIZE;
        if offset + buf.len() > self.size {
            return Err(BlockError::InvalidBlock);
        }

        let mut data = self.data.lock();
        data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}
