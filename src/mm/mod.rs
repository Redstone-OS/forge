//! Módulo de Gerenciamento de Memória.
//!
//! Submódulos:
//! - `pmm`: Gerenciador de memória física (Frames).
//! - `vmm`: Gerenciador de memória virtual (Pages).
//! - `heap`: Alocador dinâmico (Box, Vec).

pub mod heap;
pub mod pmm;
pub mod vmm;

/// Inicializa o sistema de memória completo.
pub fn init(boot_info: &'static crate::core::handoff::BootInfo) {
    // 1. Inicializar PMM (Bitmap)
    // SAFETY: BootInfo é confiável neste ponto (validado no entry).
    unsafe {
        pmm::FRAME_ALLOCATOR.lock().init(boot_info);
    }

    // 2. Inicializar VMM (Virtual Memory)
    unsafe {
        vmm::init(boot_info);
    }

    // 3. Inicializar Heap (Permite Vec/Box)
    // Mapeamos a região do Heap virtualmente para frames físicos recém alocados.
    let mut pmm_lock = pmm::FRAME_ALLOCATOR.lock();
    heap::init_heap(
        &mut |v, p, f| unsafe { vmm::map_page(v, p, f) },
        &mut *pmm_lock,
    );
}
