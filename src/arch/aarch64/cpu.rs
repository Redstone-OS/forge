/// Arquivo: aarch64/cpu.rs
///
/// Propósito: Implementação específica da arquitetura ARM64 para o trait `CpuTrait`.
/// Gerencia estados da CPU como interrupções, halt e identificação do núcleo.
///
/// Detalhes de Implementação:
/// - Implementa `CpuTrait` usando instruções e registradores de sistema ARM.
/// - `current_core_id` utiliza o registrador `MPIDR_EL1`.
/// - Controle de interrupções via registrador `DAIF`.
use crate::arch::traits::CpuTrait;

/// Implementação ARM64 do trait CPU
pub struct Cpu;

impl CpuTrait for Cpu {
    #[inline(always)]
    fn disable_interrupts() {
        unsafe {
            // Mascara IRQ e FIQ no registrador DAIF
            core::arch::asm!("msr daifset, #3", options(nomem, nostack));
        }
    }

    #[inline(always)]
    fn enable_interrupts() {
        unsafe {
            // Desmascara IRQ e FIQ no registrador DAIF
            core::arch::asm!("msr daifclr, #3", options(nomem, nostack));
        }
    }

    #[inline(always)]
    fn halt() {
        unsafe {
            // ARM 'wfi' (Wait For Interrupt)
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }

    #[inline(always)]
    fn current_core_id() -> u32 {
        let mpidr: u64;
        unsafe {
            core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nomem, nostack));
        }
        // Os bits 0-7 geralmente contêm o Aff0 (CPU ID dentro do cluster)
        (mpidr & 0xFF) as u32
    }

    #[inline(always)]
    fn interrupts_enabled() -> bool {
        let daif: u64;
        unsafe {
            core::arch::asm!("mrs {}, daif", out(reg) daif, options(nomem, nostack));
        }
        // I-bit (Interrupção) está no bit 7. Se 0, interrupções estão habilitadas.
        (daif & (1 << 7)) == 0
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

    /// Lê o registrador TTBR0_EL1 (Translation Table Base Register 0)
    /// Similar ao CR3 do x86_64.
    #[inline]
    pub fn read_ttbr0() -> u64 {
        let value: u64;
        unsafe {
            core::arch::asm!("mrs {}, ttbr0_el1", out(reg) value, options(nomem, nostack));
        }
        value
    }

    /// Escreve no registrador TTBR0_EL1 para trocar o contexto de memória.
    #[inline]
    pub unsafe fn write_ttbr0(value: u64) {
        core::arch::asm!(
            "msr ttbr0_el1, {}",
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
            in(reg) value,
            options(nostack)
        );
    }
}
