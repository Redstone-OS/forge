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
pub fn sys_handle_dup(handle_val: u32, new_rights: u32) -> SysResult<usize> {
    let handle = Handle::new((handle_val & 0xFFFF) as u16, (handle_val >> 16) as u16);
    let rights = HandleRights::from_bits_truncate(new_rights as u64);

    crate::sched::core::with_current_mut(|task| {
        if let Some(new_handle) = task.handle_table.dup(handle, rights) {
            Ok(new_handle.as_u32() as usize)
        } else {
            Err(SysError::InvalidHandle)
        }
    })
    .unwrap_or(Err(SysError::Interrupted))
}

/// Fecha um handle
pub fn sys_handle_close(handle_val: u32) -> SysResult<usize> {
    let handle = Handle::new((handle_val & 0xFFFF) as u16, (handle_val >> 16) as u16);

    crate::sched::core::with_current_mut(|task| {
        if task.handle_table.close(handle) {
            Ok(0)
        } else {
            Err(SysError::InvalidHandle)
        }
    })
    .unwrap_or(Err(SysError::Interrupted))
}

/// Verifica se handle tem rights específicos
pub fn sys_check_rights(handle: u32, rights: u32) -> SysResult<usize> {
    let _ = (handle, rights);
    Err(SysError::NotImplemented)
}
