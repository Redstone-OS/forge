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
/// - flags: flags de alocação (ignorado por enquanto)
///
/// # Returns
/// Endereço da memória alocada ou erro
pub fn sys_alloc(size: usize, _flags: u32) -> SysResult<usize> {
    const USER_HEAP_MAX: u64 = 0x20000000;

    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Alinhar tamanho a 4KB (página)
    let aligned_size = (size + 0xFFF) & !0xFFF;

    // Obter task atual para acessar seu contador de heap
    let mut current_task = crate::sched::core::CURRENT.lock();
    let task = current_task.as_mut().ok_or(SysError::Interrupted)?;

    let alloc_addr = task.heap_next;

    if alloc_addr + aligned_size as u64 > USER_HEAP_MAX {
        crate::kerror!("(Syscall) sys_alloc: OOM (Virtual)! addr=", alloc_addr);
        return Err(SysError::OutOfMemory);
    }

    // Atualizar contador da task
    task.heap_next += aligned_size as u64;

    // Obter lock do PMM para alocar frames
    let mut pmm = crate::mm::pmm::FRAME_ALLOCATOR.lock();

    // Mapear as páginas necessárias
    let pages = aligned_size / 4096;
    let flags = crate::mm::vmm::MapFlags::PRESENT
        | crate::mm::vmm::MapFlags::WRITABLE
        | crate::mm::vmm::MapFlags::USER;

    for i in 0..pages {
        let vaddr = alloc_addr + (i * 4096) as u64;

        // Alocar frame físico
        let frame = match pmm.allocate_frame() {
            Some(f) => f,
            None => {
                crate::kerror!("(Syscall) sys_alloc: PMM OOM at page", i as u64);
                return Err(SysError::OutOfMemory);
            }
        };

        // Mapear no espaço de usuário atual
        if let Err(_) = crate::mm::vmm::map_page_with_pmm(vaddr, frame.addr(), flags, &mut *pmm) {
            crate::kerror!("(Syscall) sys_alloc: map failed at", vaddr);
            return Err(SysError::OutOfMemory);
        }

        // Registrar VMA (apenas na primeira página da alocação ou em cada página?
        // Idealmente, registrar uma única VMA para o bloco todo após o loop)
    }

    // Registrar VMA para o bloco inteiro
    if let Some(aspace) = &task.aspace {
        use crate::mm::aspace::vma::{MemoryIntent, Protection, VmaFlags};
        let mut as_lock = aspace.lock();
        let _ = as_lock.map_region(
            Some(crate::mm::VirtAddr::new(alloc_addr)),
            aligned_size,
            Protection::RW,
            VmaFlags::empty(),
            MemoryIntent::Heap,
        );
    }

    // Zerar o bloco (seguro agora que está mapeado e no contexto da task)
    unsafe {
        core::ptr::write_bytes(alloc_addr as *mut u8, 0, aligned_size);
    }

    // Importante: liberar o lock do CURRENT antes de retornar,
    // embora no Rust o Scoped Lock faça isso automaticamente.

    Ok(alloc_addr as usize)
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
    // O bump allocator atual não suporta liberação de memória
    // Isso é uma limitação conhecida - memória "vazada" será recuperada
    // quando o processo terminar (suas páginas são liberadas)
    //
    // TODO: Implementar allocator real com free quando tivermos
    // um gerenciador de memória virtual por processo

    let _ = (addr, size);
    // Retornar sucesso silenciosamente para não quebrar aplicações
    Ok(0)
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
