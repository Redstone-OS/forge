//! # PCI Configuration Space
//!
//! Acesso ao espaço de configuração PCI via I/O ports.
//!
//! ## Portas de I/O
//!
//! | Porta   | Descrição           |
//! |---------|---------------------|
//! | 0xCF8   | CONFIG_ADDRESS      |
//! | 0xCFC   | CONFIG_DATA         |
//!
//! ## Formato do Endereço (CONFIG_ADDRESS)
//!
//! ```text
//! 31      23      15      10   7       0
//! ┌───────┬───────┬───────┬────┬───────┐
//! │Enable │ Reserv│  Bus  │Dev │Func│Reg│
//! │  1b   │  7b   │  8b   │ 5b │ 3b │6b │
//! └───────┴───────┴───────┴────┴───────┘
//! ```

use core::arch::asm;

/// Porta de endereço de configuração PCI
const CONFIG_ADDRESS: u16 = 0xCF8;

/// Porta de dados de configuração PCI
const CONFIG_DATA: u16 = 0xCFC;

/// Escreve em uma porta de I/O (32 bits)
#[inline]
unsafe fn outl(port: u16, value: u32) {
    asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Lê de uma porta de I/O (32 bits)
#[inline]
unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Monta o endereço de configuração PCI
///
/// # Argumentos
/// * `bus` - Número do barramento (0-255)
/// * `device` - Número do dispositivo (0-31)
/// * `function` - Número da função (0-7)
/// * `offset` - Offset no espaço de configuração (0-255, alinhado a 4 bytes)
#[inline]
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
