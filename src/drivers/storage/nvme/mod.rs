//! # NVMe Driver
//!
//! Driver para SSDs NVMe (Non-Volatile Memory Express) via PCIe.
//! Padrão moderno de alta performance para SSDs.
//!
//! ## Spec: NVMe 1.4
//!
//! ## Arquitetura NVMe:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           NVMe Controller               │
//! │  ┌─────────────────────────────────┐    │
//! │  │  Admin Queue (Submission/Compl) │    │
//! │  │  I/O Queues (1-N pairs)         │    │
//! │  │  Namespaces (1-M)               │    │
//! │  └─────────────────────────────────┘    │
//! │                  │                      │
//! │              PCIe Bus                   │
//! └─────────────────────────────────────────┘
//! ```

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::storage::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

const PCI_CLASS_MASS_STORAGE: u8 = 0x01;
const PCI_SUBCLASS_NVME: u8 = 0x08;

/// Driver NVMe para o RDS.
pub struct NvmeDriver;

impl Driver for NvmeDriver {
    fn name(&self) -> &'static str {
        "nvme"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        if dev.class_code != PCI_CLASS_MASS_STORAGE || dev.subclass_code != PCI_SUBCLASS_NVME {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(NVMe) Controller: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // TODO: Inicialização real:
        // 1. Mapear BAR0 (NVMe registers)
        // 2. Resetar controller (CC.EN = 0)
        // 3. Configurar Admin Queue
        // 4. Habilitar controller (CC.EN = 1)
        // 5. Identify Controller
        // 6. Criar I/O Queues
        // 7. Identify Namespaces

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(NVMe) Driver removido");
        Ok(())
    }
}

struct NvmeDiskState {
    enabled: bool,
    stats: StorageStats,
}

/// Dispositivo NVMe (Namespace).
pub struct NvmeDisk {
    controller: u8,
    namespace: u32,
    state: Spinlock<NvmeDiskState>,
}

impl NvmeDisk {
    pub fn new(controller: u8, namespace: u32) -> Self {
        Self {
            controller,
            namespace,
            state: Spinlock::new(NvmeDiskState {
                enabled: false,
                stats: StorageStats::default(),
            }),
        }
    }
}

impl BlockDevice for NvmeDisk {
    fn name(&self) -> &str {
        "nvme0n1"
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            model: alloc::string::String::from("NVMe SSD"),
            device_type: Some(StorageType::Nvme),
            interface: Some(StorageInterface::Nvme),
            ..Default::default()
        }
    }

    fn block_size(&self) -> usize {
        512
    } // LBA format dependent
    fn total_blocks(&self) -> u64 {
        0
    } // TODO: Identify Namespace

    fn read_block(&self, _lba: u64, _buf: &mut [u8]) -> Result<(), BlockError> {
        // TODO: Submit Read command to I/O Queue
        Err(BlockError::NotReady)
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        // TODO: Submit Write command to I/O Queue
        Err(BlockError::NotReady)
    }

    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            writable: true,
            flush: true,
            discard: true,
            async_io: true,
            max_transfer_blocks: 65536,
            io_queues: 4,
            ..Default::default()
        }
    }

    fn get_stats(&self) -> StorageStats {
        self.state.lock().stats
    }
}

/// Registra o driver NVMe.
pub fn init() {
    crate::kinfo!("(NVMe) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(NvmeDriver));
}
