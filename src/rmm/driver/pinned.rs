//! # Pinned Memory
//!
//! Memória que não pode ser swapped/evicted.

use crate::rmm::addr::PhysAddr;
use crate::rmm::zone::Zone;

/// Aloca páginas pinadas
pub fn alloc_pinned(count: usize, zone: Zone) -> Option<PhysAddr> {
    // TODO: Alocar com FrameOwner::Pinned
    None
}

/// Libera páginas pinadas
pub fn free_pinned(phys: PhysAddr, count: usize) {
    // TODO: Liberar frames
}
