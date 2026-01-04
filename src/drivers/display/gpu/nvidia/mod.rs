//! # NVIDIA Graphics Driver (Nouveau-style)
//!
//! Driver para GPUs NVIDIA (GeForce, Quadro, Tesla).
//! Focado em gerenciamento de PFIFO, PGRAPH e buffers de comando.

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::bus::pci::device::PciDeviceInfo;
use alloc::sync::Arc;

pub struct NvidiaGpuDriver;

impl NvidiaGpuDriver {
    /// Registros clássicos da NVIDIA
    pub const PMC_BOOT_0: u32 = 0x000000; // ID do chip
    pub const PFIFO_CACHES: u32 = 0x002500;
}

impl Driver for NvidiaGpuDriver {
    fn name(&self) -> &'static str {
        "NVIDIA GeForce/Quadro Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        let pci_info = match dev.get_data::<PciDeviceInfo>() {
            Some(info) => info,
            None => return Err(DriverError::NotSupported),
        };

        // Vendor ID NVIDIA = 0x10DE
        if pci_info.vendor_id != 0x10DE {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!("(GPU) NVIDIA Graphics detectada.");

        // 1. BAR0: MMIO (Registros de controle)
        let mmio_base = pci_info
            .get_bar_address(0)
            .ok_or(DriverError::HardwareFault)?;
        crate::kdebug!("  -> MMIO (BAR0):", mmio_base);

        // 2. BAR1/BAR3: VRAM (Acesso ao Framebuffer)
        let vram_base = pci_info
            .get_bar_address(1)
            .ok_or(DriverError::HardwareFault)?;
        crate::kdebug!("  -> VRAM (BAR1):", vram_base);

        // 3. Handshake com o GSP (em GPUs modernas como Turing/Ampere)

        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(NvidiaGpuDriver));
}
