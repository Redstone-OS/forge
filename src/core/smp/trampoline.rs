//! # SMP Trampoline
//!
//! Código para acordar APs (Application Processors).
//!
//! O código assembly está em `trampoline.s`.

use core::arch::global_asm;

/// Endereço onde o trampoline será copiado (< 1MB)
pub const TRAMPOLINE_ADDR: u64 = 0x8000;

/// Offset dos dados após o código
pub const TRAMPOLINE_DATA_OFFSET: usize = 0x100;

/// Dados passados para o AP via trampoline
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrampolineData {
    pub cr3: u64,        // 0x00
    pub gdt_base: u64,   // 0x08
    pub gdt_limit: u16,  // 0x10
    pub _pad0: [u8; 6],  // 0x12
    pub stack_top: u64,  // 0x18
    pub ap_id: u32,      // 0x20
    pub _pad1: u32,      // 0x24
    pub rust_entry: u64, // 0x28
}

global_asm!(include_str!("trampoline.s"));

extern "C" {
    static ap_trampoline_start: u8;
    static ap_trampoline_end: u8;
}

pub fn trampoline_code() -> &'static [u8] {
    unsafe {
        let start = &ap_trampoline_start as *const u8;
        let end = &ap_trampoline_end as *const u8;
        let len = end as usize - start as usize;
        core::slice::from_raw_parts(start, len)
    }
}

#[inline]
pub unsafe fn get_trampoline_data() -> *mut TrampolineData {
    (TRAMPOLINE_ADDR + TRAMPOLINE_DATA_OFFSET as u64) as *mut TrampolineData
}
