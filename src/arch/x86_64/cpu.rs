//! Implementação de CPU para x86_64

use crate::arch::traits::CpuTrait;

/// Implementação x86_64 do trait CPU
pub struct Cpu;

impl CpuTrait for Cpu {
    #[inline(always)]
    fn disable_interrupts() {
        // SAFETY: cli é seguro, apenas desabilita interrupções
        unsafe { core::arch::asm!("cli", options(nomem, nostack)); }
    }
    
    #[inline(always)]
    fn enable_interrupts() {
        // SAFETY: sti é seguro, apenas habilita interrupções
        unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
    }
    
    #[inline(always)]
    fn halt() {
        // SAFETY: hlt para CPU até próxima interrupção
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }
    
    fn current_core_id() -> u32 {
        // Lê APIC ID do LAPIC
        // TODO: Implementar leitura real do LAPIC
        0
    }
    
    fn interrupts_enabled() -> bool {
        let rflags: u64;
        // SAFETY: Leitura de RFLAGS é segura
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {}",
                out(reg) rflags,
                options(nomem)
            );
        }
        (rflags & (1 << 9)) != 0 // Bit IF
    }
}

// Funções auxiliares que NÃO fazem parte do trait
impl Cpu {
    /// Lê um MSR (Model Specific Register)
    #[inline]
    pub fn read_msr(msr: u32) -> u64 {
        let (low, high): (u32, u32);
        // SAFETY: rdmsr lê MSR especificado em ECX
        unsafe {
            core::arch::asm!(
                "rdmsr",
                in("ecx") msr,
                out("eax") low,
                out("edx") high,
                options(nomem, nostack)
            );
        }
        ((high as u64) << 32) | (low as u64)
    }
    
    /// Escreve em um MSR
    #[inline]
    pub fn write_msr(msr: u32, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;
        // SAFETY: wrmsr escreve MSR especificado em ECX
        unsafe {
            core::arch::asm!(
                "wrmsr",
                in("ecx") msr,
                in("eax") low,
                in("edx") high,
                options(nomem, nostack)
            );
        }
    }
    
    /// Lê CR3 (Page Table Base)
    #[inline]
    pub fn read_cr3() -> u64 {
        let value: u64;
        // SAFETY: Leitura de CR3 é segura
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack));
        }
        value
    }
    
    /// Escreve CR3 (troca page table)
    /// 
    /// # Safety
    /// 
    /// O valor deve ser um endereço físico válido de uma page table.
    #[inline]
    pub unsafe fn write_cr3(value: u64) {
        // SAFETY: Caller garante que value é válido
        core::arch::asm!("mov cr3, {}", in(reg) value, options(nomem, nostack));
    }
}
