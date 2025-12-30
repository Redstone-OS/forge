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
    use core::sync::atomic::{AtomicUsize, Ordering};

    // Região de heap do userspace: 0x10000000 - 0x20000000 (256 MB)
    const USER_HEAP_START: usize = 0x10000000;
    const USER_HEAP_END: usize = 0x20000000;

    // Bump allocator simples (global por enquanto - deveria ser per-process)
    static USER_HEAP_NEXT: AtomicUsize = AtomicUsize::new(USER_HEAP_START);

    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Alinhar tamanho a 4KB (página)
    let aligned_size = (size + 0xFFF) & !0xFFF;

    // Tentar alocar espaço
    let alloc_addr = USER_HEAP_NEXT.fetch_add(aligned_size, Ordering::SeqCst);

    if alloc_addr + aligned_size > USER_HEAP_END {
        crate::kerror!("(Syscall) sys_alloc: OOM! addr=", alloc_addr as u64);
        return Err(SysError::OutOfMemory);
    }

    // Obter lock do PMM para alocar frames
    let mut pmm = crate::mm::pmm::FRAME_ALLOCATOR.lock();

    // Mapear as páginas necessárias
    let pages = aligned_size / 4096;
    let flags = crate::mm::vmm::MapFlags::PRESENT
        | crate::mm::vmm::MapFlags::WRITABLE
        | crate::mm::vmm::MapFlags::USER;

    for i in 0..pages {
        let vaddr = alloc_addr + (i * 4096);

        // Alocar frame físico
        let frame = match pmm.allocate_frame() {
            Some(f) => f,
            None => {
                crate::kerror!("(Syscall) sys_alloc: PMM OOM at page", i as u64);
                return Err(SysError::OutOfMemory);
            }
        };

        // Mapear no espaço de usuário atual
        if let Err(_) =
            crate::mm::vmm::map_page_with_pmm(vaddr as u64, frame.addr(), flags, &mut *pmm)
        {
            crate::kerror!("(Syscall) sys_alloc: map failed at", vaddr as u64);
            return Err(SysError::OutOfMemory);
        }

        // Zerar a página
        unsafe {
            core::ptr::write_bytes(vaddr as *mut u8, 0, 4096);
        }
    }

    Ok(alloc_addr)
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
