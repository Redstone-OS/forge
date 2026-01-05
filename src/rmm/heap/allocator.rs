//! # Heap Allocator
//!
//! Orquestra Buddy e Slab allocators.

use core::alloc::Layout;
use core::ptr;

/// Heap allocator principal
pub struct HeapAllocator {
    base: usize,
    size: usize,
    // TODO: buddy e slab allocators
}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self { base: 0, size: 0 }
    }

    pub fn init(&mut self, base: usize, size: usize) {
        self.base = base;
        self.size = size;
        // TODO: Inicializar buddy e slab
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // TODO: Implementar alocação
        // if layout.size() <= SLAB_MAX_SIZE { slab.alloc() }
        // else { buddy.alloc() }
        ptr::null_mut()
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        // TODO: Implementar desalocação
    }

    pub unsafe fn realloc(&mut self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // TODO: Implementar realloc
        ptr::null_mut()
    }
}
