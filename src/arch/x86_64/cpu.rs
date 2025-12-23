//! Implementação x86_64 das operações de CPU.
//! Usa Assembly inline para acesso direto ao hardware.

use crate::arch::traits::CpuOps;
use core::arch::asm;

/// Estrutura vazia que implementa o trait de CPU para x86_64.
pub struct X64Cpu;

impl CpuOps for X64Cpu {
    #[inline]
    fn halt() {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }

    #[inline]
    fn disable_interrupts() {
        unsafe {
            asm!("cli", options(nomem, nostack, preserves_flags));
        }
    }

    #[inline]
    fn enable_interrupts() {
        unsafe {
            asm!("sti", options(nomem, nostack, preserves_flags));
        }
    }

    #[inline]
    fn are_interrupts_enabled() -> bool {
        let rflags: u64;
        unsafe {
            asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
        }
        // Bit 9 é IF (Interrupt Flag)
        (rflags & (1 << 9)) != 0
    }
}
