/// Arquivo: x86_64/memory.rs
///
/// Propósito: Configuração inicial de memória e paginação específica da arquitetura.
/// Embora o gerenciamento de memória principal fique no módulo `mm`, a arquitetura
/// precisa fornecer primitivas para ativação de tabelas de página e setup inicial.
///
/// Detalhes de Implementação:
/// - Verificação inicial do estado de paginação (CR0, CR3, EFER).
/// - Funções auxiliares para invalidação de TLB (`invlpg`).

//! Setup inicial de paginação e memória

/// Inicializa subsistema de memória da arquitetura (se necessário).
/// Geralmente o bootloader já configurou paginação básica.
pub fn init() {
    // Verificar se paginação está ativa
    // TODO: Implementar verificações de sanidade
}

/// Invalida uma entrada no TLB (Translation Lookaside Buffer) para o endereço virtual dado.
/// 
/// # Safety
/// 
/// Instrução `invlpg` é privilegiada.
pub unsafe fn tlb_flush(vaddr: u64) {
    core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags));
}

/// Invalida todo o TLB (recarregando CR3).
pub unsafe fn tlb_flush_all() {
    let cr3 = crate::arch::x86_64::cpu::Cpu::read_cr3();
    crate::arch::x86_64::cpu::Cpu::write_cr3(cr3);
}
