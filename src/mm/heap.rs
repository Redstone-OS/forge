//! Alocador de Heap Simples
//!
//! Implementa bump allocator simples para MVP.
//! TODO: Substituir por linked-list allocator completo depois.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use spin::Mutex;

/// Alocador bump simples
struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    /// Cria novo alocador
    const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
        }
    }

    /// Inicializa o heap
    unsafe fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_end = start + size;
        self.next = start;
    }

    /// Alinha endereço para cima
    fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }

    /// Aloca memória
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = Self::align_up(self.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > self.heap_end {
            // Sem memória
            ptr::null_mut()
        } else {
            self.next = alloc_end;
            alloc_start as *mut u8
        }
    }
}

/// Wrapper com Mutex para thread-safety
pub struct LockedHeap(Mutex<BumpAllocator>);

impl LockedHeap {
    /// Cria novo heap travado
    pub const fn new() -> Self {
        LockedHeap(Mutex::new(BumpAllocator::new()))
    }

    /// Inicializa o heap
    pub fn init(&self, start: usize, size: usize) {
        unsafe {
            self.0.lock().init(start, size);
        }
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator não libera memória (simplificação para MVP)
        // TODO: Implementar linked-list allocator com dealloc real
    }
}

/// Alocador global
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::new();
