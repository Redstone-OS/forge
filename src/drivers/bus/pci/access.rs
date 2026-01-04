//! # PCI Access Layer
//!
//! Fornece acesso ao espaço de configuração PCI através de IO Ports (Mecanismo #1).
//! Em arquiteturas modernas, este mecanismo pode ser substituído por ECAM (MMIO).

use crate::arch::x86_64::ports::{inl, outl};

/// Porta de endereço de configuração PCI
const CONFIG_ADDRESS: u16 = 0xCF8;
/// Porta de dados de configuração PCI
const CONFIG_DATA: u16 = 0xCFC;

/// Monta o endereço de configuração PCI
fn pci_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let bus = bus as u32;
    let device = device as u32;
    let function = function as u32;
    let offset = offset as u32;

    // Enable bit (31) + Bus (23:16) + Device (15:11) + Function (10:8) + Offset (7:0)
    0x8000_0000 | (bus << 16) | (device << 11) | (function << 8) | (offset & 0xFC)
}

/// Lê um registro de configuração PCI (32 bits)
pub fn read_config(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = pci_address(bus, device, function, offset);
    unsafe {
        outl(CONFIG_ADDRESS, address);
        inl(CONFIG_DATA)
    }
}

/// Lê um registro de configuração PCI (16 bits)
pub fn read_config_word(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = read_config(bus, device, function, offset);
    ((value >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

/// Lê um registro de configuração PCI (8 bits)
pub fn read_config_byte(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let value = read_config(bus, device, function, offset);
    ((value >> ((offset & 3) * 8)) & 0xFF) as u8
}

/// Escreve um registro de configuração PCI (32 bits)
pub fn write_config(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = pci_address(bus, device, function, offset);
    unsafe {
        outl(CONFIG_ADDRESS, address);
        outl(CONFIG_DATA, value);
    }
}

/// Escreve um registro de configuração PCI (16 bits)
pub fn write_config_word(bus: u8, device: u8, function: u8, offset: u8, value: u16) {
    let current = read_config(bus, device, function, offset);
    let shift = (offset & 2) * 8;
    let mask = 0xFFFF << shift;
    let new_value = (current & !mask) | ((value as u32) << shift);
    write_config(bus, device, function, offset, new_value);
}
