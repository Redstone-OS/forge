//! # Filesystem I/O Syscalls (0x60-0x67)
//!
//! Operações básicas de I/O: open, read, write, seek, pread, pwrite, flush, truncate

use super::handle::{alloc_handle, get_handle, update_offset, FileHandle};
use super::types::{path_from_user, FileType, OpenFlags, SeekWhence};
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// WRAPPERS
// =============================================================================

pub fn sys_open_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_open(args.arg1, args.arg2, args.arg3 as u32, args.arg4 as u32)
}

pub fn sys_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_read(args.arg1 as u32, args.arg2, args.arg3)
}

pub fn sys_write_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_write(args.arg1 as u32, args.arg2, args.arg3)
}

pub fn sys_seek_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_seek(args.arg1 as u32, args.arg2 as i64, args.arg3 as u32)
}

pub fn sys_pread_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_pread(args.arg1 as u32, args.arg2, args.arg3, args.arg4 as u64)
}

pub fn sys_pwrite_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_pwrite(args.arg1 as u32, args.arg2, args.arg3, args.arg4 as u64)
}

pub fn sys_flush_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_flush(args.arg1 as u32)
}

pub fn sys_truncate_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_truncate(args.arg1 as u32, args.arg2 as u64)
}

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Abre um arquivo ou diretório
///
/// # Args
/// - path_ptr: ponteiro para caminho
/// - path_len: tamanho do caminho
/// - flags: flags de abertura (O_RDONLY, O_WRONLY, O_RDWR, O_CREATE, etc)
/// - mode: permissões para criação (ignorado por enquanto)
///
/// # Returns
/// Handle do arquivo ou erro
pub fn sys_open(path_ptr: usize, path_len: usize, flags: u32, _mode: u32) -> SysResult<usize> {
    let path = path_from_user(path_ptr, path_len)?;
    let flags = OpenFlags(flags);

    crate::ktrace!("(FS) sys_open:", path.as_str());

    // Tentar encontrar o arquivo/diretório
    // Primeiro: tentar via VFS (que roteia para InitRAMFS ou FAT)

    // Verificar se é diretório
    if flags.is_directory() {
        // Abrir diretório para listagem
        if let Some(dir_info) = lookup_directory(&path) {
            let handle = FileHandle::new(
                path.clone(),
                FileType::Directory,
                flags,
                0,
                dir_info.first_cluster,
            );
            let id = alloc_handle(handle);
            crate::ktrace!("(FS) sys_open: abriu diretório, handle:", id as u64);
            return Ok(id as usize);
        }
        return Err(SysError::NotFound);
    }

    // Abrir arquivo regular
    if let Some(file_info) = lookup_file(&path) {
        let handle = FileHandle::new(
            path.clone(),
            FileType::Regular,
            flags,
            file_info.size,
            file_info.first_cluster,
        );
        let id = alloc_handle(handle);
        crate::ktrace!("(FS) sys_open: abriu arquivo, handle:", id as u64);
        return Ok(id as usize);
    }

    // Arquivo não encontrado
    crate::kwarn!("(FS) sys_open: não encontrado");
    Err(SysError::NotFound)
}

/// Lê dados de um arquivo
///
/// # Args
/// - handle: handle do arquivo
/// - buf_ptr: buffer de destino
/// - len: tamanho máximo a ler
///
/// # Returns
/// Bytes lidos ou erro
pub fn sys_read(handle: u32, buf_ptr: usize, len: usize) -> SysResult<usize> {
    if buf_ptr == 0 || len == 0 {
        return Err(SysError::InvalidArgument);
    }

    let h = get_handle(handle).ok_or(SysError::InvalidHandle)?;

    if !h.can_read() {
        return Err(SysError::PermissionDenied);
    }

    if h.is_directory() {
        return Err(SysError::IsDirectory);
    }

    // Ler do arquivo
    let offset = h.offset;
    let bytes_read = read_file_data(&h.path, h.first_cluster, h.size, offset, buf_ptr, len)?;

    // Atualizar offset
    update_offset(handle, offset + bytes_read as u64);

    Ok(bytes_read)
}

/// Escreve dados em um arquivo
///
/// # Args
/// - handle: handle do arquivo
/// - buf_ptr: buffer de origem
/// - len: tamanho a escrever
///
/// # Returns
/// Bytes escritos ou erro
pub fn sys_write(handle: u32, _buf_ptr: usize, _len: usize) -> SysResult<usize> {
    let h = get_handle(handle).ok_or(SysError::InvalidHandle)?;

    if !h.can_write() {
        return Err(SysError::PermissionDenied);
    }

    // TODO: Implementar escrita no FAT
    crate::kwarn!("(FS) sys_write: não implementado");
    Err(SysError::NotImplemented)
}

