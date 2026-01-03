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

pub fn sys_getpid_wrapper(_args: &SyscallArgs) -> SysResult<usize> {
    sys_getpid()
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
    crate::sched::core::exit_current(code)
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

    // Copiar path do userspace de forma segura (copia os bytes para o kernel heap)
    let path = match crate::syscall::fs::types::path_from_user(path_ptr, path_len) {
        Ok(s) => s,
        Err(e) => {
            crate::kerror!("(Syscall) sys_spawn: falha ao ler path do user");
            return Err(e);
        }
    };

    // Obter PID do chamador para definir como pai
    let current_tid = {
        let guard = crate::sched::core::CURRENT.lock();
        guard.as_ref().map(|t| t.tid)
    };

    // Chamar função de spawn existente
    match crate::sched::exec::spawn(&path, current_tid) {
        Ok(pid) => {
            crate::kinfo!("(Syscall) spawn OK, PID=", pid.as_u32() as u64);
            Ok(pid.as_u32() as usize)
        }
        Err(e) => {
            crate::kerror!("(Syscall) spawn falhou");
            // Converter ExecError para SysError
            match e {
                crate::sched::ExecError::NotFound => Err(SysError::NotFound),
                crate::sched::ExecError::InvalidFormat => Err(SysError::InvalidArgument),
                crate::sched::ExecError::OutOfMemory => Err(SysError::OutOfMemory),
                crate::sched::ExecError::PermissionDenied => Err(SysError::PermissionDenied),
            }
        }
    }
}

/// Espera processo filho terminar
///
/// # Args
/// - pid: PID do processo (0 = qualquer filho - NOT YET SUPPORTED)
/// - timeout_ms: timeout em ms (0 = bloqueante infinito - NOT YET SUPPORTED)
///
/// # Returns
/// Exit code do processo ou erro
pub fn sys_wait(pid: usize, _timeout_ms: u64) -> SysResult<usize> {
    if pid == 0 {
        crate::kwarn!("(Syscall) sys_wait(0) não implementado (any child)");
        return Err(SysError::NotImplemented);
    }

    let tid = crate::sys::types::Tid::new(pid as u32);

    // Tenta coletar o zumbi
    if let Some(code) = crate::sched::task::lifecycle::find_and_collect_zombie(tid) {
        Ok(code as usize)
    } else {
        // Se não for zumbi, pode estar rodando ou não existir
        // TODO: Verificar se processo existe. Se existir, deveria bloquear.
        // Por enquanto, Supervisor usa timeout e poll, então retornamos NotFound se não for zumbi.
        Err(SysError::NotFound)
    }
}

/// Cede o restante do quantum
///
/// Sempre retorna 0.
pub fn sys_yield() -> SysResult<usize> {
    crate::sched::core::yield_now();
    Ok(0)
}

/// Obtém o PID do processo atual
pub fn sys_getpid() -> SysResult<usize> {
    let task_guard = crate::sched::core::CURRENT.lock();
    if let Some(task) = task_guard.as_ref() {
        Ok(task.tid.as_u32() as usize)
    } else {
        Err(SysError::Interrupted)
    }
}

/// Obtém o TID da thread atual
pub fn sys_gettid() -> SysResult<usize> {
    let task_guard = crate::sched::core::CURRENT.lock();
    if let Some(task) = task_guard.as_ref() {
        Ok(task.tid.as_u32() as usize)
    } else {
        Err(SysError::Interrupted)
    }
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
