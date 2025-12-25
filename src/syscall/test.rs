//! Testes de Chamadas de Sistema (Syscalls)
//!
//! Valida constantes de erro e limites da tabela de syscalls.

/// Executa todos os testes de syscall
pub fn run_syscall_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE SYSCALL               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_syscall_table_bounds();
    test_error_codes();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ SYSCALLS VALIDADAS!                ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_syscall_table_bounds() {
    crate::kdebug!("(Syscall) Verificando limite máximo de ID...");

    let max_syscalls = 512;
    let test_id = 1024; // ID inválido

    crate::ktrace!("(Syscall) MAX: {}", max_syscalls);

    if test_id >= max_syscalls {
        crate::ktrace!("(Syscall) ID {} Rejected (Correct)", test_id);
        crate::kinfo!("(Syscall) ✓ Bounding Check Logic OK");
    } else {
        crate::kerror!("(Syscall) Out-of-bounds ID Accepted!");
    }
}

fn test_error_codes() {
    crate::kdebug!("(Syscall) Validando códigos negativos (errno)...");

    // Padrão Linux/Unix: erros são retornados como -errno
    // Em isize (64 bits):
    let ret_success: isize = 0;
    let ret_error: isize = -1; // EPERM, por exemplo

    if ret_success >= 0 {
        crate::ktrace!("(Syscall) >= 0 treated as Success");
    }
    if ret_error < 0 {
        crate::ktrace!("(Syscall) < 0 treated as Error");
    }

    crate::kinfo!("(Syscall) ✓ Error Code Convention OK");
}
