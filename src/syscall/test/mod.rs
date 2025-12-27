//! # Syscall Tests
//!
//! Testes do subsistema de syscalls.

/// Executa testes de syscall
#[cfg(feature = "self_test")]
pub fn run_syscall_tests() {
    crate::kinfo!("[Syscall Test] Iniciando...");

    test_handle_table();
    test_syscall_args();

    crate::kinfo!("[Syscall Test] Todos os testes passaram!");
}

#[cfg(feature = "self_test")]
fn test_handle_table() {
    use super::handle::{HandleRights, HandleTable, HandleType};

    let mut table = HandleTable::new();

    // Alocar handle
    let h = table
        .alloc(
            HandleType::File,
            core::ptr::null_mut(),
            HandleRights::FILE_READ,
        )
        .expect("alloc falhou");

    assert!(h.is_valid());
    assert!(table.get(h).is_some());

    // Fechar handle
    assert!(table.close(h));
    assert!(table.get(h).is_none());

    crate::ktrace!("[Syscall Test] test_handle_table OK");
}

#[cfg(feature = "self_test")]
fn test_syscall_args() {
    use super::abi::SyscallArgs;

    let args = SyscallArgs::empty();
    assert_eq!(args.num, 0);
    assert_eq!(args.arg1, 0);

    crate::ktrace!("[Syscall Test] test_syscall_args OK");
}
