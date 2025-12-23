//! Kernel Heap Allocator.
//!
//! Implementa `GlobalAlloc` usando um Bump Allocator simples.
//! Permite usar `Box`, `Vec`, `String` no Kernel.

use crate::sync::Mutex;
use core::alloc::{GlobalAlloc, Layout};

// Heap Size: 1MB inicial (expansível no futuro)
pub const HEAP_START: usize = 0xFFFF_9000_0000_0000; // Endereço Virtual Arbitrário (Higher Half)
pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub struct LockedHeap {
    inner: Mutex<BumpAllocator>,
}

impl LockedHeap {
    pub const fn empty() -> Self {
        Self {
            inner: Mutex::new(BumpAllocator::new()),
        }
    }

    pub unsafe fn init(&self, start: usize, size: usize) {
        self.inner.lock().init(start, size);
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }
}

/// Bump Allocator Simples.
/// Rápido, mas não recicla memória (vaza tudo).
/// Suficiente para o boot e estruturas estáticas.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };

        if alloc_end > self.heap_end {
            core::ptr::null_mut() // OOM
        } else {
            self.next = alloc_end;
            self.allocations += 1;
            alloc_start as *mut u8
        }
    }

    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        self.allocations -= 1;
        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Inicializa o Heap.
/// Precisa ser chamado APÓS o VMM estar mapeando a região do Heap.
pub fn init_heap(
    mapper: &mut impl FnMut(u64, u64, u64) -> bool,
    pmm: &mut crate::mm::pmm::BitmapFrameAllocator,
) {
    let page_range = HEAP_START..(HEAP_START + HEAP_SIZE);

    for page_addr in (page_range).step_by(crate::mm::pmm::FRAME_SIZE) {
        let frame = pmm.allocate_frame().expect("No frames for heap");
        let flags = crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE;
        mapper(page_addr as u64, frame.addr, flags);
    }

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
    crate::kinfo!(
        "Heap Initialized at {:#x} ({} MiB)",
        HEAP_START,
        HEAP_SIZE / 1024 / 1024
    );
}
