//! # VMO Syscalls
//!
//! Virtual Memory Object - objetos de memória compartilháveis

use crate::syscall::SysResult;

/// sys_vmo_create(size) -> Result<handle>
pub fn sys_vmo_create(size: usize) -> SysResult<usize> {
    if size == 0 {
        return Err(crate::syscall::SysError::InvalidArgument);
    }
    // TODO: Criar VMO e retornar handle
    let _ = size;
    Err(crate::syscall::SysError::NotSupported)
}

/// sys_vmo_read(handle, offset, buf, len) -> Result<bytes_read>
pub fn sys_vmo_read(handle: u64, offset: u64, buf: usize, len: usize) -> SysResult<usize> {
    let _ = (handle, offset, buf, len);
    Err(crate::syscall::SysError::NotSupported)
}

/// sys_vmo_write(handle, offset, buf, len) -> Result<bytes_written>
pub fn sys_vmo_write(handle: u64, offset: u64, buf: usize, len: usize) -> SysResult<usize> {
    let _ = (handle, offset, buf, len);
    Err(crate::syscall::SysError::NotSupported)
}

/// sys_vmo_map(handle, addr, offset, len, flags) -> Result<mapped_addr>
pub fn sys_vmo_map(
    handle: u64,
    addr: usize,
    offset: u64,
    len: usize,
    flags: u32,
) -> SysResult<usize> {
    let _ = (handle, addr, offset, len, flags);
    Err(crate::syscall::SysError::NotSupported)
}

/// sys_vmo_unmap(addr, len) -> Result<()>
pub fn sys_vmo_unmap(addr: usize, len: usize) -> SysResult<usize> {
    let _ = (addr, len);
    Err(crate::syscall::SysError::NotSupported)
}
