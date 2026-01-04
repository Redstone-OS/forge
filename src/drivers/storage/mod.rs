//! # Storage Drivers Subsystem
//!
//! Gerencia drivers de armazenamento em massa (Block Devices).
//! Inclui suporte para tecnologias legadas (ATA) e modernas (NVMe, AHCI, VirtIO).

pub mod ahci;
pub mod ata; // Legacy PATA/IDE
pub mod nvme; // Modern NVMe SSDs
pub mod ramdisk;
pub mod traits;
pub mod virtio; // VirtIO-Blk

use crate::drivers::base::driver::Driver;
use alloc::sync::Arc;

/// Inicializa e registra todos os drivers de armazenamento.
pub fn init() {
    crate::kinfo!("(Storage) Inicializando drivers de disco...");

    // 1. Ramdisk (Disco em memória)
    // Cria um disco de 32MB para testes
    let rd_driver = ramdisk::RamdiskDriver::new(32);
    crate::drivers::base::register_driver(Arc::new(rd_driver) as Arc<dyn Driver>);

    // 2. ATA Legacy (PIO)
    // Importante para QEMU default e hardware antigo
    crate::drivers::base::register_driver(Arc::new(ata::AtaDriver) as Arc<dyn Driver>);

    // 3. AHCI (SATA)
    crate::drivers::base::register_driver(Arc::new(ahci::AhciDriver) as Arc<dyn Driver>);

    // 4. NVMe (PCIe SSD)
    crate::drivers::base::register_driver(Arc::new(nvme::NvmeDriver) as Arc<dyn Driver>);

    // 5. VirtIO Block (Virtualização)
    crate::drivers::base::register_driver(Arc::new(virtio::VirtioBlkDriver) as Arc<dyn Driver>);

    crate::kinfo!("(Storage) Drivers registrados no RDM.");
}
