//! # Filesystem File Manipulation Syscalls (0x70-0x73)
//!
//! Operações de manipulação: create, unlink, rename, link

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// WRAPPERS
// =============================================================================

pub fn sys_create_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_create(args.arg1, args.arg2, args.arg3 as u32)
}

pub fn sys_unlink_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_unlink(args.arg1, args.arg2)
}

pub fn sys_rename_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_rename(args.arg1, args.arg2, args.arg3, args.arg4)
}

pub fn sys_link_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_link(args.arg1, args.arg2, args.arg3, args.arg4)
}

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Cria um arquivo vazio
///
/// # Args
/// - path_ptr: ponteiro para caminho
/// - path_len: tamanho do caminho
/// - mode: permissões
///
/// # Returns
/// 0 ou erro
pub fn sys_create(_path_ptr: usize, _path_len: usize, _mode: u32) -> SysResult<usize> {
    // TODO: Implementar quando tivermos escrita no FAT
    crate::kwarn!("(FS) sys_create: não implementado");
    Err(SysError::NotImplemented)
}

/// Remove um arquivo
///
/// # Args
/// - path_ptr: ponteiro para caminho
/// - path_len: tamanho do caminho
///
/// # Returns
/// 0 ou erro
pub fn sys_unlink(_path_ptr: usize, _path_len: usize) -> SysResult<usize> {
    // TODO: Implementar quando tivermos escrita no FAT
    crate::kwarn!("(FS) sys_unlink: não implementado");
    Err(SysError::NotImplemented)
}

/// Renomeia ou move arquivo/diretório
///
/// # Args
/// - old_ptr: ponteiro para caminho antigo
/// - old_len: tamanho do caminho antigo
/// - new_ptr: ponteiro para caminho novo
/// - new_len: tamanho do caminho novo
///
/// # Returns
/// 0 ou erro
pub fn sys_rename(
    _old_ptr: usize,
    _old_len: usize,
    _new_ptr: usize,
    _new_len: usize,
) -> SysResult<usize> {
    // TODO: Implementar quando tivermos escrita no FAT
    crate::kwarn!("(FS) sys_rename: não implementado");
    Err(SysError::NotImplemented)
}

/// Cria um hard link
///
/// # Args
/// - target_ptr: ponteiro para caminho alvo
/// - target_len: tamanho do caminho alvo
/// - link_ptr: ponteiro para caminho do link
/// - link_len: tamanho do caminho do link
///
/// # Returns
/// 0 ou erro
pub fn sys_link(
    _target_ptr: usize,
    _target_len: usize,
    _link_ptr: usize,
    _link_len: usize,
) -> SysResult<usize> {
    // Hard links não são suportados em FAT
    crate::kwarn!("(FS) sys_link: não suportado no FAT");
    Err(SysError::NotSupported)
}
