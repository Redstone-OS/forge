//! Kernel Forge — Binário Principal.
//!
//! Responsabilidade:
//! 1. Configurar o ambiente de execução "naked" (Assembly).
//! 2. Inicializar a Stack.
//! 3. Habilitar SSE.
//! 4. Saltar para `core::entry::kernel_main` (da biblioteca `forge`).

#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![feature(asm_const)]

// Importar a biblioteca do kernel.
use forge::{core as kernel_core, mm};

// Habilitar alocação
extern crate alloc;

// Stack do kernel (16 KB).
#[repr(align(16))]
struct KernelStack([u8; 16 * 1024]);

#[no_mangle]
static KERNEL_STACK: KernelStack = KernelStack([0; 16 * 1024]);

/// Ponto de entrada Naked.
/// Configura o Stack Pointer (RSP) e habilita SSE antes de chamar o código Rust.
#[naked]
#[no_mangle]
// CORREÇÃO: Forçar esta função para a seção .text._start,
// garantindo que o linker.ld a coloque bem no início (1MB).
#[link_section = ".text._start"]
pub unsafe extern "C" fn _start(boot_info_addr: u64) -> ! {
    ::core::arch::asm!(
        // 1. Salvar argumento (boot_info) em R15 (Callee-saved)
        "mov r15, rdi",

        // 2. Configurar Stack Pointer (RSP)
        "lea rax, [rip + {stack}]",
        "lea rsp, [rax + {stack_size}]",

        // 3. Zerar RBP (Frame Pointer)
        "xor rbp, rbp",

        // 4. Habilitar SSE
        "mov rax, cr0",
        "and ax, 0xFFFB",
        "or ax, 0x2",
        "mov cr0, rax",
        "mov rax, cr4",
        "or ax, 0x600",
        "mov cr4, rax",

        // 5. Restaurar argumento e chamar kernel_main
        "mov rdi, r15",
        "call {kernel_main}",

        // 6. Trap (Halt Loop Robusto)
        "2:",
        "cli",
        "hlt",
        "jmp 2b",

        stack = sym KERNEL_STACK,
        stack_size = const 16 * 1024,
        kernel_main = sym kernel_core::entry::kernel_main,
        options(noreturn)
    );
}

/// Handler de erro de alocação (OOM).
#[alloc_error_handler]
fn alloc_error_handler(layout: ::core::alloc::Layout) -> ! {
    panic!("Kernel OOM: {:?}", layout)
}
