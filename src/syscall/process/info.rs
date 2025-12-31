//! # Process Information
//!
//! getpid, gettaskinfo

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_gettaskinfo_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_gettaskinfo(args.arg1, args.arg2)
}

// === IMPLEMENTAÇÕES ===

/// Obtém informações sobre uma task
///
/// # Args
/// - pid: PID da task
/// - out_ptr: ponteiro para TaskInfo de saída
///
/// # Returns
/// 0 ou erro
pub fn sys_gettaskinfo(pid: usize, out_ptr: usize) -> SysResult<usize> {
    // TODO: Validar ponteiro
    // TODO: Buscar task por PID
    // TODO: Preencher TaskInfo
    // TODO: copy_to_user

    let _ = (pid, out_ptr);
    Err(SysError::NotImplemented)
}

/// Informações de uma task (para userspace)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskInfo {
    pub pid: u64,
    pub parent_pid: u64,
    pub state: u32,
    pub priority: u32,
    pub cpu_time_ms: u64,
    pub memory_bytes: u64,
}
