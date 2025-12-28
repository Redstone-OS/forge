//! # Forge Kernel — Entry Point
//!
//! Binário principal do kernel Redstone OS.
//!
//! ## Responsabilidades
//!
//! 1. Configurar ambiente de execução (stack, BSS)
//! 2. Saltar para `kernel_main` na biblioteca
//!
//! ## Layout de Stack
//!
//! ```text
//! [Guard Page 4KB | Stack Utilizável (SIZE - 4KB)]
//!  ^base                                        ^RSP inicial
//! ```
//!
//! A guard page é desmapeada para detectar stack overflow.

#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]

// =============================================================================
// IMPORTS
// =============================================================================

use forge::core::boot as kernel_boot;

extern crate alloc;

// =============================================================================
// LINKER SYMBOLS
// =============================================================================

extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
}

// =============================================================================
// KERNEL STACK
// =============================================================================

/// Stack size (inclui guard page de 4KB)
pub const KERNEL_STACK_SIZE: usize = 64 * 1024; // 64 KB release

/// Guard page size
pub const GUARD_PAGE_SIZE: usize = 4096;

#[repr(C, align(4096))]
struct KernelStack([u8; KERNEL_STACK_SIZE]);

#[link_section = ".bss"]
#[no_mangle]
static mut KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);

/// Retorna endereço da guard page (primeiros 4KB da stack)
#[inline]
pub fn guard_page_addr() -> u64 {
    // SAFETY: Estamos apenas lendo o endereço, não o conteúdo
    unsafe { core::ptr::addr_of!(KERNEL_STACK) as u64 }
}

// =============================================================================
// ENTRY POINT (GLOBAL ASM)
// =============================================================================

// Usamos global_asm! para garantir que não haja prêambulos do compilador (CET/IBT)
// ou padding indesejado no início da seção .text._start.
core::arch::global_asm!(
    ".section .text._start",
    ".global _start",
    "_start:",
    // -------------------------------------------------------------------------
    // 1. Preservar boot_info em R15 (callee-saved)
    // -------------------------------------------------------------------------
    "mov r15, rdi",

    // -------------------------------------------------------------------------
    // 2. Configurar stack pointer
    // -------------------------------------------------------------------------
    "lea rax, [rip + {stack}]",
    "lea rsp, [rax + {stack_size}]",

    // -------------------------------------------------------------------------
    // 3. Zerar frame pointer
    // -------------------------------------------------------------------------
    "xor rbp, rbp",

    // -------------------------------------------------------------------------
    // 4. Zerar BSS (CRÍTICO!)
    // -------------------------------------------------------------------------
    "lea rdi, [rip + {bss_start}]",
    "lea rcx, [rip + {bss_end}]",
    "sub rcx, rdi",
    "xor eax, eax",
    "rep stosb",

    // -------------------------------------------------------------------------
    // 5. Alinhar stack (System V ABI: 16 bytes)
    // -------------------------------------------------------------------------
    "and rsp, -16",

    // -------------------------------------------------------------------------
    // 6. Chamar kernel_main(boot_info)
    // -------------------------------------------------------------------------
    "mov rdi, r15",
    "call {kernel_main}",

    // -------------------------------------------------------------------------
    // 7. Halt loop (Fallback)
    // -------------------------------------------------------------------------
    "2:",
    "cli",
    "hlt",
    "jmp 2b",

    stack = sym KERNEL_STACK,
    stack_size = const KERNEL_STACK_SIZE,
    bss_start = sym __bss_start,
    bss_end = sym __bss_end,
    kernel_main = sym kernel_boot::kernel_main,
);
