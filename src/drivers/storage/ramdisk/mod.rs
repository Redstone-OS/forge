//! # Ramdisk Driver
//!
//! Driver de disco em memória volátil. Útil para testes e sistemas
//! que precisam de armazenamento temporário rápido.
//!
//! ## Características:
//! - Dados em RAM (volátil)
//! - Performance máxima (sem I/O real)
//! - Tamanho configurável
//! - Útil para boot sem disco físico

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::storage::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

const BLOCK_SIZE: usize = 512;
const DEFAULT_SIZE_MB: usize = 32;

/// Driver Ramdisk para o RDS.
pub struct RamdiskDriver {
    _size_mb: usize,
}

impl RamdiskDriver {
    pub fn new(size_mb: usize) -> Self {
        Self { _size_mb: size_mb }
    }
}

impl Default for RamdiskDriver {
    fn default() -> Self {
        Self::new(DEFAULT_SIZE_MB)
    }
}

impl Driver for RamdiskDriver {
    fn name(&self) -> &'static str {
        "ramdisk"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // Ramdisk não é descoberto via hardware - ele é virtual
        // A criação do dispositivo é feita em init(), não via probe()
        Err(DriverError::NotSupported)
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::drivers::storage::unregister_device("ram0");
        Ok(())
    }
}

struct RamdiskState {
    data: Vec<u8>,
    stats: StorageStats,
}

/// Dispositivo Ramdisk.
pub struct RamdiskDevice {
    size: usize,
    state: Spinlock<RamdiskState>,
}

impl RamdiskDevice {
    pub fn new(size_mb: usize) -> Self {
        let size = size_mb * 1024 * 1024;
        let mut data = Vec::with_capacity(size);
        data.resize(size, 0);

        Self {
            size,
            state: Spinlock::new(RamdiskState {
                data,
                stats: StorageStats::default(),
            }),
        }
    }
}

impl BlockDevice for RamdiskDevice {
    fn name(&self) -> &str {
        "ram0"
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            model: alloc::string::String::from("RedstoneOS Ramdisk"),
            serial: alloc::string::String::from("RAM00001"),
            firmware: alloc::string::String::from("1.0"),
            device_type: Some(StorageType::Virtual),
            interface: Some(StorageInterface::Ramdisk),
        }
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
            return Err(BlockError::InvalidLba);
        }

        let mut state = self.state.lock();
        buf.copy_from_slice(&state.data[offset..offset + buf.len()]);
        state.stats.blocks_read += 1;
        state.stats.bytes_read += buf.len() as u64;
        Ok(())
    }

    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError> {
        let offset = (lba as usize) * BLOCK_SIZE;
        if offset + buf.len() > self.size {
            return Err(BlockError::InvalidLba);
        }

        let mut state = self.state.lock();
        state.data[offset..offset + buf.len()].copy_from_slice(buf);
        state.stats.blocks_written += 1;
        state.stats.bytes_written += buf.len() as u64;
        Ok(())
    }

    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            writable: true,
            flush: true,
            max_transfer_blocks: 256,
            ..Default::default()
        }
    }

    fn get_stats(&self) -> StorageStats {
        self.state.lock().stats
    }

    fn reset_stats(&self) {
        self.state.lock().stats = StorageStats::default();
    }
}

/// Registra o driver Ramdisk.
///
/// **Nota**: O Ramdisk é um dispositivo virtual, não descoberto via hardware.
/// Por isso o driver retorna NotSupported em probe() e nós criamos o device aqui.
pub fn init() {
    // crate::kinfo!("(Ramdisk) Registrando driver...");
    let driver = RamdiskDriver::new(DEFAULT_SIZE_MB);
    crate::drivers::base::register_driver(Arc::new(driver));

    // Cria o dispositivo ramdisk manualmente (não via probe)
    // Comentado por enquanto - descomentar se precisar de ramdisk
    // crate::kinfo!("(Ramdisk) Criando disco de {}MB", DEFAULT_SIZE_MB);
    // let device = RamdiskDevice::new(DEFAULT_SIZE_MB);
    // crate::drivers::storage::register_device(Arc::new(device));
}
