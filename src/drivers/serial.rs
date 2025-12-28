//! # Driver Serial v0.1.0
//!
//! Driver de saída ultra-minimalista para QEMU debugcon.
//!
//! ## Design
//! - **Zero polling**: QEMU debugcon é instantâneo
//! - **Zero buffer**: Saída direta, sem buffer circular
//! - **Zero interrupts**: Sem estado de UART para gerenciar
//! - **Zero alocação**: Assembly inline puro
//! - **Zero SSE/AVX**: Todas operações em registradores de propósito geral

/// Escreve uma string para QEMU debugcon.
#[inline(always)]
pub fn write(s: &str) {
    let ptr = s.as_ptr();
    let len = s.len();

    if len == 0 {
        return;
    }

    // SAFETY: Escrever na porta debugcon do QEMU é sempre seguro.
    // Sem estado de hardware para corromper, sem espera necessária.
    unsafe {
        core::arch::asm!(
            "2:",
            "test {len}, {len}",
            "jz 3f",
            "mov al, [{ptr}]",
            "out 0xE9, al",
            "inc {ptr}",
            "dec {len}",
            "jmp 2b",
            "3:",
            ptr = inout(reg) ptr => _,
            len = inout(reg) len => _,
            out("al") _,
            options(nostack, preserves_flags)
        );
    }
}
