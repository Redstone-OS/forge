/// Arquivo: x86_64/cpu.rs
///
/// Propósito: Implementação específica da arquitetura x86_64 para o trait `CpuTrait`.
/// Também fornece funções utilitárias para acesso a registradores específicos (MSRs, CR3)
/// e controle de estado da CPU.
///
/// Detalhes de Implementação:
/// - Implementa `CpuTrait` usando instruções assembly inline (`cli`, `sti`, `hlt`).
/// - Verifica o bit de interrupção (IF) no registrador RFLAGS.
/// - Fornece acesso seguro (unsafe) a MSRs via `rdmsr` e `wrmsr`.
/// - Gerencia leitura/escrita do registrador de tabela de páginas (CR3).

//! Implementação de CPU para x86_64

use crate::arch::traits::CpuTrait;

/// Implementação x86_64 do trait CPU
pub struct Cpu;

impl CpuTrait for Cpu {
    #[inline(always)]
    fn disable_interrupts() {
        // SAFETY: cli é seguro, apenas desabilita interrupções no core atual
        unsafe { core::arch::asm!("cli", options(nomem, nostack)); }
    }
    
    #[inline(always)]
    fn enable_interrupts() {
        // SAFETY: sti é seguro, apenas habilita interrupções no core atual
        unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
    }
    
    #[inline(always)]
    fn halt() {
        // SAFETY: hlt para a CPU até a próxima interrupção; seguro e economiza energia
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }
    
    fn current_core_id() -> u32 {
        // Lê APIC ID do LAPIC
        // TODO: Implementar leitura real do LAPIC quando o módulo APIC estiver pronto
        0
    }
    
    fn interrupts_enabled() -> bool {
        let rflags: u64;
        // SAFETY: Leitura de RFLAGS é segura e não tem efeitos colaterais
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {}",
                out(reg) rflags,
                options(nomem)
            );
        }
        (rflags & (1 << 9)) != 0 // Verifica o Bit 9 (Interrupt Flag)
    }
}

// Funções auxiliares específicas para x86_64 que NÃO fazem parte do trait genérico
impl Cpu {
    /// Lê um MSR (Model Specific Register)
    #[inline]
    pub fn read_msr(msr: u32) -> u64 {
        let (low, high): (u32, u32);
        // SAFETY: rdmsr lê o MSR especificado em ECX. O caller deve garantir que o MSR existe.
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
    
    /// Escreve em um MSR (Model Specific Register)
    #[inline]
    pub fn write_msr(msr: u32, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;
        // SAFETY: wrmsr escreve no MSR especificado em ECX. Operação privilegiada.
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
    
    /// Lê o registrador de controle CR3 (Page Table Base)
    #[inline]
    pub fn read_cr3() -> u64 {
        let value: u64;
        // SAFETY: Leitura de CR3 é segura e essencial para gerenciamento de memória
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack));
        }
        value
    }
    
    /// Escreve no registrador de controle CR3 (Troca de Tabela de Páginas / Contexto de Memória)
    /// 
    /// # Safety
    /// 
    /// O valor fornecido DEVE ser um endereço físico válido de uma tabela de páginas (PML4) alinhada.
    /// Carregar um CR3 inválido causará Triple Fault imediato.
    #[inline]
    pub unsafe fn write_cr3(value: u64) {
        // SAFETY: O caller garante que o endereço físico é válido. A instrução invalida o TLB (exceto global pages).
        core::arch::asm!("mov cr3, {}", in(reg) value, options(nomem, nostack));
    }
}
