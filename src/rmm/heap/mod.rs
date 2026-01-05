//! # Kernel Heap
//!
//! Heap do kernel usando Buddy + Slab allocators.
//! Implementa GlobalAlloc para uso com alloc crate (Box, Vec, etc).

pub mod allocator;
pub mod aslr;
pub mod buddy;
pub mod slab;

pub use allocator::HeapAllocator;

use crate::sync::Spinlock;
use core::alloc::{GlobalAlloc, Layout};

/// Heap global do kernel
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Wrapper com lock para GlobalAlloc
pub struct LockedHeap(Spinlock<Option<HeapAllocator>>);

impl LockedHeap {
    pub const fn empty() -> Self {
        Self(Spinlock::new(None))
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut guard = self.0.lock();
        match guard.as_mut() {
            Some(heap) => heap.alloc(layout),
            None => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut guard = self.0.lock();
        if let Some(heap) = guard.as_mut() {
            heap.dealloc(ptr, layout);
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let mut guard = self.0.lock();
        match guard.as_mut() {
            Some(heap) => heap.realloc(ptr, layout, new_size),
            None => core::ptr::null_mut(),
        }
    }
}

/// Inicializa o heap do kernel
pub fn init() {
    crate::kinfo!("(RMM/Heap) Inicializando heap...");

    // TODO: Implementar inicialização
    // 1. Gerar ASLR offset
    // 2. Alocar frames para heap
    // 3. Mapear no espaço virtual
    // 4. Inicializar buddy + slab
    // 5. Conectar ao GlobalAlloc

    crate::kinfo!("(RMM/Heap) Heap inicializado (stub)");
}
