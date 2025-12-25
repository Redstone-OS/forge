//! Syscalls de Memória
//!
//! Alocação e mapeamento de memória virtual.

use super::abi::map_flags;
use super::error::{SysError, SysResult};

/// Aloca memória virtual.
///
/// # Syscall
/// `SYS_ALLOC (0x10)` - Args: (size, flags)
///
/// # Argumentos
/// - `size`: Tamanho em bytes (será arredondado para páginas)
/// - `flags`: map_flags (READ, WRITE, EXEC)
///
/// # Retorno
/// Endereço da região alocada
pub fn sys_alloc(size: usize, flags: usize) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Arredondar para páginas
    let page_size = 4096usize;
    let pages = (size + page_size - 1) / page_size;

    let _flags = flags as u32;

    // TODO: Usar VMM para alocar páginas no espaço do processo
    // TODO: Mapear com permissões corretas

    crate::kwarn!(
        "[Syscall] alloc({} bytes, {} páginas) não implementado",
        size,
        pages
    );
    Err(SysError::NotImplemented)
}

/// Libera memória alocada.
///
/// # Syscall
/// `SYS_FREE (0x11)` - Args: (addr, size)
///
/// # Argumentos
/// - `addr`: Endereço retornado por SYS_ALLOC
/// - `size`: Tamanho original
pub fn sys_free(addr: usize, size: usize) -> SysResult<usize> {
    if addr == 0 {
        return Err(SysError::BadAddress);
    }

    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Verificar alinhamento
    if addr % 4096 != 0 {
        return Err(SysError::BadAlignment);
    }

    // TODO: Usar VMM para desmapear e liberar páginas

    crate::kwarn!("[Syscall] free({:#x}, {}) não implementado", addr, size);
    Err(SysError::NotImplemented)
}

/// Mapeia região de memória ou handle.
///
/// # Syscall
/// `SYS_MAP (0x12)` - Args: (addr, size, flags, handle)
///
/// # Argumentos
/// - `addr`: Endereço desejado (0 = kernel escolhe)
/// - `size`: Tamanho do mapeamento
/// - `flags`: map_flags
/// - `handle`: Handle de memória/arquivo (0 = anônimo)
///
/// # Retorno
/// Endereço do mapeamento
pub fn sys_map(addr: usize, size: usize, flags: usize, handle: usize) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Verificar alinhamento se endereço fixo
    let flags_u32 = flags as u32;
    if flags_u32 & map_flags::FIXED != 0 && addr % 4096 != 0 {
        return Err(SysError::BadAlignment);
    }

    // TODO: Implementar mapeamento
    // - Se handle == 0: mapeamento anônimo
    // - Se handle != 0: verificar tipo (Memory/File) e mapear

    crate::kwarn!(
        "[Syscall] map({:#x}, {}, flags={:#x}, handle={}) não implementado",
        addr,
        size,
        flags,
        handle
    );
    Err(SysError::NotImplemented)
}

/// Remove mapeamento de memória.
///
/// # Syscall
/// `SYS_UNMAP (0x13)` - Args: (addr, size)
pub fn sys_unmap(addr: usize, size: usize) -> SysResult<usize> {
    if addr == 0 || size == 0 {
        return Err(SysError::InvalidArgument);
    }

    if addr % 4096 != 0 {
        return Err(SysError::BadAlignment);
    }

    // TODO: Usar VMM para remover mapeamento

    crate::kwarn!("[Syscall] unmap({:#x}, {}) não implementado", addr, size);
    Err(SysError::NotImplemented)
}
