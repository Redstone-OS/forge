//! # Eviction Engine
//!
//! Remove páginas da memória usando rmap.

use crate::mm::PhysAddr;
use core::sync::atomic::{AtomicU64, Ordering};

/// Páginas evicted
pub static PAGES_EVICTED: AtomicU64 = AtomicU64::new(0);

/// Evicta páginas para liberar memória
pub fn evict_pages(target_pages: usize) -> usize {
    let mut evicted = 0;

    for _ in 0..target_pages {
        // TODO: Obter candidato do page ager
        // TODO: Usar rmap para unmapear de todos os processos
        // TODO: Escrever em swap se necessário
        // TODO: Liberar frame
        evicted += 1;
    }

    PAGES_EVICTED.fetch_add(evicted as u64, Ordering::Relaxed);
    evicted
}

/// Evicta uma página específica
pub fn evict_page(phys: PhysAddr) -> bool {
    // 1. Obter rmap do frame
    let count = crate::mm::pfm::rmap::count(phys).unwrap_or(0);
    if count == 0 {
        return false;
    }

    // 2. Unmapear de todos os address spaces
    let _ = crate::mm::pfm::rmap::unmap_all(phys);

    // 3. Se dirty, escrever em swap
    // TODO: Implementar swap

    // 4. Liberar o frame
    let _ = crate::mm::pfm::free_frame(phys, 0);

    PAGES_EVICTED.fetch_add(1, Ordering::Relaxed);
    true
}
