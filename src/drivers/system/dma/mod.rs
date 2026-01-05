//! # Legacy DMA Controller (8237)
//!
//! Driver para o controlador DMA legado usado por dispositivos ISA.
//! Usado por: Floppy, Sound Blaster, parallel port.
//!
//! ## Limitações:
//! - Endereços apenas nos primeiros 16MB
//! - Não pode cruzar limite de 64KB
//! - 4 canais por controlador (8 total, mas canal 4 é cascade)

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::system::traits::*;
use alloc::sync::Arc;

/// Portas do DMA Controller.
#[allow(dead_code)]
mod ports {
    // Master DMA (canais 0-3)
    pub const DMA1_CMD: u16 = 0x08;
    pub const DMA1_REQ: u16 = 0x09;
    pub const DMA1_MASK: u16 = 0x0A;
    pub const DMA1_MODE: u16 = 0x0B;
    pub const DMA1_CLEAR_FF: u16 = 0x0C;
    pub const DMA1_RESET: u16 = 0x0D;
    pub const DMA1_MASK_ALL: u16 = 0x0F;

    // Slave DMA (canais 4-7)
    pub const DMA2_CMD: u16 = 0xD0;
    pub const DMA2_REQ: u16 = 0xD2;
    pub const DMA2_MASK: u16 = 0xD4;
    pub const DMA2_MODE: u16 = 0xD6;
    pub const DMA2_CLEAR_FF: u16 = 0xD8;
    pub const DMA2_RESET: u16 = 0xDA;
    pub const DMA2_MASK_ALL: u16 = 0xDE;

    // Page registers
    pub const DMA_PAGE: [u16; 8] = [0x87, 0x83, 0x81, 0x82, 0x8F, 0x8B, 0x89, 0x8A];

    // Address registers
    pub const DMA_ADDR: [u16; 8] = [0x00, 0x02, 0x04, 0x06, 0xC0, 0xC4, 0xC8, 0xCC];

    // Count registers
    pub const DMA_COUNT: [u16; 8] = [0x01, 0x03, 0x05, 0x07, 0xC2, 0xC6, 0xCA, 0xCE];
}

/// Driver DMA legado.
pub struct DmaDriver;

impl Driver for DmaDriver {
    fn name(&self) -> &'static str {
        "dma-legacy"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // DMA Legacy só deve parear com o dispositivo 8237 DMA platform
        if dev.device_type != DeviceType::System {
            return Err(DriverError::NotSupported);
        }

        // Verifica se é o dispositivo DMA correto pelo nome
        if !dev.name_as_str().contains("8237") && !dev.name_as_str().contains("DMA") {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!("(DMA) Inicializando controlador legado...");

        // Reset ambos os controladores
        reset_controller(true); // Master
        reset_controller(false); // Slave

        // Mascarar todos os canais inicialmente
        mask_all();

        crate::kinfo!("(DMA) Controlador inicializado");
        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }
}

/// Inicializa o subsistema DMA.
pub fn init() {
    crate::kinfo!("(DMA) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(DmaDriver) as Arc<dyn Driver>);
}

/// Reseta um controlador DMA.
fn reset_controller(master: bool) {
    use crate::arch::x86_64::ports::outb;

    let port = if master {
        ports::DMA1_RESET
    } else {
        ports::DMA2_RESET
    };
    outb(port, 0);
}

/// Mascara todos os canais DMA.
pub fn mask_all() {
    use crate::arch::x86_64::ports::outb;

    outb(ports::DMA1_MASK_ALL, 0x0F); // Mask channels 0-3
    outb(ports::DMA2_MASK_ALL, 0x0F); // Mask channels 4-7
}

/// Mascara um canal específico.
pub fn mask_channel(channel: DmaChannel) {
    use crate::arch::x86_64::ports::outb;

    let ch = channel as u8;
    let (port, bit) = if ch < 4 {
        (ports::DMA1_MASK, ch)
    } else {
        (ports::DMA2_MASK, ch - 4)
    };

    outb(port, 0x04 | bit); // Bit 2 = set mask
}

/// Desmascara um canal específico.
pub fn unmask_channel(channel: DmaChannel) {
    use crate::arch::x86_64::ports::outb;

    let ch = channel as u8;
    let (port, bit) = if ch < 4 {
        (ports::DMA1_MASK, ch)
    } else {
        (ports::DMA2_MASK, ch - 4)
    };

    outb(port, bit); // Bit 2 = 0 = clear mask
}

/// Configura um canal DMA para transferência.
///
/// ## Parâmetros:
/// - `channel`: Canal a configurar
/// - `address`: Endereço físico (deve ser < 16MB)
/// - `count`: Número de bytes - 1
/// - `mode`: Leitura ou escrita
pub fn setup_transfer(
    channel: DmaChannel,
    address: u32,
    count: u16,
    mode: DmaMode,
) -> Result<(), DmaError> {
    use crate::arch::x86_64::ports::outb;

    let ch = channel as u8;

    // Verificar limite de 16MB
    if address >= 0x1000000 {
        return Err(DmaError::AddressTooHigh);
    }

    // Verificar se não cruza limite de 64KB
    let page_start = address >> 16;
    let page_end = (address + count as u32) >> 16;
    if page_start != page_end {
        return Err(DmaError::BoundaryCross);
    }

    // Canal 4 é cascade, não usar
    if ch == 4 {
        return Err(DmaError::InvalidChannel);
    }

    // Mascarar canal durante configuração
    mask_channel(channel);

    // Limpar flip-flop
    let ff_port = if ch < 4 {
        ports::DMA1_CLEAR_FF
    } else {
        ports::DMA2_CLEAR_FF
    };
    outb(ff_port, 0);

    // Configurar modo
    let mode_byte = match mode {
        DmaMode::Read => 0x44 | ch,   // Single, addr inc, read
        DmaMode::Write => 0x48 | ch,  // Single, addr inc, write
        DmaMode::Verify => 0x40 | ch, // Single, addr inc, verify
    };
    let mode_port = if ch < 4 {
        ports::DMA1_MODE
    } else {
        ports::DMA2_MODE
    };
    outb(mode_port, mode_byte);

    // Configurar endereço (low/high)
    let addr_port = ports::DMA_ADDR[ch as usize];
    outb(addr_port, (address & 0xFF) as u8);
    outb(addr_port, ((address >> 8) & 0xFF) as u8);

    // Configurar page register
    let page_port = ports::DMA_PAGE[ch as usize];
    outb(page_port, ((address >> 16) & 0xFF) as u8);

    // Configurar count (count - 1)
    let count_port = ports::DMA_COUNT[ch as usize];
    outb(count_port, (count & 0xFF) as u8);
    outb(count_port, ((count >> 8) & 0xFF) as u8);

    // Desmascarar canal
    unmask_channel(channel);

    Ok(())
}
