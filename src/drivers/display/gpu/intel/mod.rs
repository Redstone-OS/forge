//! # Intel Graphics Driver (i915-style)
//!
//! Driver para GPUs integradas Intel (HD Graphics, Iris, UHD).
//! Responsável pelo Display Engine e Render Engine via MMIO.

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::bus::pci::device::PciDevice;
use alloc::sync::Arc;

pub struct IntelGpuDriver;

impl IntelGpuDriver {
    /// Registros MMIO comuns da Intel
    pub const DE_PIPE_A_CONF: u32 = 0x70008;
    pub const DE_PIPE_A_HORZ: u32 = 0x70004;
    pub const DE_PIPE_A_VERT: u32 = 0x7000C;

    // TODO: Implementar GTT (Graphics Translation Table)
    // TODO: Implementar Ring Buffers para execução de comandos
}

impl Driver for IntelGpuDriver {
    fn name(&self) -> &'static str {
        "Intel Integrated Graphics Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        let pci_info = match dev.get_data::<PciDevice>() {
            Some(info) => info,
            None => return Err(DriverError::NotSupported),
        };

        // Vendor ID Intel = 0x8086
        if pci_info.vendor_id != 0x8086 {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!("(GPU) Intel HD/UHD Graphics detectada.");

        // 1. Mapear MMIO (GTTMMADR no BAR0)
        let mmio_base = pci_info.bar_address(0);
        if mmio_base == 0 {
            return Err(DriverError::HardwareFault);
        }
        crate::kdebug!("  -> MMIO Base:", mmio_base);

        // 2. Mapear GTT e Framebuffer (GMADR no BAR2)
        let fb_base = pci_info.bar_address(2);
        if fb_base == 0 {
            return Err(DriverError::HardwareFault);
        }
        crate::kdebug!("  -> Framebuffer Base:", fb_base);

        // 3. Inicialização de baixo nível (Pipes e Planes)

        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(IntelGpuDriver));
}
