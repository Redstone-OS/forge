//! # SMP Trampoline
//!
//! Código para acordar APs (Application Processors).
//!
//! O código assembly está em `trampoline.S` e é incluído via `global_asm!`.

use core::arch::global_asm;

/// Endereço onde o trampoline será copiado (< 1MB)
pub const TRAMPOLINE_ADDR: u64 = 0x8000;

/// Offset dos dados após o código
pub const TRAMPOLINE_DATA_OFFSET: usize = 0x100;

/// Dados passados para o AP via trampoline
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrampolineData {
    /// CR3 do kernel (page table raiz)
    pub cr3: u64, // offset 0x00
    /// Endereço base da GDT do kernel
    pub gdt_base: u64, // offset 0x08
    /// Limite da GDT
    pub gdt_limit: u16, // offset 0x10
    /// Padding
    pub _pad0: [u8; 6], // offset 0x12
    /// Stack top para este AP
    pub stack_top: u64, // offset 0x18
    /// ID lógico do AP
    pub ap_id: u32, // offset 0x20
    /// Padding
    pub _pad1: u32, // offset 0x24
    /// Entry point do código Rust (ap_entry)
    pub rust_entry: u64, // offset 0x28
}

// Incluir o código assembly do trampoline
global_asm!(include_str!("trampoline.S"));

// Símbolos exportados pelo assembly
extern "C" {
    static ap_trampoline_start: u8;
    static ap_trampoline_end: u8;
}

/// Obtém o ponteiro para o início do código do trampoline
pub fn trampoline_code() -> &'static [u8] {
    unsafe {
        let start = &ap_trampoline_start as *const u8;
        let end = &ap_trampoline_end as *const u8;
        let len = end as usize - start as usize;
        core::slice::from_raw_parts(start, len)
    }
}

/// Obtém o ponteiro para TrampolineData
///
/// Usa endereço físico diretamente já que está no identity map (< 4GB)
#[inline]
pub unsafe fn get_trampoline_data() -> *mut TrampolineData {
    (TRAMPOLINE_ADDR + TRAMPOLINE_DATA_OFFSET as u64) as *mut TrampolineData
}
