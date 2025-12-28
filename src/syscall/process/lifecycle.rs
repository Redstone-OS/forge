//! # Process Lifecycle
//!
//! exit, spawn, wait, yield
//!
//! Refatorado para usar API estática do scheduler.

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_exit_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_exit(args.arg1 as i32)
}

pub fn sys_spawn_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_spawn(args.arg1, args.arg2, args.arg3, args.arg4)
}

pub fn sys_wait_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_wait(args.arg1, args.arg2 as u64)
}

pub fn sys_yield_wrapper(_args: &SyscallArgs) -> SysResult<usize> {
    sys_yield()
}

// === IMPLEMENTAÇÕES ===

/// Encerra o processo atual
///
/// Nunca retorna.
pub fn sys_exit(code: i32) -> SysResult<usize> {
    crate::kinfo!("(Syscall) sys_exit code=", code as u64);

    // TODO: Marcar processo como terminado
    // TODO: Notificar processos esperando (wait)
    // TODO: Limpar recursos

    // Por agora: loop infinito cedendo CPU
    loop {
        let _ = sys_yield();
    }
}

/// Cria novo processo
///
/// # Args
/// - path_ptr: caminho do executável
/// - path_len: tamanho do caminho
/// - args_ptr: argumentos (array de strings)
/// - args_len: número de argumentos
///
/// # Returns
/// PID do novo processo ou erro
pub fn sys_spawn(
    path_ptr: usize,
    path_len: usize,
    args_ptr: usize,
    args_len: usize,
) -> SysResult<usize> {
    // TODO: Validar ponteiros
    // TODO: Ler path do userspace
    // TODO: Carregar ELF via VFS
    // TODO: Criar novo address space
    // TODO: Criar nova Task
    // TODO: Adicionar ao scheduler
    // TODO: Retornar PID

    let _ = (path_ptr, path_len, args_ptr, args_len);
    crate::kwarn!("(Syscall) sys_spawn não implementado");
    Err(SysError::NotImplemented)
}

/// Espera processo filho terminar
///
/// # Args
/// - pid: PID do processo (0 = qualquer filho)
/// - timeout_ms: timeout em ms (0 = bloqueante infinito)
///
/// # Returns
/// Exit code do processo ou erro
pub fn sys_wait(pid: usize, timeout_ms: u64) -> SysResult<usize> {
    // TODO: Validar que pid é filho do processo atual
    // TODO: Se processo já terminou, retornar exit code
    // TODO: Se não, bloquear até terminar ou timeout

    let _ = (pid, timeout_ms);
    crate::kwarn!("(Syscall) sys_wait não implementado");
    Err(SysError::NotImplemented)
}

/// Cede o restante do quantum
///
/// Sempre retorna 0.
pub fn sys_yield() -> SysResult<usize> {
    crate::sched::scheduler::yield_now();
    Ok(0)
}
