//! # Swap (Stub)
//!
//! Swap de páginas para armazenamento secundário.
//! STUB: Será implementado quando block device e reclaim estiverem prontos.

use crate::rmm::addr::PhysAddr;

/// Slot no swap
#[derive(Debug, Clone, Copy)]
pub struct SwapSlot(pub u64);

/// Inicializa swap
pub fn init() {
    // TODO: Configurar backing store
}

/// Escreve página em swap
pub fn swap_out(_phys: PhysAddr) -> Option<SwapSlot> {
    // TODO: Implementar quando block device estiver pronto
    None
}

/// Lê página de swap
pub fn swap_in(_slot: SwapSlot) -> Option<PhysAddr> {
    // TODO: Implementar
    None
}

/// Libera slot de swap
pub fn free_slot(_slot: SwapSlot) {
    // TODO: Implementar
}

/// Retorna espaço usado no swap
pub fn used() -> usize {
    0
}

/// Retorna espaço total do swap
pub fn total() -> usize {
    0
}
