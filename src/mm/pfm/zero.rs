//! # Zero-on-Alloc
//!
//! Zeragem de páginas para segurança e zero-fill-on-demand.

use crate::mm::PhysAddr;
use core::sync::atomic::{AtomicU64, Ordering};

/// Estatísticas de zeragem
pub static PAGES_ZEROED: AtomicU64 = AtomicU64::new(0);
pub static BYTES_ZEROED: AtomicU64 = AtomicU64::new(0);

/// Zera uma página física
#[inline]
pub fn zero_page(phys: PhysAddr) {
    unsafe {
        let ptr: *mut u8 = crate::mm::hhdm::phys_to_virt(phys.as_u64());
        core::ptr::write_bytes(ptr, 0, crate::mm::config::PAGE_SIZE);
    }
    PAGES_ZEROED.fetch_add(1, Ordering::Relaxed);
    BYTES_ZEROED.fetch_add(crate::mm::config::PAGE_SIZE as u64, Ordering::Relaxed);
}

/// Zera múltiplas páginas
pub fn zero_pages(phys: PhysAddr, count: usize) {
    for i in 0..count {
        let addr = PhysAddr::new(phys.as_u64() + (i * crate::mm::config::PAGE_SIZE) as u64);
        zero_page(addr);
    }
}

/// Zera uma página usando instruções non-temporal (NT) se disponível
#[inline]
pub fn zero_page_nt(phys: PhysAddr) {
    // Por enquanto, usa zeragem normal
    // TODO: Implementar com MOVNTI para não poluir cache
    zero_page(phys);
}

/// Verifica se uma página está zerada
pub fn is_zeroed(phys: PhysAddr) -> bool {
    unsafe {
        let ptr: *const u64 = crate::mm::hhdm::phys_to_virt(phys.as_u64());
        let count = crate::mm::config::PAGE_SIZE / 8;
        for i in 0..count {
            if *ptr.add(i) != 0 {
                return false;
            }
        }
    }
    true
}
