//! # Memory Allocation Syscalls
//!
//! Syscalls para alocação e gerenciamento de memória virtual.
//!
//! ## Syscalls Implementados
//!
//! | Syscall        | Número | Status      |
//! |----------------|--------|-------------|
//! | `sys_alloc`    | 0x10   | Funcional   |
//! | `sys_free`     | 0x11   | Stub (noop) |
//! | `sys_map`      | 0x12   | Stub        |
//! | `sys_unmap`    | 0x13   | Stub        |
//! | `sys_mprotect` | 0x14   | Stub        |
//! | `sys_meminfo`  | 0x15   | Stub        |
//! | `sys_alloc_at` | 0x16   | Stub        |
//!
//! ## Layout de Memória do Processo
//!
//! ```text
//! │ 0xFFFF_FFFF_FFFF_FFFF │
//! ├───────────────────────┤ Kernel Space
//! │                       │
//! │ 0x0000_8000_0000_0000 │
//! ├───────────────────────┤
//! │      User Stack       │ (grows down)
//! ├───────────────────────┤ 0x7FFF_FFFF_0000
//! │      SHM Region       │ 0x6_0000_0000 - 0x6_FFFF_FFFF
//! ├───────────────────────┤
//! │      Heap/mmap        │ 0x1000_0000 - 0x2000_0000
//! ├───────────────────────┤
//! │      Code/Data        │ 0x0040_0000 - ...
//! ├───────────────────────┤
//! │ 0x0000_0000_0000_0000 │
//! ```

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::aspace::vma::{MemoryIntent, Protection};
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// CONSTANTES
// =============================================================================

// Todo: Revisar
#[allow(unused)]
/// Base do heap do usuário (256 MB)
const USER_HEAP_BASE: u64 = 0x1000_0000;

/// Limite do heap do usuário (512 MB)
const USER_HEAP_MAX: u64 = 0x2000_0000;

// Flags de alocação
pub const ALLOC_ZEROED: u32 = 1;
pub const ALLOC_COMMIT: u32 = 2;
pub const ALLOC_GUARD: u32 = 4;

// =============================================================================
// WRAPPERS
// =============================================================================

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

pub fn sys_meminfo_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_meminfo(args.arg1)
}

pub fn sys_alloc_at_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_alloc_at(args.arg1, args.arg2, args.arg3 as u32)
}

// =============================================================================
// IMPLEMENTAÇÕES
// =============================================================================

/// Aloca memória virtual (kernel escolhe endereço)
///
/// # Argumentos
///
/// * `size` - Tamanho em bytes (será alinhado a PAGE_SIZE)
/// * `flags` - Flags de alocação (ALLOC_ZEROED, ALLOC_COMMIT, ALLOC_GUARD)
///
/// # Retorna
///
/// * `Ok(addr)` - Endereço da memória alocada
/// * `Err(InvalidArgument)` - Size = 0
/// * `Err(OutOfMemory)` - Sem memória disponível
pub fn sys_alloc(size: usize, flags: u32) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Alinhar tamanho a PAGE_SIZE
    let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    let pages = aligned_size / PAGE_SIZE;

    // Obter task atual
    let (alloc_addr, target_cr3, aspace_arc) = crate::sched::core::with_current_mut(|task| {
        let addr = task.heap_next;
        if addr + aligned_size as u64 > USER_HEAP_MAX {
            crate::kerror!("(Syscall) sys_alloc: OOM (Virtual)! addr=", addr);
            return Err(SysError::OutOfMemory);
        }

        // Atualizar heap pointer
        task.heap_next += aligned_size as u64;

        let cr3 = task.aspace.as_ref().map(|a| a.lock().cr3()).unwrap_or(0);
        let aspace = task.aspace.clone();

        Ok((addr, cr3, aspace))
    })
    .unwrap_or(Err(SysError::Interrupted))?;

    // Flags de alocação
    let alloc_flags = if flags & ALLOC_ZEROED != 0 {
        AllocFlags::ZERO
    } else {
        AllocFlags::empty()
    };

    // Alocar e mapear páginas
    for i in 0..pages {
        let vaddr = alloc_addr + (i * PAGE_SIZE) as u64;

        // Alocar frame físico
        let frame = phys::alloc(FrameOwner::Process { pid: 0 }, Zone::Normal, alloc_flags)
            .ok_or_else(|| {
                crate::kerror!("(Syscall) sys_alloc: PMM OOM at page", i as u64);
                SysError::OutOfMemory
            })?;

        // Mapear no address space
        if let Err(_) = map_user_page(target_cr3, vaddr, frame.as_u64()) {
            crate::kerror!("(Syscall) sys_alloc: map failed at", vaddr);
            return Err(SysError::OutOfMemory);
        }
    }

    // Registrar VMA
    if let Some(aspace) = aspace_arc {
        let mut as_lock = aspace.lock();
        let _ = as_lock.map_region(
            VirtAddr::new(alloc_addr),
            aligned_size,
            Protection::RW,
            MemoryIntent::Heap,
        );
    }

    // Zerar (se não zerado pela alocação)
    if flags & ALLOC_ZEROED != 0 {
        // Já zerado pelo AllocFlags::ZERO
    } else {
        unsafe {
            core::ptr::write_bytes(alloc_addr as *mut u8, 0, aligned_size);
        }
    }

    crate::ktrace!("(Syscall) sys_alloc: addr=", alloc_addr);
    crate::ktrace!("(Syscall) sys_alloc: size=", aligned_size as u64);

    Ok(alloc_addr as usize)
}

