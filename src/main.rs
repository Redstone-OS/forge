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

// =============================================================================
// STACK DO KERNEL - TAMANHO DINÂMICO DEBUG/RELEASE
// =============================================================================
//
// Release: 32 KB + 4KB guard = 36 KB total
// Debug:   512 KB + 4KB guard = 516 KB total
//
// A guard page (primeiros 4KB) é desmapeada para detectar stack overflow.
// A stack cresce de cima para baixo: RSP inicia em (base + size) e decresce.
// Quando RSP chegar nos primeiros 4KB, ocorre page fault.
//
// Layout:
//   [Guard 4KB | Stack utilizável (SIZE - 4KB) ]
//    ^base                                    ^RSP inicial

/// Tamanho da stack em bytes (condicional) - INCLUI guard page de 4KB
#[cfg(debug_assertions)]
pub const KERNEL_STACK_SIZE: usize = 512 * 1024; // 512 KB para debug (508KB úteis)

#[cfg(not(debug_assertions))]
pub const KERNEL_STACK_SIZE: usize = 32 * 1024; // 32 KB para release (28KB úteis)

#[repr(align(4096))] // Alinhar a 4KB para guard page funcionar
struct KernelStack([u8; KERNEL_STACK_SIZE]);

/// Guard page size (4KB) - não mapeada, detecta stack overflow
pub const GUARD_PAGE_SIZE: usize = 4096;

// CRÍTICO: link_section=".bss" força a stack para o final da memória do kernel.
// Sem isso, a stack vai para .rodata (logo após .text), e a guard page
// calculada como (stack_base - 4KB) acaba DENTRO do .text, desmapeando código!
#[link_section = ".bss"]
#[no_mangle]
static mut KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);

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
        // 4. SSE DESABILITADO no target spec
        // ============================================================
        // SSE foi desabilitado via x86_64-redstone.json (-sse,-sse2,+soft-float)
        // O compilador não gerará instruções SSE/AVX, então não precisamos
        // configurar CR0/CR4 para SSE aqui.

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
        stack_size = const KERNEL_STACK_SIZE,
        bss_start = sym __bss_start,
        bss_end = sym __bss_end,
        kernel_main = sym kernel_core::entry::kernel_main,
        options(noreturn)
    );
}
