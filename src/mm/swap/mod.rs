//! # Swap Subsystem
//!
//! Backing store para páginas evicted.

use crate::mm::PhysAddr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Swap está habilitado?
static SWAP_ENABLED: AtomicBool = AtomicBool::new(false);

/// Páginas em swap
static PAGES_SWAPPED_OUT: AtomicU64 = AtomicU64::new(0);
static PAGES_SWAPPED_IN: AtomicU64 = AtomicU64::new(0);

/// Slot de swap (índice no backing store)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapSlot(pub u64);

impl SwapSlot {
    pub const INVALID: Self = Self(u64::MAX);
    pub fn is_valid(&self) -> bool {
        self.0 != u64::MAX
    }
}

/// Inicializa swap com backing store
pub fn init(_backing_size: u64) {
    // TODO: Alocar bitmap de slots livres
    SWAP_ENABLED.store(true, Ordering::Release);
    crate::kinfo!("(SWAP) Initialized");
}

/// Verifica se swap está habilitado
pub fn is_enabled() -> bool {
    SWAP_ENABLED.load(Ordering::Acquire)
}

/// Escreve página em swap
pub fn swap_out(phys: PhysAddr) -> Option<SwapSlot> {
    if !is_enabled() {
        return None;
    }

    // TODO: Encontrar slot livre
    // TODO: Escrever conteúdo no backing store

    let _ = phys;
    PAGES_SWAPPED_OUT.fetch_add(1, Ordering::Relaxed);
    None
}

/// Lê página de swap
pub fn swap_in(slot: SwapSlot) -> Option<PhysAddr> {
    if !slot.is_valid() {
        return None;
    }

    // TODO: Alocar frame
    // TODO: Ler do backing store
    // TODO: Liberar slot

    PAGES_SWAPPED_IN.fetch_add(1, Ordering::Relaxed);
    None
}

/// Libera slot de swap
pub fn free_slot(_slot: SwapSlot) {
    // TODO: Marcar slot como livre
}

/// Estatísticas de swap
pub fn stats() -> (u64, u64) {
    (
        PAGES_SWAPPED_OUT.load(Ordering::Relaxed),
        PAGES_SWAPPED_IN.load(Ordering::Relaxed),
    )
}