/// Libera memória alocada
///
/// # TODO
///
/// Implementar liberação real quando tivermos gerenciador de heap adequado.
/// Por enquanto, memória é recuperada quando o processo termina.
pub fn sys_free(addr: usize, size: usize) -> SysResult<usize> {
    // TODO: Implementar liberação real
    // - Lookup VMA contendo addr
    // - Unmap páginas
    // - Liberar frames com phys::free()
    // - Remover VMA
    //
    // Por enquanto: noop (memória liberada quando processo termina)
    let _ = (addr, size);
    Ok(0)
}

/// Mapeia região de memória (estilo mmap)
///
/// # TODO
///
/// Implementar mapeamento real com suporte a:
/// - MAP_ANONYMOUS: memória anônima
/// - MAP_PRIVATE: copy-on-write
/// - MAP_SHARED: compartilhada
/// - MAP_FIXED: endereço exato
pub fn sys_map(addr: usize, size: usize, prot: u32, flags: u32) -> SysResult<usize> {
    // TODO: Implementar mapeamento
    let _ = (addr, size, prot, flags);
    crate::kwarn!("(Syscall) sys_map não implementado");
    Err(SysError::NotImplemented)
}

/// Remove mapeamento de memória
///
/// # TODO
///
/// Implementar unmapping real:
/// - Lookup VMAs na região
/// - Unmap páginas
/// - Liberar frames se não compartilhados
/// - Remover/ajustar VMAs
pub fn sys_unmap(addr: usize, size: usize) -> SysResult<usize> {
    // TODO: Implementar unmapping
    let _ = (addr, size);
    crate::kwarn!("(Syscall) sys_unmap não implementado");
    Err(SysError::NotImplemented)
}

/// Altera proteções de uma região de memória
///
/// # TODO
///
/// Implementar mprotect real:
/// - Validar nova proteção
/// - Atualizar page table entries
/// - Flush TLB
pub fn sys_mprotect(addr: usize, size: usize, prot: u32) -> SysResult<usize> {
    // TODO: Implementar mprotect
    let _ = (addr, size, prot);
    crate::kwarn!("(Syscall) sys_mprotect não implementado");
    Err(SysError::NotImplemented)
}

/// Obtém informações de memória do sistema
///
/// # TODO
///
/// Implementar com estrutura MemInfo contendo:
/// - total_pages
/// - free_pages
/// - used_pages
/// - shared_pages
/// - cached_pages
pub fn sys_meminfo(out_ptr: usize) -> SysResult<usize> {
    // TODO: Implementar meminfo
    let _ = out_ptr;
    crate::kwarn!("(Syscall) sys_meminfo não implementado");
    Err(SysError::NotImplemented)
}

/// Aloca memória em endereço específico
///
/// # TODO
///
/// Similar a sys_alloc mas com endereço fixo.
/// Falha se região já estiver mapeada.
pub fn sys_alloc_at(addr: usize, size: usize, flags: u32) -> SysResult<usize> {
    // TODO: Implementar alloc_at
    let _ = (addr, size, flags);
    crate::kwarn!("(Syscall) sys_alloc_at não implementado");
    Err(SysError::NotImplemented)
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Mapeia uma página física no address space do processo
fn map_user_page(target_cr3: u64, vaddr: u64, phys_frame: u64) -> Result<(), SysError> {
    let table_flags: u64 = 0x7; // Present | Writable | User
    let page_flags: u64 = 0x7;

    unsafe {
        let pml4_phys = target_cr3 & !0xFFF;
        let pml4 = hhdm::phys_to_virt(pml4_phys) as *mut u64;

        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pd_idx = ((vaddr >> 21) & 0x1FF) as usize;
        let pt_idx = ((vaddr >> 12) & 0x1FF) as usize;

        // Ensure PDPT exists
        if (*pml4.add(pml4_idx)) & 1 == 0 {
            let new_pdpt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(SysError::OutOfMemory)?;
            *pml4.add(pml4_idx) = new_pdpt.as_u64() | table_flags;
        }

        let pdpt_phys = (*pml4.add(pml4_idx)) & 0x000F_FFFF_FFFF_F000;
        let pdpt = hhdm::phys_to_virt(pdpt_phys) as *mut u64;

        // Ensure PD exists
        if (*pdpt.add(pdpt_idx)) & 1 == 0 {
            let new_pd = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(SysError::OutOfMemory)?;
            *pdpt.add(pdpt_idx) = new_pd.as_u64() | table_flags;
        }

        let pd_phys = (*pdpt.add(pdpt_idx)) & 0x000F_FFFF_FFFF_F000;
        let pd = hhdm::phys_to_virt(pd_phys) as *mut u64;

        // Ensure PT exists
        if (*pd.add(pd_idx)) & 1 == 0 {
            let new_pt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(SysError::OutOfMemory)?;
            *pd.add(pd_idx) = new_pt.as_u64() | table_flags;
        }

        let pt_phys = (*pd.add(pd_idx)) & 0x000F_FFFF_FFFF_F000;
        let pt = hhdm::phys_to_virt(pt_phys) as *mut u64;

        // Map the page
        *pt.add(pt_idx) = phys_frame | page_flags;
    }

    Ok(())
}
