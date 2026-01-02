//! # Filesystem Directory Syscalls (0x6C-0x6F)
//!
//! Operações de diretório: getdents, mkdir, rmdir, getcwd

use super::handle::{get_handle, update_dir_index};
use super::types::{DirEntryBuilder, FileType};
use crate::sync::Spinlock;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// CURRENT WORKING DIRECTORY
// =============================================================================

/// Diretório de trabalho atual (global por enquanto)
/// TODO: Mover para estrutura por-processo
static CWD: Spinlock<String> = Spinlock::new(String::new());

/// Inicializa CWD com "/"
pub fn init_cwd() {
    *CWD.lock() = String::from("/");
}

// =============================================================================
// WRAPPERS
// =============================================================================

pub fn sys_getdents_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_getdents(args.arg1 as u32, args.arg2, args.arg3)
}

pub fn sys_mkdir_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_mkdir(args.arg1, args.arg2, args.arg3 as u32)
}

pub fn sys_rmdir_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_rmdir(args.arg1, args.arg2)
}

pub fn sys_getcwd_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_getcwd(args.arg1, args.arg2)
}

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Lista entradas de diretório
///
/// # Args
/// - handle: handle do diretório (aberto com O_DIRECTORY)
/// - buf_ptr: buffer de destino
/// - buf_len: tamanho do buffer
///
/// # Returns
/// Bytes escritos no buffer, ou 0 se não há mais entradas
pub fn sys_getdents(handle: u32, buf_ptr: usize, buf_len: usize) -> SysResult<usize> {
    if buf_ptr == 0 || buf_len == 0 {
        return Err(SysError::InvalidArgument);
    }

    let h = get_handle(handle).ok_or(SysError::InvalidHandle)?;

    if !h.is_directory() {
        return Err(SysError::NotDirectory);
    }

    crate::ktrace!("(FS) sys_getdents:", h.path.as_ptr() as u64);

    // Obter lista de entradas do diretório
    let entries = list_directory(&h.path)?;

    let start_index = h.dir_index;
    if start_index >= entries.len() {
        // Não há mais entradas
        return Ok(0);
    }

    // Buffer de saída
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, buf_len) };
    let mut written = 0;
    let mut current_index = start_index;

    for entry in entries.iter().skip(start_index) {
        let remaining = &mut buf[written..];

        if let Some(entry_len) = DirEntryBuilder::write(
            remaining,
            0, // ino - não temos inode numbers por enquanto
            entry.file_type,
            &entry.name,
        ) {
            written += entry_len;
            current_index += 1;
        } else {
            // Buffer cheio
            break;
        }
    }

    // Atualizar índice no handle
    update_dir_index(handle, current_index);

    crate::ktrace!("(FS) sys_getdents bytes gravados:", written as u64);
    Ok(written)
}

/// Cria um diretório
pub fn sys_mkdir(_path_ptr: usize, _path_len: usize, _mode: u32) -> SysResult<usize> {
    // TODO: Implementar quando tivermos escrita no FAT
    crate::kwarn!("(FS) sys_mkdir: não implementado");
    Err(SysError::NotImplemented)
}

/// Remove um diretório vazio
pub fn sys_rmdir(_path_ptr: usize, _path_len: usize) -> SysResult<usize> {
    // TODO: Implementar quando tivermos escrita no FAT
    crate::kwarn!("(FS) sys_rmdir: não implementado");
    Err(SysError::NotImplemented)
}

/// Obtém diretório de trabalho atual
///
/// # Args
/// - buf_ptr: buffer de destino
/// - buf_len: tamanho do buffer
///
/// # Returns
/// Tamanho do path (incluindo null terminator)
pub fn sys_getcwd(buf_ptr: usize, buf_len: usize) -> SysResult<usize> {
    if buf_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    let cwd = CWD.lock();
    let cwd_bytes = cwd.as_bytes();
    let required_len = cwd_bytes.len() + 1; // +1 para null terminator

    if buf_len < required_len {
        return Err(SysError::BufferTooSmall);
    }

    // Copiar para userspace
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, buf_len) };
    buf[..cwd_bytes.len()].copy_from_slice(cwd_bytes);
    buf[cwd_bytes.len()] = 0; // null terminator

    Ok(required_len)
}

// =============================================================================
// HELPERS
// =============================================================================

/// Entrada de diretório interna
struct DirEntryInfo {
    name: String,
    file_type: FileType,
}

/// Lista conteúdo de um diretório
fn list_directory(path: &str) -> SysResult<Vec<DirEntryInfo>> {
    let normalized = path.trim_start_matches('/');

    // Adicionar . e .. para todos os diretórios
    let mut entries = Vec::new();

    entries.push(DirEntryInfo {
        name: String::from("."),
        file_type: FileType::Directory,
    });

    entries.push(DirEntryInfo {
        name: String::from(".."),
        file_type: FileType::Directory,
    });

    // Diretório raiz
    if normalized.is_empty() {
        let root_dirs = [
            "system",
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

        for name in root_dirs {
            entries.push(DirEntryInfo {
                name: String::from(name),
                file_type: FileType::Directory,
            });
        }

        return Ok(entries);
    }

    // Diretórios conhecidos do sistema
    match normalized {
        "system" => {
            entries.push(DirEntryInfo {
                name: String::from("core"),
                file_type: FileType::Directory,
            });
            entries.push(DirEntryInfo {
                name: String::from("services"),
                file_type: FileType::Directory,
            });
        }
        "system/core" => {
            // Listar do InitRAMFS
            if let Some(supervisor) = crate::fs::initramfs::lookup_file("/system/core/supervisor") {
                let _ = supervisor; // Só verificar que existe
                entries.push(DirEntryInfo {
                    name: String::from("supervisor"),
                    file_type: FileType::Regular,
                });
            }
        }
        "system/services" | "apps" => {
            // Listar do FAT
            let fat_entries = list_fat_directory(normalized);
            entries.extend(fat_entries);
        }
        _ => {
            // Tentar listar do FAT
            let fat_entries = list_fat_directory(normalized);
            if fat_entries.is_empty() {
                return Err(SysError::NotFound);
            }
            entries.extend(fat_entries);
        }
    }

    Ok(entries)
}

/// Lista conteúdo de um diretório no FAT
fn list_fat_directory(path: &str) -> Vec<DirEntryInfo> {
    let mut entries = Vec::new();

    // Usar a função pública de listagem do FAT
    if let Some(dir_entries) = crate::fs::fat::list_directory(path) {
        for entry in dir_entries {
            entries.push(DirEntryInfo {
                name: entry.name,
                file_type: if entry.is_directory {
                    FileType::Directory
                } else {
                    FileType::Regular
                },
            });
        }
    }

    entries
}
