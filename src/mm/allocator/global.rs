//! Global Allocator - Alocador de memória global
//!
//! Implementação básica do global allocator para o kernel.

use core::alloc::{GlobalAlloc, Layout};

/// Dummy allocator - apenas para compilar
/// TODO(prioridade=alta, versão=v1.0): Implementar allocator real
pub struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // TODO: Implementar alocação real
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // TODO: Implementar dealocação real
    }
}

#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

// TODO(prioridade=alta, versão=v1.0): Implementar allocator real
// - Integrar com kmalloc/kfree
// - Usar buddy allocator ou slab allocator
// - Suportar diferentes tamanhos de blocos
