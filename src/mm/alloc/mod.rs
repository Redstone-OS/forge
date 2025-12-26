//! # Alloc - Kernel Allocators
//!
//! Slab Allocator + Buddy System para alocação dinâmica.

// TODO: Implementar em fases posteriores
pub mod bump;
pub use bump::BumpAllocator;

pub mod buddy;
pub use buddy::BuddyAllocator;

pub mod slab;
pub use slab::SlabAllocator;
