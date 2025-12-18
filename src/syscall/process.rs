//! Syscalls de Processos (0-99)
//!
//! # Syscalls Implementadas
//! - 0: fork
//! - 1: exec
//! - 2: exit
//! - 3: wait
//! - 4: kill
//! - 5: getpid
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar todas as syscalls essenciais
//! - TODO(prioridade=média, versão=v2.0): Adicionar getppid, setuid, setgid, etc

/// Syscall 0: fork
///
/// # TODOs
/// - TODO(prioridade=alta, versão=v1.0): Implementar fork()
pub fn sys_fork() -> isize {
    todo!("Implementar sys_fork()")
}

/// Syscall 1: exec
///
/// # TODOs
/// - TODO(prioridade=alta, versão=v1.0): Implementar exec()
pub fn sys_exec(path: &str) -> isize {
    todo!("Implementar sys_exec()")
}

/// Syscall 2: exit
///
/// # TODOs
/// - TODO(prioridade=alta, versão=v1.0): Implementar exit()
pub fn sys_exit(code: i32) -> ! {
    todo!("Implementar sys_exit()")
}
