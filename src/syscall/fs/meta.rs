//! # Filesystem Metadata Syscalls (0x68-0x6B)
//!
//! Operações de metadados: stat, fstat, chmod, chown

use super::handle::get_handle;
use super::types::{path_from_user, FileStat, FileType};
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// WRAPPERS
// =============================================================================

pub fn sys_stat_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_stat(args.arg1, args.arg2, args.arg3)
}

pub fn sys_fstat_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_fstat(args.arg1 as u32, args.arg2)
}

pub fn sys_chmod_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_chmod(args.arg1, args.arg2, args.arg3 as u32)
}

pub fn sys_chown_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_chown(args.arg1, args.arg2, args.arg3 as u32, args.arg4 as u32)
}

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Obtém informações de arquivo por caminho
///
/// # Args
/// - path_ptr: ponteiro para caminho
/// - path_len: tamanho do caminho
/// - stat_ptr: ponteiro para struct FileStat de saída
///
/// # Returns
/// 0 ou erro
pub fn sys_stat(path_ptr: usize, path_len: usize, stat_ptr: usize) -> SysResult<usize> {
    let path = path_from_user(path_ptr, path_len)?;

    if stat_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    crate::ktrace!("(FS) sys_stat:", path.as_ptr() as u64);

    // Verificar se é diretório conhecido
    let normalized = path.trim_start_matches('/');
    let known_dirs = [
        "",
        "system",
        "system/services",
        "system/core",
        "apps",
        "users",
        "devices",
        "volumes",
        "runtime",
        "state",
        "data",
        "net",
        "snapshots",
        "boot",
    ];

    let stat = if known_dirs.contains(&normalized) {
        // É um diretório
        FileStat {
            file_type: FileType::Directory as u8,
            mode: 0o755,
            _pad: 0,
            size: 0,
            nlink: 2,
            uid: 0,
            gid: 0,
            _pad2: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    } else if let Some(data) = crate::fs::vfs::read_file(&path) {
        // É um arquivo
        FileStat {
            file_type: FileType::Regular as u8,
            mode: 0o644,
            _pad: 0,
            size: data.len() as u64,
            nlink: 1,
            uid: 0,
            gid: 0,
            _pad2: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    } else {
        return Err(SysError::NotFound);
    };

    // Copiar para userspace
    let dest = unsafe { &mut *(stat_ptr as *mut FileStat) };
    *dest = stat;

    Ok(0)
}

/// Obtém informações de arquivo por handle
///
/// # Args
/// - handle: handle do arquivo
/// - stat_ptr: ponteiro para struct FileStat de saída
///
/// # Returns
/// 0 ou erro
pub fn sys_fstat(handle: u32, stat_ptr: usize) -> SysResult<usize> {
    if stat_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    let h = get_handle(handle).ok_or(SysError::InvalidHandle)?;

    let stat = FileStat {
        file_type: h.file_type as u8,
        mode: if h.is_directory() { 0o755 } else { 0o644 },
        _pad: 0,
        size: h.size,
        nlink: if h.is_directory() { 2 } else { 1 },
        uid: 0,
        gid: 0,
        _pad2: 0,
        atime: 0,
        mtime: 0,
        ctime: 0,
    };

    // Copiar para userspace
    let dest = unsafe { &mut *(stat_ptr as *mut FileStat) };
    *dest = stat;

    Ok(0)
}

/// Altera permissões de arquivo
pub fn sys_chmod(_path_ptr: usize, _path_len: usize, _mode: u32) -> SysResult<usize> {
    // TODO: Implementar quando tivermos sistema de permissões
    crate::kwarn!("(FS) sys_chmod: não implementado");
    Err(SysError::NotImplemented)
}

/// Altera dono/grupo de arquivo
pub fn sys_chown(_path_ptr: usize, _path_len: usize, _uid: u32, _gid: u32) -> SysResult<usize> {
    // TODO: Implementar quando tivermos sistema de usuários
    crate::kwarn!("(FS) sys_chown: não implementado");
    Err(SysError::NotImplemented)
}
