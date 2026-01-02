//! # Filesystem Control Syscalls (0x7B-0x7F)
//!
//! Operações avançadas: ioctl, fcntl, flock, access, chdir

use super::handle::get_handle;
use super::types::path_from_user;
use crate::sync::Spinlock;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};
use alloc::string::String;

// =============================================================================
// CURRENT WORKING DIRECTORY (shared with dir.rs)
// =============================================================================

/// Diretório de trabalho atual
/// TODO: Mover para estrutura por-processo
static CWD: Spinlock<String> = Spinlock::new(String::new());

// =============================================================================
// WRAPPERS
// =============================================================================

pub fn sys_ioctl_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_ioctl(args.arg1 as u32, args.arg2 as u32, args.arg3)
}

pub fn sys_fcntl_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_fcntl(args.arg1 as u32, args.arg2 as u32, args.arg3)
}

pub fn sys_flock_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_flock(args.arg1 as u32, args.arg2 as u32)
}

pub fn sys_access_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_access(args.arg1, args.arg2, args.arg3 as u32)
}

pub fn sys_chdir_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_chdir(args.arg1, args.arg2)
}

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Controle de dispositivo
///
/// # Args
/// - handle: handle do arquivo/dispositivo
/// - cmd: comando
/// - arg_ptr: argumento
///
/// # Returns
/// Depende do comando
pub fn sys_ioctl(_handle: u32, _cmd: u32, _arg_ptr: usize) -> SysResult<usize> {
    // TODO: Implementar para /devices/*
    crate::kwarn!("(FS) sys_ioctl: não implementado");
    Err(SysError::NotImplemented)
}

/// Controle de handle
///
/// # Args
/// - handle: handle do arquivo
/// - cmd: comando (F_GETFL, F_SETFL, F_DUPFD, etc)
/// - arg: argumento
///
/// # Returns
/// Depende do comando
pub fn sys_fcntl(handle: u32, cmd: u32, _arg: usize) -> SysResult<usize> {
    // Comandos básicos
    const F_GETFL: u32 = 1;
    const F_SETFL: u32 = 2;
    const F_GETFD: u32 = 3;
    const F_SETFD: u32 = 4;

    let h = get_handle(handle).ok_or(SysError::InvalidHandle)?;

    match cmd {
        F_GETFL => {
            // Retorna flags de abertura
            Ok(h.flags.0 as usize)
        }
        F_SETFL => {
            // TODO: Atualizar flags
            crate::kwarn!("(FS) fcntl F_SETFL: não implementado");
            Err(SysError::NotImplemented)
        }
        F_GETFD => {
            // FD flags (close-on-exec, etc)
            Ok(0)
        }
        F_SETFD => {
            // TODO: Implementar close-on-exec
            Ok(0)
        }
        _ => {
            crate::kwarn!("(FS) fcntl: comando desconhecido:", cmd as u64);
            Err(SysError::InvalidArgument)
        }
    }
}

/// Lock de arquivo
///
/// # Args
/// - handle: handle do arquivo
/// - operation: LOCK_SH, LOCK_EX, LOCK_UN, LOCK_NB
///
/// # Returns
/// 0 ou erro
pub fn sys_flock(_handle: u32, _operation: u32) -> SysResult<usize> {
    // TODO: Implementar file locking
    // Por enquanto, fingimos que funcionou (single-user system)
    Ok(0)
}

/// Verifica permissões de acesso
///
/// # Args
/// - path_ptr: caminho
/// - path_len: tamanho
/// - mode: R_OK=4, W_OK=2, X_OK=1, F_OK=0
///
/// # Returns
/// 0 se permitido, erro caso contrário
pub fn sys_access(path_ptr: usize, path_len: usize, mode: u32) -> SysResult<usize> {
    let path = path_from_user(path_ptr, path_len)?;

    const F_OK: u32 = 0;
    const X_OK: u32 = 1;
    const W_OK: u32 = 2;
    const R_OK: u32 = 4;

    // Verificar existência
    let exists = check_exists(&path);

    if !exists {
        return Err(SysError::NotFound);
    }

    if mode == F_OK {
        return Ok(0);
    }

    // Por enquanto, assumimos permissão total para arquivos existentes
    // TODO: Implementar checagem real de permissões

    if (mode & W_OK) != 0 {
        // Verificar se é read-only (FAT pode ter flag RO)
        // Por enquanto, retornamos PermissionDenied para /system/*
        if path.starts_with("/system") {
            return Err(SysError::PermissionDenied);
        }
    }

    Ok(0)
}

/// Altera diretório de trabalho
///
/// # Args
/// - path_ptr: novo diretório
/// - path_len: tamanho
///
/// # Returns
/// 0 ou erro
pub fn sys_chdir(path_ptr: usize, path_len: usize) -> SysResult<usize> {
    let path = path_from_user(path_ptr, path_len)?;

    crate::ktrace!("(FS) sys_chdir:", path.as_ptr() as u64);

    // Verificar se é um diretório válido
    if !is_valid_directory(&path) {
        return Err(SysError::NotDirectory);
    }

    // Normalizar o path
    let normalized = normalize_path(&path);

    // Atualizar CWD
    *CWD.lock() = normalized;

    Ok(0)
}

// =============================================================================
// HELPERS
// =============================================================================

/// Verifica se um path existe
fn check_exists(path: &str) -> bool {
    let normalized = path.trim_start_matches('/');

    // Diretórios conhecidos
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

    if known_dirs.contains(&normalized) {
        return true;
    }

    // Tentar como arquivo
    crate::fs::vfs::read_file(path).is_some()
}

/// Verifica se é um diretório válido
fn is_valid_directory(path: &str) -> bool {
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

    if known_dirs.contains(&normalized) {
        return true;
    }

    // TODO: Verificar no FAT se é diretório
    false
}

/// Normaliza um path
fn normalize_path(path: &str) -> String {
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

    for (i, comp) in components.iter().enumerate() {
        if i > 0 {
            result.push('/');
        }
        result.push_str(comp);
    }

    if result.is_empty() {
        String::from("/")
    } else {
        result
    }
}
