//! # Generic Graphics Adapter Driver (Unified VGA/LFB)
//!
//! Este driver é o cavalo de batalha do RedstoneOS para compatibilidade.
//! Ele orquestra o suporte desde o modo texto VGA até alta resolução UEFI.

pub mod lfb;
pub mod ops;
pub mod vga;

use self::lfb::LfbController;
use self::vga::VgaController;
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct GenericGpuDriver {
    // Aqui no futuro poderemos ter instâncias dos controladores selecionados no boot
}

impl GenericGpuDriver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Driver for GenericGpuDriver {
    fn name(&self) -> &'static str {
        "Redstone Generic Graphics Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        if dev.device_type != DeviceType::Display {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!("(GPU/Generic) Assumindo o controle do dispositivo de display.");

        // 1. Verificar se estamos em UEFI (LFB) ou BIOS Legado (VGA)
        // 2. Desabilitar cursores de hardware legados
        unsafe {
            let vga = VgaController;
            vga.clear_text_mode();
            vga.disable_cursor();
        }

        // 3. Registrar o Framebuffer no subsistema FB global

        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(GenericGpuDriver::new()));
}
