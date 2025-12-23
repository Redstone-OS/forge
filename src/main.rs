// (FASE2) src/main.rs
//! Kernel Forge — Binário Principal.
//!
//! Responsabilidade:
//! 1. Configurar o ambiente de execução "naked" (Assembly).
//! 2. Inicializar a Stack.
//! 3. Habilitar SSE (necessário para Rust).
//! 4. Saltar para `core::entry::kernel_main`.

#![no_std]
#![no_main]
#![feature(naked_functions)]

// Importar a biblioteca do kernel
use forge::{core, mm};

// Stack do kernel (16 KB).
// Alinhamento de 16 bytes é mandatório para a System V ABI x86_64.
#[repr(align(16))]
struct KernelStack([u8; 16 * 1024]);

#[no_mangle]
static KERNEL_STACK: KernelStack = KernelStack([0; 16 * 1024]);

/// Ponto de entrada Naked.
/// Configura o Stack Pointer (RSP) e habilita SSE antes de chamar o código Rust.
#[naked]
#[no_mangle]
pub unsafe extern "C" fn _start(boot_info_addr: u64) -> ! {
    core::arch::naked_asm!(
        // 1. Salvar argumento (boot_info) em R15 (Callee-saved)
        "mov r15, rdi",

        // 2. Configurar Stack Pointer (RSP)
        // Usamos RIP-relative addressing para encontrar a stack de forma segura
        "lea rsp, [rip + {stack} + {stack_size}]",

        // 3. Zerar RBP (Frame Pointer) para terminar stack traces corretamente
        "xor rbp, rbp",

        // 4. Habilitar SSE (Necessário para floats e otimizações de memcpy do Rust)
        // CR0: Clear EM (bit 2), Set MP (bit 1)
        "mov rax, cr0",
        "and ax, 0xFFFB",
        "or ax, 0x2",
        "mov cr0, rax",
        // CR4: Set OSFXSR (bit 9) and OSXMMEXCPT (bit 10)
        "mov rax, cr4",
        "or ax, 0x600",
        "mov cr4, rax",

        // 5. Restaurar argumento (RDI) e chamar kernel_main
        "mov rdi, r15",
        "call {kernel_main}",

        // 6. Trap (caso kernel_main retorne, o que é impossível)
        "cli",
        "hlt",
        "jmp . - 2",

        stack = sym KERNEL_STACK,
        stack_size = const 16 * 1024,
        kernel_main = sym core::entry::kernel_main,
    );
}

/// Handler de erro de alocação (OOM).
/// Requerido porque usamos `extern crate alloc`.
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Kernel OOM: {:?}", layout)
}
