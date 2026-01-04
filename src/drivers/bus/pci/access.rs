//! # Acesso ao Espaço de Configuração PCI
//!
//! Este arquivo implementa as funções de leitura/escrita para o
//! **espaço de configuração** dos dispositivos PCI.
//!
//! ## Métodos de Acesso:
//!
//! ### Portas Legadas (PCI 2.x):
//! - ADDRESS: 0xCF8 (32 bits)
//! - DATA: 0xCFC (32 bits)
//! - Suporta até 256 bytes por device
//!
//! ### ECAM (PCIe):
//! - Memory-Mapped Configuration Space
//! - 4KB por device (função)
//! - Detectado via tabela ACPI MCFG
//!
//! ## Formato do Endereço (porta 0xCF8):
//! ```text
//! 31     24 23    16 15    11 10     8 7      2 1 0
//! +---------+--------+--------+--------+--------+---+
//! | Enable  |  Bus   | Device |Function| Offset | 0 |
//! +---------+--------+--------+--------+--------+---+
//! ```

use super::{PciAddress, PCI_CONFIG_ADDRESS, PCI_CONFIG_DATA};
use super::{ECAM_AVAILABLE, ECAM_BASE};
use core::arch::asm;

// =============================================================================
// ACESSO VIA PORTAS I/O (LEGACY)
// =============================================================================

/// Lê um valor de 32 bits do espaço de configuração.
///
/// ## Parâmetros:
/// - `addr`: Endereço PCI (bus/device/function)
/// - `offset`: Offset no espaço de config (0-255, alinhado a 4)
///
/// ## Retorno:
/// Valor de 32 bits lido
pub fn read_config(addr: PciAddress, offset: u8) -> u32 {
    // Verifica se ECAM está disponível
    if *ECAM_AVAILABLE.lock() {
        return read_config_ecam(addr, offset);
    }

    read_config_legacy(addr, offset)
}

/// Escreve um valor de 32 bits no espaço de configuração.
///
/// ## Parâmetros:
/// - `addr`: Endereço PCI (bus/device/function)
/// - `offset`: Offset no espaço de config (0-255, alinhado a 4)
/// - `value`: Valor a escrever
pub fn write_config(addr: PciAddress, offset: u8, value: u32) {
    if *ECAM_AVAILABLE.lock() {
        write_config_ecam(addr, offset, value);
        return;
    }

    write_config_legacy(addr, offset, value);
}

// =============================================================================
// IMPLEMENTAÇÃO LEGACY (PORTAS 0xCF8/0xCFC)
// =============================================================================

/// Monta o endereço para a porta CONFIG_ADDRESS.
#[inline]
fn make_address(addr: PciAddress, offset: u8) -> u32 {
    // Bit 31: Enable
    // Bits 23-16: Bus
    // Bits 15-11: Device
    // Bits 10-8: Function
    // Bits 7-2: Register (offset >> 2)
    // Bits 1-0: 00 (alinhamento)
    0x8000_0000
        | ((addr.bus as u32) << 16)
        | ((addr.device as u32) << 11)
        | ((addr.function as u32) << 8)
        | ((offset as u32) & 0xFC)
}

/// Lê via portas legadas.
fn read_config_legacy(addr: PciAddress, offset: u8) -> u32 {
    let address = make_address(addr, offset);

    unsafe {
        // Escreve endereço
        outl(PCI_CONFIG_ADDRESS, address);
        // Lê dados
        inl(PCI_CONFIG_DATA)
    }
}

/// Escreve via portas legadas.
fn write_config_legacy(addr: PciAddress, offset: u8, value: u32) {
    let address = make_address(addr, offset);

    unsafe {
        // Escreve endereço
        outl(PCI_CONFIG_ADDRESS, address);
        // Escreve dados
        outl(PCI_CONFIG_DATA, value);
    }
}

// =============================================================================
// IMPLEMENTAÇÃO ECAM (MMIO)
// =============================================================================

/// Lê via ECAM (memory-mapped).
fn read_config_ecam(addr: PciAddress, offset: u8) -> u32 {
    let base = *ECAM_BASE.lock();

    // Endereço ECAM: base + (bus << 20) + (device << 15) + (function << 12) + offset
    let ecam_addr = base
        + ((addr.bus as u64) << 20)
        + ((addr.device as u64) << 15)
        + ((addr.function as u64) << 12)
        + (offset as u64);

    unsafe { core::ptr::read_volatile(ecam_addr as *const u32) }
}

/// Escreve via ECAM (memory-mapped).
fn write_config_ecam(addr: PciAddress, offset: u8, value: u32) {
    let base = *ECAM_BASE.lock();

    let ecam_addr = base
        + ((addr.bus as u64) << 20)
        + ((addr.device as u64) << 15)
        + ((addr.function as u64) << 12)
        + (offset as u64);

    unsafe { core::ptr::write_volatile(ecam_addr as *mut u32, value) }
}

// =============================================================================
// FUNÇÕES DE LEITURA DE TAMANHOS ESPECÍFICOS
// =============================================================================

/// Lê um byte do espaço de configuração.
pub fn read_config_u8(addr: PciAddress, offset: u8) -> u8 {
    let value = read_config(addr, offset & 0xFC);
    let shift = (offset & 0x03) * 8;
    ((value >> shift) & 0xFF) as u8
}

/// Lê uma word (16 bits) do espaço de configuração.
pub fn read_config_u16(addr: PciAddress, offset: u8) -> u16 {
    let value = read_config(addr, offset & 0xFC);
    let shift = (offset & 0x02) * 8;
    ((value >> shift) & 0xFFFF) as u16
}

/// Escreve um byte no espaço de configuração.
pub fn write_config_u8(addr: PciAddress, offset: u8, value: u8) {
    let aligned_offset = offset & 0xFC;
    let shift = (offset & 0x03) * 8;
    let mask = !(0xFF << shift);

    let current = read_config(addr, aligned_offset);
    let new_value = (current & mask) | ((value as u32) << shift);
    write_config(addr, aligned_offset, new_value);
}

/// Escreve uma word (16 bits) no espaço de configuração.
pub fn write_config_u16(addr: PciAddress, offset: u8, value: u16) {
    let aligned_offset = offset & 0xFC;
    let shift = (offset & 0x02) * 8;
    let mask = !(0xFFFF << shift);

    let current = read_config(addr, aligned_offset);
    let new_value = (current & mask) | ((value as u32) << shift);
    write_config(addr, aligned_offset, new_value);
}

// =============================================================================
// INSTRUÇÕES DE I/O PORT (x86)
// =============================================================================

/// Lê 32 bits de uma porta I/O.
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

/// Escreve 32 bits em uma porta I/O.
#[inline]
unsafe fn outl(port: u16, value: u32) {
    asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nomem, nostack, preserves_flags)
    );
}
