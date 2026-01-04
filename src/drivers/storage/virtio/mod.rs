//! # VirtIO Block Driver
//!
//! Driver para discos paravirtualizados VirtIO (virtio-blk).
//! Alta performance em máquinas virtuais QEMU/KVM.
//!
//! ## Spec: OASIS VirtIO v1.2 - Section 5.2 Block Device

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::storage::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_BLK_DEVICE: u16 = 0x1001; // Legacy
const VIRTIO_BLK_DEVICE_MODERN: u16 = 0x1042; // Modern (0x1040 + 2)

/// Driver VirtIO Block para o RDS.
pub struct VirtioBlkDriver;

impl Driver for VirtioBlkDriver {
    fn name(&self) -> &'static str {
        "virtio-blk"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        let is_virtio_blk = dev.vendor_id == VIRTIO_VENDOR
            && (dev.device_id == VIRTIO_BLK_DEVICE || dev.device_id == VIRTIO_BLK_DEVICE_MODERN);

        if !is_virtio_blk {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(VirtIO-Blk) Disco virtual: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // TODO: Inicialização real:
        // 1. Reset device
        // 2. Negociar features
        // 3. Configurar virtqueue
        // 4. Ler config space (capacity, etc)
        // 5. Set DRIVER_OK

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::drivers::storage::unregister_device("vda");
        Ok(())
    }
}

struct VirtioDiskState {
    enabled: bool,
    capacity: u64,
    stats: StorageStats,
}

/// Dispositivo VirtIO Block.
pub struct VirtioDisk {
    index: u8,
    state: Spinlock<VirtioDiskState>,
}

impl VirtioDisk {
    pub fn new(index: u8, capacity: u64) -> Self {
        Self {
            index,
            state: Spinlock::new(VirtioDiskState {
                enabled: false,
                capacity,
                stats: StorageStats::default(),
            }),
        }
    }
}

impl BlockDevice for VirtioDisk {
    fn name(&self) -> &str {
        match self.index {
            0 => "vda",
            1 => "vdb",
            2 => "vdc",
            _ => "vdX",
        }
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            model: alloc::string::String::from("VirtIO Block Device"),
            device_type: Some(StorageType::Virtual),
            interface: Some(StorageInterface::Virtio),
            ..Default::default()
        }
    }

    fn block_size(&self) -> usize {
        512
    }

    fn total_blocks(&self) -> u64 {
        self.state.lock().capacity
    }

    fn read_block(&self, _lba: u64, _buf: &mut [u8]) -> Result<(), BlockError> {
        // TODO: Submit to virtqueue
        Err(BlockError::NotReady)
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        // TODO: Submit to virtqueue
        Err(BlockError::NotReady)
    }

    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            writable: true,
            flush: true,
            discard: true,
            max_transfer_blocks: 256,
            ..Default::default()
        }
    }

    fn get_stats(&self) -> StorageStats {
        self.state.lock().stats
    }
}

/// Registra o driver VirtIO Block.
pub fn init() {
    crate::kinfo!("(VirtIO-Blk) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(VirtioBlkDriver));
}
