//! # Filesystem Mount Syscalls (0x77-0x7A)
//!
//! Operações de montagem: mount, umount, statfs, sync

use super::types::{path_from_user, FsStat};
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// WRAPPERS
// =============================================================================

pub fn sys_mount_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_mount(
        args.arg1, args.arg2, args.arg3, args.arg4, args.arg5, args.arg6,
    )
}

pub fn sys_umount_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_umount(args.arg1, args.arg2, args.arg3 as u32)
}

pub fn sys_statfs_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_statfs(args.arg1, args.arg2, args.arg3)
}

pub fn sys_sync_wrapper(_args: &SyscallArgs) -> SysResult<usize> {
    sys_sync()
}

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Monta um filesystem
///
/// # Args
/// - source_ptr: dispositivo de origem
/// - source_len: tamanho
/// - target_ptr: ponto de montagem
/// - target_len: tamanho
/// - fstype_ptr: tipo de filesystem (fat, ext4, etc)
/// - flags: flags de montagem
///
/// # Returns
/// 0 ou erro
pub fn sys_mount(
    _source_ptr: usize,
    _source_len: usize,
    _target_ptr: usize,
    _target_len: usize,
    _fstype_ptr: usize,
    _flags: usize,
) -> SysResult<usize> {
    // TODO: Implementar montagem dinâmica
    // Por enquanto, o FAT é montado automaticamente no boot
    crate::kwarn!("(FS) sys_mount: não implementado");
    Err(SysError::NotImplemented)
}

/// Desmonta um filesystem
///
/// # Args
/// - target_ptr: ponto de montagem
/// - target_len: tamanho
/// - flags: flags
///
/// # Returns
/// 0 ou erro
pub fn sys_umount(_target_ptr: usize, _target_len: usize, _flags: u32) -> SysResult<usize> {
    // TODO: Implementar desmontagem
    crate::kwarn!("(FS) sys_umount: não implementado");
    Err(SysError::NotImplemented)
}

/// Obtém informações do filesystem
///
/// # Args
/// - path_ptr: path dentro do filesystem
/// - path_len: tamanho
/// - statfs_ptr: ponteiro para struct FsStat
///
/// # Returns
/// 0 ou erro
pub fn sys_statfs(path_ptr: usize, path_len: usize, statfs_ptr: usize) -> SysResult<usize> {
    let _path = path_from_user(path_ptr, path_len)?;

    if statfs_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    // Por enquanto, retorna valores placeholder
    // TODO: Obter valores reais do FAT montado
    let stat = FsStat {
        fs_type: 0x4D44, // "MD" - DOS FAT
        block_size: 512,
        total_blocks: 0, // TODO: calcular
        free_blocks: 0,  // TODO: calcular
        total_inodes: 0, // FAT não tem inodes
        free_inodes: 0,
        max_name_len: 255, // LFN support
        _pad: 0,
    };

    // Copiar para userspace
    let dest = unsafe { &mut *(statfs_ptr as *mut FsStat) };
    *dest = stat;

    Ok(0)
}

/// Sincroniza todos os buffers para disco
///
/// # Returns
/// 0 ou erro
pub fn sys_sync() -> SysResult<usize> {
    // Por enquanto, não temos buffering de escrita
    // Quando implementarmos, este syscall forçará flush
    crate::ktrace!("(FS) sys_sync: não implementado");
    Ok(0)
}
