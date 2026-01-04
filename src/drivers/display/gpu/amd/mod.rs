//! # AMD Radeon Graphics Driver (AMDGPU-style)
//!
//! Driver para GPUs AMD (Radeon, Instinct).
//! Implementa suporte para IP Blocks (GFX, SDMA, VCN) e AtomBIOS/PSP.

// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::bus::pci::device::PciDevice;
use alloc::sync::Arc;

pub struct AmdGpuDriver;

impl Driver for AmdGpuDriver {
    fn name(&self) -> &'static str {
        "AMD Radeon Graphics Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        let pci_info = match dev.get_data::<PciDevice>() {
            Some(info) => info,
            None => return Err(DriverError::NotSupported),
        };

        // Vendor ID AMD/ATI = 0x1002
        if pci_info.vendor_id != 0x1002 {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!("(GPU) AMD Radeon detectada.");

        // 1. BAR0/BAR1: VRAM (Acesso ao Framebuffer grande)
        let vram_base = pci_info.bar_address(0);
        if vram_base == 0 {
            return Err(DriverError::HardwareFault);
        }
        crate::kdebug!("  -> VRAM (BAR0):", vram_base);

        // 2. BAR2: MMIO (Registros de controle)
        let mmio_base = pci_info.bar_address(2);
        if mmio_base == 0 {
            return Err(DriverError::HardwareFault);
        }
        crate::kdebug!("  -> MMIO (BAR2):", mmio_base);

        // 3. Inicialização via AtomBIOS ou PSP (Platform Security Processor)

        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(AmdGpuDriver));
}
