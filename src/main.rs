//! Kernel Forge — Binário Principal.
//!
//! Responsabilidade:
//! 1. Configurar o ambiente de execução "naked" (Assembly).
//! 2. Inicializar a Stack.
//! 3. Habilitar SSE.
//! 4. **ZERAR BSS** (CRÍTICO - evita lixo em variáveis estáticas).
//! 5. Saltar para `core::entry::kernel_main` (da biblioteca `forge`).

#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]

// Importar a biblioteca do kernel.
use forge::core as kernel_core;

// Habilitar alocação
extern crate alloc;

// Símbolos do linker para BSS
extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
}

// Stack do kernel (64 KB) + Guard Page.
// A guard page é uma página NÃO mapeada após a stack que causa page fault
// em caso de stack overflow, permitindo detecção precoce do problema.
#[repr(align(16))]
struct KernelStack([u8; 64 * 1024]);

/// Guard page size (4KB) - não mapeada, detecta stack overflow
pub const GUARD_PAGE_SIZE: usize = 4096;

#[no_mangle]
static KERNEL_STACK: KernelStack = KernelStack([0; 64 * 1024]);

/// Guard page marker (deve ser NOT PRESENT no VMM após init)
/// O VMM deve chamar unmap_page() para este endereço durante init.
pub fn guard_page_virt() -> u64 {
    unsafe { &KERNEL_STACK as *const _ as u64 - GUARD_PAGE_SIZE as u64 }
}

/// Ponto de entrada Naked.
/// Configura o Stack Pointer (RSP), habilita SSE, ZERA BSS, e chama kernel_main.
///
/// # IMPORTANTE: Zeragem de BSS
///
/// A seção BSS contém variáveis estáticas não inicializadas. O padrão C/Rust
/// assume que elas são zeradas, mas o bootloader UEFI NÃO zera essa região.
/// Se não zerarmos aqui, variáveis como `TEST_ARENA` no BuddyAllocator
/// conterão lixo de memória, causando crashes.
#[naked]
#[no_mangle]
#[link_section = ".text._start"]
pub unsafe extern "C" fn _start(boot_info_addr: u64) -> ! {
    ::core::arch::asm!(
        // ============================================================
        // 1. Salvar argumento (boot_info) em R15 (Callee-saved)
        // ============================================================
        "mov r15, rdi",

        // ============================================================
        // 2. Configurar Stack Pointer (RSP)
        // ============================================================
        "lea rax, [rip + {stack}]",
        "lea rsp, [rax + {stack_size}]",

        // 3. Zerar RBP (Frame Pointer)
        "xor rbp, rbp",

        // ============================================================
        // 4. Habilitar SSE (necessário para código Rust)
        // ============================================================
        "mov rax, cr0",
        "and ax, 0xFFFB",      // Limpar CR0.EM (bit 2)
        "or ax, 0x2",          // Setar CR0.MP (bit 1)
        "mov cr0, rax",
        "mov rax, cr4",
        "or ax, 0x600",        // Setar CR4.OSFXSR (bit 9) e CR4.OSXMMEXCPT (bit 10)
        "mov cr4, rax",

        // ============================================================
        // 5. ZERAR BSS (CRÍTICO!)
        // ============================================================
        // Usa rep stosb que é otimizado em hardware nas CPUs modernas.
        // RDI = destino, RCX = contagem, AL = valor (0)
        "lea rdi, [rip + {bss_start}]",
        "lea rcx, [rip + {bss_end}]",
        "sub rcx, rdi",        // RCX = tamanho do BSS
        "xor eax, eax",        // AL = 0
        "rep stosb",           // Preenche [RDI..RDI+RCX) com AL

        // ============================================================
        // 6. Garantir alinhamento de 16 bytes para SSE (System V ABI)
        // ============================================================
        "and rsp, -16",

        // ============================================================
        // 7. Restaurar argumento e chamar kernel_main
        // ============================================================
        "mov rdi, r15",
        "call {kernel_main}",

        // ============================================================
        // 8. Halt Loop (se kernel_main retornar, o que não deve acontecer)
        // ============================================================
        "2:",
        "cli",
        "hlt",
        "jmp 2b",

        stack = sym KERNEL_STACK,
        stack_size = const 64 * 1024,
        bss_start = sym __bss_start,
        bss_end = sym __bss_end,
        kernel_main = sym kernel_core::entry::kernel_main,
        options(noreturn)
    );
}
