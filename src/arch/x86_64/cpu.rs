// (FIX) src/arch/x86_64/cpu.rs
//! Implementação x86_64 das operações de CPU (HAL).
//!
//! Usa Assembly inline para acesso direto ao hardware, controle de interrupções
//! e leitura de registradores específicos de modelo (MSR).
//!
//! # Segurança
//! Esta implementação assume que o código está rodando em modo longo (64-bit)
//! e nível de privilégio de kernel (Ring 0).

use crate::arch::traits::cpu::{CoreId, CpuOps};
use crate::sys::Errno;
use core::arch::asm;

/// MSR: IA32_APIC_BASE (Endereço base do APIC Local)
const IA32_APIC_BASE: u32 = 0x1B;
/// Bit 8 do IA32_APIC_BASE indica se é o processador de boot (BSP)
const MSR_APIC_BSP_FLAG: u64 = 1 << 8;

pub struct X64Cpu;

impl X64Cpu {
    /// Lê um Model Specific Register (MSR).
    ///
    /// # Safety
    /// Ler um MSR reservado ou inválido pode causar uma falha geral de proteção (#GP).
    #[inline]
    unsafe fn rdmsr(msr: u32) -> u64 {
        let (high, low): (u32, u32);
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags),
        );
        ((high as u64) << 32) | (low as u64)
    }
}

impl CpuOps for X64Cpu {
    /// Retorna o ID do núcleo atual.
    ///
    /// Tenta ler do CPUID (Initial APIC ID) se o GS Base ainda não estiver configurado.
    fn current_id() -> CoreId {
        // Em um sistema totalmente inicializado, leríamos do registrador GS (Per-CPU).
        // Durante o boot, usamos CPUID folha 1, bits 24-31 de EBX (Initial APIC ID).
        //
        // CORREÇÃO CRÍTICA: Preservação manual de RBX.
        // O LLVM reserva RBX para uso interno, então não podemos usá-lo como operando de saída direto.
        let ebx: u32;
        unsafe {
            asm!(
                "push rbx",       // Salvar RBX original
                "cpuid",          // Executar CPUID (clobbers EAX, EBX, ECX, EDX)
                "mov {0:e}, ebx", // Mover resultado de EBX para variável de saída
                "pop rbx",        // Restaurar RBX original
                out(reg) ebx,
                inout("eax") 1u32 => _,
                out("ecx") _,
                out("edx") _,
                // 'nostack' removido pois estamos usando push/pop
                options(nomem, preserves_flags),
            );
        }
        CoreId(ebx >> 24)
    }

    /// Verifica se este é o Bootstrap Processor (BSP).
    /// Lê o MSR IA32_APIC_BASE bit 8.
    fn is_bsp() -> bool {
        unsafe {
            let msr_value = Self::rdmsr(IA32_APIC_BASE);
            (msr_value & MSR_APIC_BSP_FLAG) != 0
        }
    }

    /// Para a execução da CPU até a próxima interrupção (HLT).
    #[inline]
    fn halt() {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }

    /// Dica para a CPU que estamos em um spinloop (PAUSE).
    #[inline]
    fn relax() {
        unsafe {
            asm!("pause", options(nomem, nostack, preserves_flags));
        }
    }

    /// Barreira de memória (MFENCE).
    /// Garante que todas as operações de memória anteriores completem antes das posteriores.
    #[inline]
    fn memory_fence() {
        unsafe {
            asm!("mfence", options(nostack, preserves_flags));
        }
    }

    /// Desabilita interrupções (CLI).
    ///
    /// # Safety
    /// Requer privilégios de Ring 0.
    #[inline]
    unsafe fn disable_interrupts() {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }

    /// Habilita interrupções (STI).
    ///
    /// # Safety
    /// Requer privilégios de Ring 0. Pode causar preempção imediata.
    #[inline]
    unsafe fn enable_interrupts() {
        asm!("sti", options(nomem, nostack, preserves_flags));
    }

    /// Verifica se as interrupções estão habilitadas (RFLAGS.IF).
    #[inline]
    fn are_interrupts_enabled() -> bool {
        let rflags: u64;
        unsafe {
            // PUSHFQ empilha RFLAGS, POP retira para registrador.
            // Removemos 'nostack' por segurança.
            asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
        }
        // Bit 9 é IF (Interrupt Flag)
        (rflags & (1 << 9)) != 0
    }
    // --- Suporte a SMP (Stub até driver APIC estar pronto) ---

    fn send_ipi(_target: CoreId, _vector: u8) -> Result<(), Errno> {
        // Requer Driver Local APIC inicializado.
        // TODO: Conectar com crate::drivers::apic::send_ipi
        Err(Errno::ENOSYS)
    }

    fn broadcast_ipi(_vector: u8) -> Result<(), Errno> {
        // Requer Driver Local APIC inicializado.
        Err(Errno::ENOSYS)
    }
}
