//! # Filesystem Symlink Syscalls (0x74-0x76)
//!
//! Operações com links simbólicos: symlink, readlink, realpath

use super::types::path_from_user;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// WRAPPERS
// =============================================================================

pub fn sys_symlink_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_symlink(args.arg1, args.arg2, args.arg3, args.arg4)
}

pub fn sys_readlink_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_readlink(args.arg1, args.arg2, args.arg3, args.arg4)
}

pub fn sys_realpath_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_realpath(args.arg1, args.arg2, args.arg3, args.arg4)
}

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Cria um link simbólico
///
/// # Args
/// - target_ptr: ponteiro para caminho alvo
/// - target_len: tamanho do caminho alvo
/// - link_ptr: ponteiro para caminho do link
/// - link_len: tamanho do caminho do link
///
/// # Returns
/// 0 ou erro
pub fn sys_symlink(
    _target_ptr: usize,
    _target_len: usize,
    _link_ptr: usize,
    _link_len: usize,
) -> SysResult<usize> {
    // Symlinks não são suportados em FAT
    crate::kwarn!("(FS) sys_symlink: não suportado no FAT");
    Err(SysError::NotSupported)
}

/// Lê o destino de um link simbólico
///
/// # Args
/// - path_ptr: ponteiro para caminho do link
/// - path_len: tamanho do caminho
/// - buf_ptr: buffer para destino
/// - buf_len: tamanho do buffer
///
/// # Returns
/// Tamanho do destino ou erro
pub fn sys_readlink(
    _path_ptr: usize,
    _path_len: usize,
    _buf_ptr: usize,
    _buf_len: usize,
) -> SysResult<usize> {
    // Symlinks não são suportados em FAT
    Err(SysError::NotSupported)
}

/// Resolve caminho para forma canônica
///
/// # Args
/// - path_ptr: ponteiro para caminho
/// - path_len: tamanho do caminho
/// - buf_ptr: buffer para resultado
/// - buf_len: tamanho do buffer
///
/// # Returns
/// Tamanho do caminho resolvido ou erro
pub fn sys_realpath(
    path_ptr: usize,
    path_len: usize,
    buf_ptr: usize,
    buf_len: usize,
) -> SysResult<usize> {
    let path = path_from_user(path_ptr, path_len)?;

    if buf_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    // Por enquanto, apenas normaliza o path (remove //, ., etc)
    let normalized = normalize_path(&path);
    let bytes = normalized.as_bytes();

    if buf_len < bytes.len() + 1 {
        return Err(SysError::BufferTooSmall);
    }

    // Copiar para userspace
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, buf_len) };
    buf[..bytes.len()].copy_from_slice(bytes);
    buf[bytes.len()] = 0;

    Ok(bytes.len() + 1)
}

// =============================================================================
// HELPERS
// =============================================================================

/// Normaliza um path
fn normalize_path(path: &str) -> alloc::string::String {
    use alloc::string::String;
    use alloc::vec::Vec;

    let mut components: Vec<&str> = Vec::new();
    let is_absolute = path.starts_with('/');

    for component in path.split('/') {
        match component {
            "" | "." => continue,
            ".." => {
                if !components.is_empty() {
                    components.pop();
                }
            }
            _ => components.push(component),
        }
    }

    let mut result = String::new();
    if is_absolute {
        result.push('/');
    }
    result.push_str(&components.join("/"));

    if result.is_empty() {
        String::from("/")
    } else {
        result
    }
}
