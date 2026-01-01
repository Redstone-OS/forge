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

pub fn sys_shm_get_size_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_shm_get_size(args.arg1 as u64)
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

    // Base address for SHM mappings (24GB)
    // Movido para 24GB (0x6_0000_0000) e implementado lógica de limpeza de Huge Pages.
    // O problema anterior era um PDE pré-existente marcado como Huge Page (2MB) que apontava
    // para memória física inexistente (Identity Map incorreto/sobra).
    // O Mapper não quebrava essa página, então a escrita ía para o limbo.
    let base_addr = if suggested_addr != 0 {
        suggested_addr as u64
    } else {
        0x6_0000_0000 + (shm_id * 0x1000000)
    };

    let registry = SHM_REGISTRY.lock();
    if let Some(shm) = registry.get(id) {
        // crate::kdebug!("(Syscall) sys_shm_map: vaddr=", base_addr);

        // 0. FIX DO BURACO NEGRO (Bunker Buster)
        // Verifica se existe uma Huge Page (2MB) bloqueando este endereço.
        // Se existir, ZERA a entrada do PDE para forçar o mapper a criar uma Page Table nova.
        unsafe {
            nuke_huge_page_if_exists(base_addr);
        }

        // 1. Mapear
        match shm.map(base_addr) {
            Ok(_) => {
                // Flush TLB
                unsafe {
                    let cr3: u64;
                    core::arch::asm!("mov {}, cr3", out(reg) cr3);
                    core::arch::asm!("mov cr3, {}", in(reg) cr3);
                }

                Ok(base_addr as usize)
            }
            Err(_) => Err(SysError::BadAddress),
        }
    } else {
        Err(SysError::InvalidHandle)
    }
}

/// Obtém o tamanho de uma região SHM
///
/// # Args
/// - shm_id: ID da região
///
/// # Returns
/// Tamanho em bytes
pub fn sys_shm_get_size(shm_id: u64) -> SysResult<usize> {
    let id = ShmId(shm_id);
    let registry = SHM_REGISTRY.lock();

    if let Some(shm) = registry.get(id) {
        Ok(shm.size)
    } else {
        Err(SysError::InvalidHandle)
    }
}

// Remove entradas Huge Page que bloqueiam o mapeamento granular
unsafe fn nuke_huge_page_if_exists(vaddr: u64) {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3);
    let pml4_phys = cr3 & !0xFFF;

    let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
    let pd_idx = ((vaddr >> 21) & 0x1FF) as usize;

    let pml4_ptr = pml4_phys as *const u64;
    let pml4e = *pml4_ptr.add(pml4_idx);
    if pml4e & 1 == 0 {
        return;
    } // Não mapeado

    let pdpt_phys = pml4e & 0x000F_FFFF_FFFF_F000;
    let pdpt_ptr = pdpt_phys as *const u64;
    let pdpte = *pdpt_ptr.add(pdpt_idx);
    if pdpte & 1 == 0 {
        return;
    } // Não mapeado

    // Check Huge PDPT (1GB) - just in case
    if (pdpte & 0x80) != 0 {
        crate::kdebug!("(SHM) WARN: Found 1GB Huge Page at PDPT. Nuking...");
        let mut_pdpt_ptr = pdpt_phys as *mut u64;
        *mut_pdpt_ptr.add(pdpt_idx) = 0; // Clear
        core::arch::asm!("invlpg [{}]", in(reg) vaddr);
        return;
    }

    let pd_phys = pdpte & 0x000F_FFFF_FFFF_F000;
    let pd_ptr = pd_phys as *const u64;
    let pde = *pd_ptr.add(pd_idx);
    if pde & 1 == 0 {
        return;
    } // Não mapeado

    // Check Huge PDE (2MB) - O CULPADO
    if (pde & 0x80) != 0 {
        crate::kdebug!("(SHM) FATAL: Found 2MB Huge Page at PDE. Nuking to allow split!");
        crate::kdebug!("(SHM) PDE was: ", pde);
        let mut_pd_ptr = pd_phys as *mut u64;
        *mut_pd_ptr.add(pd_idx) = 0; // DELETE A ENTRADA
        core::arch::asm!("invlpg [{}]", in(reg) vaddr);
    }
}
