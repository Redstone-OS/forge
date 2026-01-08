//! # AHCI / SATA Driver
//!
//! Driver para controladores SATA via interface AHCI (Advanced Host Controller Interface).
//! Padrão moderno para discos SATA em PCs desde ~2004.
//!
//! ## Spec: AHCI 1.3.1
//!
//! ## Arquitetura AHCI:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           AHCI Controller               │
//! │  ┌─────────────────────────────────┐    │
//! │  │  Generic Host Control (GHC)     │    │
//! │  │  Port Registers (0-31)          │    │
//! │  └─────────────────────────────────┘    │
//! │                  │                      │
//! │           SATA Link                     │
//! │                  │                      │
//! │    ┌─────────┬───┴───┬─────────┐        │
//! │    ▼         ▼       ▼         ▼        │
//! │  Port 0    Port 1  Port 2   Port 3      │
//! │  (HDD)     (SSD)   (empty)  (DVD)       │
//! └─────────────────────────────────────────┘
//! ```

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::storage::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

const PCI_CLASS_MASS_STORAGE: u8 = 0x01;
const PCI_SUBCLASS_SATA: u8 = 0x06;
// TODO: Revisar no futuro
#[allow(unused)]
const PCI_PROGIF_AHCI: u8 = 0x01;

/// Driver AHCI para o RDS.
pub struct AhciDriver;

impl Driver for AhciDriver {
    fn name(&self) -> &'static str {
        "ahci"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // AHCI: Class 0x01, Subclass 0x06, ProgIF 0x01
        if dev.class_code != PCI_CLASS_MASS_STORAGE || dev.subclass_code != PCI_SUBCLASS_SATA {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(AHCI) Controlador SATA: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // TODO: Inicialização real:
        // 1. Ler ABAR de BAR5
        // 2. Habilitar AHCI mode
        // 3. Enumerar portas
        // 4. Identificar dispositivos conectados

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(AHCI) Driver removido");
        Ok(())
    }
}

// TODO: Revisar no futuro
#[allow(unused)]
struct AhciDiskState {
    enabled: bool,
    stats: StorageStats,
}

/// Dispositivo SATA via AHCI.
pub struct AhciDisk {
    port: u8,
    state: Spinlock<AhciDiskState>,
}

impl AhciDisk {
    pub fn new(port: u8) -> Self {
        Self {
            port,
            state: Spinlock::new(AhciDiskState {
                enabled: false,
                stats: StorageStats::default(),
            }),
        }
    }
}

impl BlockDevice for AhciDisk {
    fn name(&self) -> &str {
        match self.port {
            0 => "sda",
            1 => "sdb",
            2 => "sdc",
            3 => "sdd",
            _ => "sdX",
        }
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            model: alloc::string::String::from("AHCI SATA Disk"),
            device_type: Some(StorageType::Hdd),
            interface: Some(StorageInterface::Ahci),
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
        // TODO: Enviar FIS H2D com READ DMA EXT
        Err(BlockError::NotReady)
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        // TODO: Enviar FIS H2D com WRITE DMA EXT
        Err(BlockError::NotReady)
    }

    fn get_stats(&self) -> StorageStats {
        self.state.lock().stats
    }
}

/// Registra o driver AHCI.
pub fn init() {
    // crate::kinfo!("(AHCI) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(AhciDriver));
}
