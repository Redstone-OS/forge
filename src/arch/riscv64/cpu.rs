/// Arquivo: riscv64/cpu.rs
///
/// Propósito: Implementação específica da arquitetura RISC-V 64 para o trait `CpuTrait`.
/// Gerencia estados da CPU como interrupções, halt e identificação do hart (core).
///
/// Detalhes de Implementação:
/// - Implementa `CpuTrait` usando CSRs (Control and Status Registers).
/// - `current_core_id` utiliza o registrador `tp` (Thread Pointer) ou `mhartid`.
/// - Controle de interrupções via bit `sie` no registrador `sstatus`.
use crate::arch::traits::CpuTrait;

/// Implementação RISC-V 64 do trait CPU
pub struct Cpu;

impl CpuTrait for Cpu {
    #[inline(always)]
    fn disable_interrupts() {
        unsafe {
            // Limpa o bit SIE (Supervisor Interrupt Enable) no sstatus
            core::arch::asm!("csrci sstatus, 1 << 1", options(nomem, nostack));
        }
    }

    #[inline(always)]
    fn enable_interrupts() {
        unsafe {
            // Seta o bit SIE (Supervisor Interrupt Enable) no sstatus
            core::arch::asm!("csrsi sstatus, 1 << 1", options(nomem, nostack));
        }
    }

    #[inline(always)]
    fn halt() {
        unsafe {
            // RISC-V 'wfi' (Wait For Interrupt) é o equivalente ao 'hlt'
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }

    #[inline(always)]
    fn current_core_id() -> u32 {
        let hart_id: usize;
        unsafe {
            // Em muitas implementações, o ID do hart é armazenado no tp ou lido via CSR
            core::arch::asm!("mv {}, tp", out(reg) hart_id, options(nomem, nostack));
        }
        hart_id as u32
    }

    #[inline(always)]
    fn interrupts_enabled() -> bool {
        let sstatus: usize;
        unsafe {
            core::arch::asm!("csrr {}, sstatus", out(reg) sstatus, options(nomem, nostack));
        }
        (sstatus & (1 << 1)) != 0
    }
}

impl Cpu {
    #[inline(always)]
    pub fn disable_interrupts() {
        <Self as CpuTrait>::disable_interrupts();
    }

    #[inline(always)]
    pub fn enable_interrupts() {
        <Self as CpuTrait>::enable_interrupts();
    }

    #[inline(always)]
    pub fn halt() {
        <Self as CpuTrait>::halt();
    }

    #[inline(always)]
    pub fn current_core_id() -> u32 {
        <Self as CpuTrait>::current_core_id()
    }

    #[inline(always)]
    pub fn interrupts_enabled() -> bool {
        <Self as CpuTrait>::interrupts_enabled()
    }

    /// Lê o registrador satp (Supervisor Address Translation and Protection)
    /// Similar ao CR3 do x86_64.
    #[inline]
    pub fn read_satp() -> usize {
        let value: usize;
        unsafe {
            core::arch::asm!("csrr {}, satp", out(reg) value, options(nomem, nostack));
        }
        value
    }

    /// Escreve no registrador satp para trocar o espaço de endereçamento.
    ///
    /// # Safety
    ///
    /// O valor deve apontar para uma root page table válida.
    #[inline]
    pub unsafe fn write_satp(value: usize) {
        core::arch::asm!("csrw satp, {}", in(reg) value, options(nomem, nostack));
        // Sfence.vma para invalidar TLB
        core::arch::asm!("sfence.vma zero, zero", options(nostack));
    }
}
