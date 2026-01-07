//! # Shared Memory Syscalls
//!
//! Syscalls para criação e mapeamento de memória compartilhada.
//!
//! ## Syscalls Implementados
//!
//! | Syscall | Descrição |
//! |---------|-----------|
//! | `sys_shm_create` | Cria nova região SHM |
//! | `sys_shm_map` | Mapeia região no address space |
//! | `sys_shm_get_size` | Obtém tamanho de uma região |
//!
//! ## Uso
//!
//! ```rust,ignore
//! // Processo A: cria região de 4KB
//! let shm_id = sys_shm_create(4096)?;
//!
//! // Processo A: mapeia
//! let addr_a = sys_shm_map(shm_id, 0)?;
//!
//! // Processo B: mapeia mesma região (precisa conhecer shm_id)
//! let addr_b = sys_shm_map(shm_id, 0)?;
//!
//! // Ambos processos podem ler/escrever no mesmo endereço físico
//! ```

use crate::ipc::shm::{ShmError, ShmId, SHM_REGISTRY};
use crate::rmm::addr::VirtAddr;
use crate::rmm::virt::aspace::vma::{MemoryIntent, Protection};
use crate::rmm::virt::mapper;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// WRAPPERS
// =============================================================================

/// Wrapper para sys_shm_create
pub fn sys_shm_create_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_shm_create(args.arg1)
}

/// Wrapper para sys_shm_map
pub fn sys_shm_map_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_shm_map(args.arg1 as u64, args.arg2)
}

/// Wrapper para sys_shm_get_size
pub fn sys_shm_get_size_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_shm_get_size(args.arg1 as u64)
}

// =============================================================================
// IMPLEMENTAÇÕES
// =============================================================================

/// Tamanho máximo de uma região SHM (16 MB)
const SHM_MAX_SIZE: usize = 16 * 1024 * 1024;

/// Base de mapeamento para regiões SHM (24 GB)
const SHM_MMAP_BASE: u64 = 0x6_0000_0000;

/// Offset entre regiões SHM (16 MB)
const SHM_REGION_OFFSET: u64 = 0x100_0000;

/// Cria uma região de memória compartilhada
///
/// # Argumentos
///
/// * `size` - Tamanho em bytes (será arredondado para páginas)
///
/// # Retorna
///
/// * `Ok(shm_id)` - ID da região criada
/// * `Err(InvalidArgument)` - Tamanho inválido
/// * `Err(OutOfMemory)` - Sem memória disponível
pub fn sys_shm_create(size: usize) -> SysResult<usize> {
    // Validar tamanho
    if size == 0 || size > SHM_MAX_SIZE {
        crate::kerror!("(Syscall) sys_shm_create: invalid size=", size as u64);
        return Err(SysError::InvalidArgument);
    }

    let mut registry = SHM_REGISTRY.lock();

    match registry.create(size) {
        Ok(id) => {
            crate::kinfo!("(Syscall) sys_shm_create: id=", id.as_u64());
            Ok(id.as_u64() as usize)
        }
        Err(e) => {
            crate::kerror!("(Syscall) sys_shm_create failed");
            Err(shm_error_to_sys_error(e))
        }
    }
}

/// Mapeia uma região SHM no espaço do processo atual
///
/// # Argumentos
///
/// * `shm_id` - ID da região
/// * `suggested_addr` - Endereço sugerido (0 = kernel escolhe)
///
/// # Retorna
///
/// * `Ok(vaddr)` - Endereço onde foi mapeado
/// * `Err(InvalidHandle)` - ID inválido
/// * `Err(BadAddress)` - Falha no mapeamento
pub fn sys_shm_map(shm_id: u64, suggested_addr: usize) -> SysResult<usize> {
    let id = ShmId::new(shm_id);

    crate::ktrace!("(Syscall) sys_shm_map: id=", shm_id);

    // Determinar endereço de mapeamento
    let base_addr = if suggested_addr != 0 {
        suggested_addr as u64
    } else {
        // Calcular endereço baseado no ID
        SHM_MMAP_BASE + (shm_id * SHM_REGION_OFFSET)
    };

    // Obter CR3 do processo atual
    let (target_cr3, aspace_arc) = crate::sched::core::with_current(|task| {
        if let Some(ref aspace) = task.aspace {
            (aspace.lock().cr3(), Some(aspace.clone()))
        } else {
            (0, None)
        }
    })
    .unwrap_or((0, None));

    if target_cr3 == 0 {
        return Err(SysError::InvalidHandle);
    }

    // Mapear região
    let registry = SHM_REGISTRY.lock();
    if let Some(shm) = registry.get(id) {
        crate::ktrace!("(Syscall) sys_shm_map: addr=", base_addr);

        // Mapear páginas físicas no address space
        if let Err(e) = shm.map_at(target_cr3, base_addr) {
            crate::kerror!("(Syscall) sys_shm_map: map failed");
            return Err(shm_error_to_sys_error(e));
        }

        // Registrar VMA no AddressSpace
        if let Some(aspace) = aspace_arc {
            let mut as_lock = aspace.lock();
            let _ = as_lock.map_region(
                VirtAddr::new(base_addr),
                shm.size(),
                Protection::RW,
                MemoryIntent::SharedMemory,
            );
            crate::ktrace!("(Syscall) sys_shm_map: VMA registered");
        }

        // Flush TLB
        unsafe {
            let cr3 = mapper::read_cr3();
            mapper::write_cr3(cr3);
        }

        Ok(base_addr as usize)
    } else {
        crate::kerror!("(Syscall) sys_shm_map: invalid id=", shm_id);
        Err(SysError::InvalidHandle)
    }
}

/// Obtém o tamanho de uma região SHM
///
/// # Argumentos
///
/// * `shm_id` - ID da região
///
/// # Retorna
///
/// * `Ok(size)` - Tamanho em bytes
/// * `Err(InvalidHandle)` - ID inválido
pub fn sys_shm_get_size(shm_id: u64) -> SysResult<usize> {
    let id = ShmId::new(shm_id);
    let registry = SHM_REGISTRY.lock();

    if let Some(shm) = registry.get(id) {
        Ok(shm.size())
    } else {
        Err(SysError::InvalidHandle)
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Converte ShmError para SysError
fn shm_error_to_sys_error(e: ShmError) -> SysError {
    match e {
        ShmError::OutOfMemory => SysError::OutOfMemory,
        ShmError::InvalidId => SysError::InvalidHandle,
        ShmError::MapFailed => SysError::BadAddress,
        ShmError::NotMapped => SysError::BadAddress,
        ShmError::InvalidSize => SysError::InvalidArgument,
        ShmError::InvalidAddress => SysError::BadAddress,
    }
}
