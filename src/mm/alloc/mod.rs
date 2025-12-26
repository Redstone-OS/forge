//! # Alloc - Kernel Allocators
//!
//! Slab Allocator + Buddy System para alocação dinâmica.

pub mod bump;
pub use bump::BumpAllocator;

pub mod buddy;
pub use buddy::BuddyAllocator;

pub mod slab;
pub use slab::SlabAllocator;

// Per-CPU caches para reduzir contenção
#[cfg(feature = "percpu_caches")]
pub mod percpu;