/// Move posição de leitura/escrita
///
/// # Args
/// - handle: handle do arquivo
/// - offset: deslocamento
/// - whence: referência (SET=0, CUR=1, END=2)
///
/// # Returns
/// Nova posição ou erro
pub fn sys_seek(handle: u32, offset: i64, whence: u32) -> SysResult<usize> {
    let h = get_handle(handle).ok_or(SysError::InvalidHandle)?;
    let whence = SeekWhence::from_u32(whence).ok_or(SysError::InvalidArgument)?;

    let new_offset = match whence {
        SeekWhence::Set => {
            if offset < 0 {
                return Err(SysError::InvalidArgument);
            }
            offset as u64
        }
        SeekWhence::Cur => {
            let cur = h.offset as i64;
            let new = cur + offset;
            if new < 0 {
                return Err(SysError::InvalidArgument);
            }
            new as u64
        }
        SeekWhence::End => {
            let end = h.size as i64;
            let new = end + offset;
            if new < 0 {
                return Err(SysError::InvalidArgument);
            }
            new as u64
        }
    };

    update_offset(handle, new_offset);
    Ok(new_offset as usize)
}

/// Lê em offset específico (sem mover cursor)
pub fn sys_pread(handle: u32, buf_ptr: usize, len: usize, offset: u64) -> SysResult<usize> {
    if buf_ptr == 0 || len == 0 {
        return Err(SysError::InvalidArgument);
    }

    let h = get_handle(handle).ok_or(SysError::InvalidHandle)?;

    if !h.can_read() {
        return Err(SysError::PermissionDenied);
    }

    if h.is_directory() {
        return Err(SysError::IsDirectory);
    }

    // Ler sem atualizar offset
    read_file_data(&h.path, h.first_cluster, h.size, offset, buf_ptr, len)
}

/// Escreve em offset específico (sem mover cursor)
pub fn sys_pwrite(_handle: u32, _buf_ptr: usize, _len: usize, _offset: u64) -> SysResult<usize> {
    // TODO: Implementar escrita
    crate::kwarn!("(FS) sys_pwrite: não implementado");
    Err(SysError::NotImplemented)
}

/// Força flush de buffers para disco
pub fn sys_flush(_handle: u32) -> SysResult<usize> {
    // Por enquanto, não temos buffering
    Ok(0)
}

/// Redimensiona arquivo
pub fn sys_truncate(_handle: u32, _new_size: u64) -> SysResult<usize> {
    // TODO: Implementar truncate
    crate::kwarn!("(FS) sys_truncate: não implementado");
    Err(SysError::NotImplemented)
}

// =============================================================================
// HELPERS - INTEGRAÇÃO COM VFS/FAT
// =============================================================================

/// Info retornada por lookup
struct FileInfo {
    size: u64,
    first_cluster: u32,
}

struct DirInfo {
    first_cluster: u32,
}

/// Busca um arquivo pelo path
fn lookup_file(path: &str) -> Option<FileInfo> {
    // Tentar via FAT
    // Por enquanto, lemos o arquivo inteiro para obter o tamanho
    // TODO: Implementar stat sem ler todo o conteúdo

    if let Some(data) = crate::fs::vfs::read_file(path) {
        // Arquivo encontrado - precisamos do first_cluster
        // Por enquanto, usamos 0 e lemos via path
        Some(FileInfo {
            size: data.len() as u64,
            first_cluster: 0, // Placeholder - lemos via path
        })
    } else {
        None
    }
}

/// Busca um diretório pelo path
fn lookup_directory(path: &str) -> Option<DirInfo> {
    // Verificar se é um diretório válido
    let normalized = path.trim_start_matches('/');

    // Diretórios raiz conhecidos (cache para evitar I/O desnecessário)
    // Estes diretórios sempre existem e não precisam de verificação no FAT
    let known_dirs = [
        "", // raiz
        "system",
        "system/services",
        "system/core",
        "apps",
        "apps/system",
        "apps/system/terminal",
        "apps/system/index",
        "users",
        "devices",
        "volumes",
        "runtime",
        "state",
        "state/db",
        "state/indexes",
        "state/indexes/apps",
        "state/system-state",
        "data",
        "net",
        "snapshots",
        "boot",
    ];

    // Verificar case-insensitive na cache
    let normalized_upper = normalized.to_uppercase();
    for known in known_dirs.iter() {
        if known.to_uppercase() == normalized_upper {
            return Some(DirInfo { first_cluster: 0 });
        }
    }

    // Não está na cache - tentar via FAT
    // Isso permite acessar qualquer diretório válido no disco
    if let Some(_entries) = crate::fs::fat::list_directory(normalized) {
        // FAT encontrou o diretório - é válido
        crate::ktrace!("(FS) lookup_directory via FAT ok");
        return Some(DirInfo { first_cluster: 0 });
    }

    crate::ktrace!("(FS) lookup_directory: not found");
    None
}

/// Lê dados de um arquivo
fn read_file_data(
    path: &str,
    _first_cluster: u32,
    file_size: u64,
    offset: u64,
    buf_ptr: usize,
    len: usize,
) -> SysResult<usize> {
    // Por enquanto, lemos o arquivo inteiro e copiamos a porção desejada
    // TODO: Otimizar para ler apenas os clusters necessários

    if offset >= file_size {
        return Ok(0); // EOF
    }

    let data = crate::fs::vfs::read_file(path).ok_or(SysError::IoError)?;

    let start = offset as usize;
    let available = data.len().saturating_sub(start);
    let to_copy = len.min(available);

    if to_copy == 0 {
        return Ok(0);
    }

    // Copiar para userspace
    // TODO: Proper copy_to_user
    let dest = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, to_copy) };
    dest.copy_from_slice(&data[start..start + to_copy]);

    Ok(to_copy)
}
