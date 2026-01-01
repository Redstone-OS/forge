//! # Memory Advise Syscall
//!
//! sys_madvise - dicas de uso de memória

use crate::syscall::SysResult;

/// Normal usage
pub const MADV_NORMAL: i32 = 0;
/// Expect random access
pub const MADV_RANDOM: i32 = 1;
/// Expect sequential access
pub const MADV_SEQUENTIAL: i32 = 2;
/// Will need this soon
pub const MADV_WILLNEED: i32 = 3;
/// Don't need this anymore
pub const MADV_DONTNEED: i32 = 4;
/// Free pages immediately
pub const MADV_FREE: i32 = 8;

/// sys_madvise(addr, size, advice) -> Result<()>
pub fn sys_madvise(addr: usize, size: usize, advice: i32) -> SysResult<usize> {
    if size == 0 {
        return Err(crate::syscall::SysError::InvalidArgument);
    }

    match advice {
        MADV_NORMAL | MADV_RANDOM | MADV_SEQUENTIAL => {
            // Apenas hints, não precisam fazer nada agora
        }
        MADV_WILLNEED => {
            // TODO: Prefetch pages
        }
        MADV_DONTNEED => {
            // TODO: Marcar páginas como descartáveis
        }
        MADV_FREE => {
            // TODO: Liberar páginas imediatamente
        }
        _ => return Err(crate::syscall::SysError::InvalidArgument),
    }

    let _ = addr;
    Ok(0)
}
