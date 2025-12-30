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

pub fn sys_port_connect_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_port_connect(args.arg1, args.arg2)
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
        0x1000_0000 + (shm_id * 0x100000)
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

/// Conecta a uma porta nomeada
///
/// # Args
/// - name_ptr: ponteiro para nome da porta
/// - name_len: tamanho do nome
///
/// # Returns
/// port_id
pub fn sys_port_connect(name_ptr: usize, name_len: usize) -> SysResult<usize> {
    if name_ptr == 0 || name_len == 0 || name_len > 256 {
        return Err(SysError::InvalidArgument);
    }

    // Ler nome do userspace
    let name_bytes = unsafe { core::slice::from_raw_parts(name_ptr as *const u8, name_len) };

    let name = match core::str::from_utf8(name_bytes) {
        Ok(s) => s,
        Err(_) => return Err(SysError::InvalidArgument),
    };

    let registry = crate::ipc::port::PORT_REGISTRY.lock();
    if let Some(port_id) = registry.lookup(name) {
        crate::kdebug!("(Syscall) sys_port_connect: found port=", port_id.as_u64());
        Ok(port_id.as_u64() as usize)
    } else {
        Err(SysError::NotFound)
    }
}
