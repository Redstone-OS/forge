//! # Poll Syscall
//!
//! Multiplexação de I/O.

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_poll_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_poll(args.arg1, args.arg2, args.arg3 as i64)
}

// === IMPLEMENTAÇÕES ===

/// Espera eventos em múltiplos handles
///
/// # Args
/// - fds_ptr: ponteiro para array de PollFd
/// - nfds: número de entradas no array
/// - timeout_ms: timeout em ms (-1 = infinito, 0 = não bloqueia)
///
/// # Returns
/// Número de handles com eventos ou erro
pub fn sys_poll(fds_ptr: usize, nfds: usize, timeout_ms: i64) -> SysResult<usize> {
    // TODO: Validar ponteiro (array de PollFd)
    // TODO: Para cada fd, verificar se eventos solicitados ocorreram
    // TODO: Se nenhum evento e timeout != 0, bloquear
    // TODO: Preencher revents em cada PollFd
    // TODO: Retornar número de handles com eventos

    let _ = (fds_ptr, nfds, timeout_ms);
    crate::kwarn!("(Syscall) sys_poll não implementado");
    Err(SysError::NotImplemented)
}
