//! # Bochs Graphics Adapter (BGA) Driver
//!
//! Driver para o controlador de vídeo virtual Bochs/QEMU.
//! Suporta modos de 32 bits com Linear Framebuffer (LFB).

pub mod regs;

use crate::arch::x86_64::ports::{inw, outw};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::bus::pci::device::PciDeviceInfo;
use alloc::sync::Arc;

pub struct BochsDriver;

impl BochsDriver {
    /// Escreve em um registrador VBE Dispi
    fn write_reg(&self, index: u16, data: u16) {
        unsafe {
            outw(regs::INDEX_PORT, index);
            outw(regs::DATA_PORT, data);
        }
    }

    /// Lê de um registrador VBE Dispi
    fn read_reg(&self, index: u16) -> u16 {
        unsafe {
            outw(regs::INDEX_PORT, index);
            inw(regs::DATA_PORT)
        }
    }

    /// Verifica a presença do hardware BGA
    fn check_version(&self) -> bool {
        let id = self.read_reg(regs::VBE_DISPI_INDEX_ID);
        id >= regs::VBE_DISPI_ID0 && id <= regs::VBE_DISPI_ID5
    }

    /// Configura a resolução de vídeo
    pub fn set_mode(&self, width: u16, height: u16, bpp: u16) {
        self.write_reg(regs::VBE_DISPI_INDEX_ENABLE, regs::VBE_DISPI_DISABLED);
        self.write_reg(regs::VBE_DISPI_INDEX_XRES, width);
        self.write_reg(regs::VBE_DISPI_INDEX_YRES, height);
        self.write_reg(regs::VBE_DISPI_INDEX_BPP, bpp);
        self.write_reg(
            regs::VBE_DISPI_INDEX_ENABLE,
            regs::VBE_DISPI_ENABLED | regs::VBE_DISPI_LFB_ENABLED,
        );
    }
}

impl Driver for BochsDriver {
    fn name(&self) -> &'static str {
        "Bochs Graphics Adapter Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Display
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // 1. Verificar se o dispositivo PCI é um Bochs BGA
        let pci_info = match dev.get_data::<PciDeviceInfo>() {
            Some(info) => info,
            None => return Err(DriverError::NotSupported),
        };

        if pci_info.vendor_id != 0x1234 || pci_info.device_id != 0x1111 {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!("(Display) Bochs BGA detectado no barramento PCI.");

        // 2. Verificar hardware via portas I/O
        if !self.check_version() {
            return Err(DriverError::HardwareFault);
        }

        // 3. Obter o Framebuffer do BAR0
        let framebuffer_phys = match pci_info.get_bar_address(0) {
            Some(addr) => addr,
            None => return Err(DriverError::HardwareFault),
        };

        crate::kinfo!("(Display) Framebuffer detectado em:", framebuffer_phys);

        // 4. Configuração inicial (Padrão: 1024x768x32)
        self.set_mode(1024, 768, 32);

        // TODO: Registrar o framebuffer no VideoManager/Display subsystem

        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(BochsDriver));
}
