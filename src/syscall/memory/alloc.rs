//! # Memory Allocation Syscalls
//!
//! alloc, free, map, unmap

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_alloc_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_alloc(args.arg1, args.arg2 as u32)
}

pub fn sys_free_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_free(args.arg1, args.arg2)
}

pub fn sys_map_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_map(args.arg1, args.arg2, args.arg3 as u32, args.arg4 as u32)
}

pub fn sys_unmap_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_unmap(args.arg1, args.arg2)
}

pub fn sys_mprotect_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_mprotect(args.arg1, args.arg2, args.arg3 as u32)
}

// === IMPLEMENTAÇÕES ===

/// Aloca memória virtual
///
/// # Args
/// - size: tamanho em bytes
/// - flags: flags de alocação
///
/// # Returns
/// Endereço da memória alocada ou erro
pub fn sys_alloc(size: usize, flags: u32) -> SysResult<usize> {
    // TODO: Validar size (alinhamento, máximo)
    // TODO: Encontrar região livre no address space do processo
    // TODO: Alocar páginas físicas
    // TODO: Mapear páginas
    // TODO: Retornar endereço

    let _ = (size, flags);
    crate::kwarn!("(Syscall) sys_alloc não implementado");
    Err(SysError::NotImplemented)
}

/// Libera memória alocada
///
/// # Args
/// - addr: endereço da região
/// - size: tamanho da região
///
/// # Returns
/// 0 ou erro
pub fn sys_free(addr: usize, size: usize) -> SysResult<usize> {
    // TODO: Validar que addr está no espaço do processo
    // TODO: Validar que a região foi alocada via sys_alloc
    // TODO: Desmapear páginas
    // TODO: Liberar páginas físicas

    let _ = (addr, size);
    crate::kwarn!("(Syscall) sys_free não implementado");
    Err(SysError::NotImplemented)
}

/// Mapeia memória ou handle
///
/// # Args
/// - addr: endereço desejado (0 = kernel escolhe)
/// - size: tamanho da região
/// - flags: permissões (READ/WRITE/EXEC)
/// - handle: handle do objeto (0 = memória anônima)
///
/// # Returns
/// Endereço mapeado ou erro
pub fn sys_map(addr: usize, size: usize, flags: u32, handle: u32) -> SysResult<usize> {
    // TODO: Validar parâmetros
    // TODO: Se handle != 0, validar que é mapeável
    // TODO: Criar mapeamento no VMM do processo
    // TODO: Retornar endereço

    let _ = (addr, size, flags, handle);
    crate::kwarn!("(Syscall) sys_map não implementado");
    Err(SysError::NotImplemented)
}

/// Remove mapeamento de memória
///
/// # Args
/// - addr: endereço da região
/// - size: tamanho da região
///
/// # Returns
/// 0 ou erro
pub fn sys_unmap(addr: usize, size: usize) -> SysResult<usize> {
    // TODO: Validar parâmetros
    // TODO: Remover mapeamento do VMM
    // TODO: Flush TLB

    let _ = (addr, size);
    crate::kwarn!("(Syscall) sys_unmap não implementado");
    Err(SysError::NotImplemented)
}

/// Altera as proteções de uma região de memória
pub fn sys_mprotect(addr: usize, size: usize, flags: u32) -> SysResult<usize> {
    let _ = (addr, size, flags);
    crate::kwarn!("(Syscall) sys_mprotect não implementado");
    Err(SysError::NotImplemented)
}
