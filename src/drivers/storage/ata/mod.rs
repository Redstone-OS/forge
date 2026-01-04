//! # ATA/IDE Driver
//!
//! Driver para discos PATA/IDE legados usando PIO mode.
//! Importante para hardware antigo e QEMU default.
//!
//! ## Portas I/O:
//! - Primary: 0x1F0-0x1F7, Control: 0x3F6
//! - Secondary: 0x170-0x177, Control: 0x376

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::storage::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

// ATA I/O Ports
// TODO: Revisar no futuro
#[allow(unused)]
const ATA_PRIMARY_BASE: u16 = 0x1F0;
#[allow(unused)]
const ATA_PRIMARY_CTRL: u16 = 0x3F6;
#[allow(unused)]
const ATA_SECONDARY_BASE: u16 = 0x170;
#[allow(unused)]
const ATA_SECONDARY_CTRL: u16 = 0x376;

// ATA Commands
#[allow(unused)]
const ATA_CMD_READ_PIO: u8 = 0x20;
#[allow(unused)]
const ATA_CMD_WRITE_PIO: u8 = 0x30;
#[allow(unused)]
const ATA_CMD_IDENTIFY: u8 = 0xEC;

/// Driver ATA para o RDS.
pub struct AtaDriver;

impl Driver for AtaDriver {
    fn name(&self) -> &'static str {
        "ata"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // ATA é detectado via portas I/O, não PCI tradicional
        // Aceita dispositivos de classe IDE (0x01, 0x01)
        if dev.class_code != 0x01 || dev.subclass_code != 0x01 {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(ATA) Controlador IDE: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // TODO: Detect drives on primary/secondary channels

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(ATA) Driver removido");
        Ok(())
    }
}

// TODO: Revisar no futuro
#[allow(unused)]
struct AtaDiskState {
    enabled: bool,
    stats: StorageStats,
}

/// Dispositivo ATA/IDE.
pub struct AtaDisk {
    channel: u8, // 0 = primary, 1 = secondary
    drive: u8,   // 0 = master, 1 = slave
    state: Spinlock<AtaDiskState>,
}

impl AtaDisk {
    pub fn new(channel: u8, drive: u8) -> Self {
        Self {
            channel,
            drive,
            state: Spinlock::new(AtaDiskState {
                enabled: false,
                stats: StorageStats::default(),
            }),
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    fn base_port(&self) -> u16 {
        if self.channel == 0 {
            ATA_PRIMARY_BASE
        } else {
            ATA_SECONDARY_BASE
        }
    }
}

impl BlockDevice for AtaDisk {
    fn name(&self) -> &str {
        match (self.channel, self.drive) {
            (0, 0) => "hda",
            (0, 1) => "hdb",
            (1, 0) => "hdc",
            (1, 1) => "hdd",
            _ => "hdX",
        }
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            model: alloc::string::String::from("ATA/IDE Disk"),
            device_type: Some(StorageType::Hdd),
            interface: Some(StorageInterface::Ata),
            ..Default::default()
        }
    }

    fn block_size(&self) -> usize {
        512
    }
    fn total_blocks(&self) -> u64 {
        0
    } // TODO: IDENTIFY

    fn read_block(&self, _lba: u64, _buf: &mut [u8]) -> Result<(), BlockError> {
        // TODO: PIO read
        Err(BlockError::NotReady)
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        // TODO: PIO write
        Err(BlockError::NotReady)
    }

    fn get_stats(&self) -> StorageStats {
        self.state.lock().stats
    }
}

/// Registra o driver ATA.
pub fn init() {
    crate::kinfo!("(ATA) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(AtaDriver));
}
