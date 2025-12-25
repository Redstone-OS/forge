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
    /// Executa a instrução CPUID.
    ///
    /// # Safety
    /// Seguro do ponto de vista de memória, mas retorna dados crus do hardware.
    pub fn cpuid(leaf: u32, subleaf: u32) -> CpuidResult {
        let eax: u32;
        let ebx: u32;
        let ecx: u32;
        let edx: u32;

        unsafe {
            asm!(
                "push rbx",
                "cpuid",
                "mov {0:e}, ebx",
                "pop rbx",
                out(reg) ebx,
                inout("eax") leaf => eax,
                inout("ecx") subleaf => ecx,
                out("edx") edx,
                options(nomem, preserves_flags),
            );
        }
        CpuidResult { eax, ebx, ecx, edx }
    }

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

    /// Inicializa a FPU/SSE (Floating Point Unit / Streaming SIMD Extensions).
    ///
    /// Essencial para que o compilador Rust possa usar instruções otimizadas (XMM)
    /// em operações de memória (memcpy, memset) e formatação de strings,
    /// prevenindo exceções #UD (Invalid Opcode).
    pub unsafe fn init_sse() {
        let mut cr0: u64;
        let mut cr4: u64;

        // 1. Configurar CR0
        // - Limpar EM (Emulation) -> Bit 2
        // - Limpar TS (Task Switched) -> Bit 3 (Evita exceção #NM em first use)
        // - Setar MP (Monitor Coprocessor) -> Bit 1
        asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
        cr0 &= !(1 << 2); // Clear EM
        cr0 &= !(1 << 3); // Clear TS
        cr0 |= 1 << 1; // Set MP
        asm!("mov cr0, {}", in(reg) cr0, options(nomem, nostack, preserves_flags));

        // 2. Configurar CR4
        // - Setar OSFXSR (OS Support for FXSAVE/FXRSTOR) -> Bit 9
        // - Setar OSXMMEXCPT (OS Support for Unmasked SIMD FPU Exceptions) -> Bit 10
        asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
        cr4 |= 1 << 9; // Set OSFXSR
        cr4 |= 1 << 10; // Set OSXMMEXCPT
        asm!("mov cr4, {}", in(reg) cr4, options(nomem, nostack, preserves_flags));

        // 3. Inicializar unidade FPU (x87)
        asm!("fninit", options(nomem, nostack, preserves_flags));

        // 4. Inicializar unidade SSE (MXCSR)
        // Valor padrão 0x1F80: Todas as exceções mascaradas, Round to Nearest, Flush to Zero off
        let mxcsr: u32 = 0x1F80;
        asm!("ldmxcsr [{}]", in(reg) &mxcsr, options(nostack, preserves_flags));

        crate::ktrace!("(Arch) FPU/SSE habilitado (CR0.MP=1, CR4.OSFXSR=1, MXCSR=0x1F80)");
    }
}

/// Resultado de uma execução do CPUID (EAX, EBX, ECX, EDX).
#[derive(Debug, Clone, Copy)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
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
