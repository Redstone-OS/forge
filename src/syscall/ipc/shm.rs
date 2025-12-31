//! # Shared Memory Syscalls
//!
//! Criação e mapeamento de memória compartilhada.

use crate::ipc::shm::{ShmId, SHM_REGISTRY};
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_shm_create_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_shm_create(args.arg1)
}

pub fn sys_shm_map_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_shm_map(args.arg1 as u64, args.arg2)
}

// === IMPLEMENTAÇÕES ===

/// Cria uma região de memória compartilhada
///
/// # Args
/// - size: tamanho em bytes
///
/// # Returns
/// shm_id da região criada
pub fn sys_shm_create(size: usize) -> SysResult<usize> {
    if size == 0 || size > 16 * 1024 * 1024 {
        // Máximo 16MB por região SHM
        return Err(SysError::InvalidArgument);
    }

    let mut registry = SHM_REGISTRY.lock();
    match registry.create(size) {
        Ok(id) => {
            crate::kdebug!("(Syscall) sys_shm_create: id=", id.as_u64());
            Ok(id.as_u64() as usize)
        }
        Err(_) => Err(SysError::OutOfMemory),
    }
}

/// Mapeia uma região SHM no espaço do processo
///
/// # Args
/// - shm_id: ID da região
/// - suggested_addr: endereço sugerido (0 = kernel escolhe)
///
/// # Returns
/// Endereço onde foi mapeado
pub fn sys_shm_map(shm_id: u64, suggested_addr: usize) -> SysResult<usize> {
    let id = ShmId(shm_id);

    // Usar endereço fixo no userspace para SHM (0x10000000 + offset)
    let base_addr = if suggested_addr != 0 {
        suggested_addr as u64
    } else {
        // Endereço padrão baseado no ID
        // MUDANÇA: USAR 0x4000_0000 para evitar conflito com o HEAP (que começa em 0x1000_0000)
        0x4000_0000 + (shm_id * 0x100000)
    };

    let registry = SHM_REGISTRY.lock();
    if let Some(shm) = registry.get(id) {
        match shm.map(base_addr) {
            Ok(vaddr) => {
                crate::kdebug!("(Syscall) sys_shm_map: vaddr=", vaddr.as_u64());
                Ok(vaddr.as_u64() as usize)
            }
            Err(_) => Err(SysError::OutOfMemory),
        }
    } else {
        Err(SysError::InvalidHandle)
    }
}
