/// Arquivo: aarch64/memory.rs
///
/// Propósito: Layout de memória específico para ARM64.
/// Gerencia TTBR0 (User) e TTBR1 (Kernel).

pub const KERNEL_OFFSET: usize = 0xFFFF000000000000;

/// Inicializa paginação básica
pub unsafe fn init() {
    // Configuração de MAIR_EL1, TCR_EL1 etc.
}
