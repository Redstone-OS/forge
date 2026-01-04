/// Arquivo: riscv64/memory.rs
///
/// Propósito: Definições específicas de layout de memória para RISC-V.
/// Gerencia as tabelas de páginas Sv39 ou Sv48.

pub const KERNEL_OFFSET: usize = 0xFFFFFFC000000000; // Exemplo para Sv39

/// Inicializa paginação básica ou limpa TLBs
pub unsafe fn init() {
    // Configuração inicial de MMU
}
