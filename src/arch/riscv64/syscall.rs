/// Arquivo: riscv64/syscall.rs
///
/// Propósito: Interface de chamadas de sistema (syscall) para RISC-V.
/// No RISC-V, syscalls são disparadas pela instrução `ecall`.

/// Inicializa CSRs necessários para syscalls rápidas (se aplicável ao ambiente)
pub unsafe fn init() {
    // Configurações de ambiente para ecall
}

/// Handler de syscall chamado pelo trap handler
pub fn handle_syscall(id: usize, args: &[usize]) -> usize {
    // Dispatch para o core do kernel
    0
}
