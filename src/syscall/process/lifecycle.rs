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

pub fn sys_gettid_wrapper(_args: &SyscallArgs) -> SysResult<usize> {
    sys_gettid()
}

pub fn sys_thread_create_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_thread_create(args.arg1, args.arg2, args.arg3)
}

pub fn sys_thread_exit_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_thread_exit(args.arg1 as i32)
}

// === IMPLEMENTAÇÕES ===

/// Encerra o processo atual
///
/// Nunca retorna.
pub fn sys_exit(code: i32) -> ! {
    crate::kinfo!("(Syscall) sys_exit code=", code as u64);

    // Chamar exit_current do scheduler que remove o processo
    // e pula para o próximo sem reenfileirar
    crate::sched::scheduler::exit_current()
}

/// Cria novo processo
///
/// # Args
/// - path_ptr: caminho do executável (userspace)
/// - path_len: tamanho do caminho
/// - args_ptr: argumentos (ignorado por enquanto)
/// - args_len: número de argumentos (ignorado por enquanto)
///
/// # Returns
/// PID do novo processo ou erro
pub fn sys_spawn(
    path_ptr: usize,
    path_len: usize,
    _args_ptr: usize,
    _args_len: usize,
) -> SysResult<usize> {
    // Validar ponteiros básicos
    if path_ptr == 0 || path_len == 0 || path_len > 256 {
        crate::kerror!("(Syscall) sys_spawn: path inválido");
        return Err(SysError::InvalidArgument);
    }

    // Ler path do userspace
    // TODO: Validar que ponteiro está em região de usuário
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };

    // Converter para &str
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => {
            crate::kerror!("(Syscall) sys_spawn: path não é UTF-8 válido");
            return Err(SysError::InvalidArgument);
        }
    };

    crate::kinfo!("(Syscall) sys_spawn:", path_ptr as u64);

    // Chamar função de spawn existente
    match crate::sched::exec::spawn::spawn::spawn(path) {
        Ok(pid) => {
            crate::kinfo!("(Syscall) spawn OK, PID=", pid.as_u32() as u64);
            Ok(pid.as_u32() as usize)
        }
        Err(e) => {
            crate::kerror!("(Syscall) spawn falhou");
            // Converter ExecError para SysError
            match e {
                crate::sched::exec::spawn::spawn::ExecError::NotFound => Err(SysError::NotFound),
                crate::sched::exec::spawn::spawn::ExecError::InvalidFormat => {
                    Err(SysError::InvalidArgument)
                }
                crate::sched::exec::spawn::spawn::ExecError::OutOfMemory => {
                    Err(SysError::OutOfMemory)
                }
                crate::sched::exec::spawn::spawn::ExecError::PermissionDenied => {
                    Err(SysError::PermissionDenied)
                }
            }
        }
    }
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

/// Obtém o TID da thread atual
pub fn sys_gettid() -> SysResult<usize> {
    // TODO: Retornar TID real do scheduler
    Ok(0)
}

/// Cria uma nova thread no processo atual
pub fn sys_thread_create(entry_ptr: usize, stack_ptr: usize, arg: usize) -> SysResult<usize> {
    let _ = (entry_ptr, stack_ptr, arg);
    crate::kwarn!("(Syscall) sys_thread_create não implementado");
    Err(SysError::NotImplemented)
}

/// Encerra a thread atual
pub fn sys_thread_exit(code: i32) -> SysResult<usize> {
    crate::kinfo!("(Syscall) sys_thread_exit code=", code as u64);
    // TODO: Implementar encerramento de thread no scheduler
    loop {
        let _ = sys_yield();
    }
}
