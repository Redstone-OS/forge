//! # Handle Management
//!
//! Handles type-safe com reference counting.

pub mod rights;
pub mod table;

pub use rights::HandleRights;
pub use table::{Handle, HandleEntry, HandleTable, HandleType};

use super::abi::SyscallArgs;
use super::error::{SysError, SysResult};

/// Wrapper para sys_handle_dup
pub fn sys_handle_dup_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_handle_dup(args.arg1 as u32, args.arg2 as u32)
}

/// Wrapper para sys_handle_close
pub fn sys_handle_close_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_handle_close(args.arg1 as u32)
}

/// Wrapper para sys_check_rights
pub fn sys_check_rights_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_check_rights(args.arg1 as u32, args.arg2 as u32)
}

/// Duplica handle com rights reduzidos
pub fn sys_handle_dup(handle: u32, new_rights: u32) -> SysResult<usize> {
    // TODO: Implementar
    let _ = (handle, new_rights);
    Err(SysError::NotImplemented)
}

/// Fecha um handle
pub fn sys_handle_close(handle: u32) -> SysResult<usize> {
    // TODO: Implementar
    let _ = handle;
    Err(SysError::NotImplemented)
}

/// Verifica se handle tem rights específicos
pub fn sys_check_rights(handle: u32, rights: u32) -> SysResult<usize> {
    // TODO: Implementar
    let _ = (handle, rights);
    Err(SysError::NotImplemented)
}
