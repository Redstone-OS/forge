//! Kernel Heap Allocator.
//!
//! Implementa GlobalAlloc usando um Bump Allocator simples.
//! Permite usar Box, Vec, String no Kernel.

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
///
/// # Mudança Importante
///
/// Esta função agora recebe o PMM diretamente como parâmetro ao invés de
/// receber uma closure de mapeamento. Isso é necessário porque:
///
/// 1. O VMM precisa alocar frames para criar page tables
/// 2. Se o VMM tentar adquirir o lock do PMM enquanto o heap já tem o lock,
///    ocorre DEADLOCK
/// 3. Passando o PMM diretamente, evitamos a necessidade de lock duplo
///
/// # Pré-requisitos
/// - PMM deve estar inicializado
/// - VMM deve estar inicializado (scratch slot pronto)
///
/// # Safety
/// Deve ser chamada apenas uma vez durante a inicialização do kernel
pub fn init_heap(pmm: &mut crate::mm::pmm::BitmapFrameAllocator) {
    let page_range = HEAP_START..(HEAP_START + HEAP_SIZE);

    for page_addr in (page_range).step_by(crate::mm::pmm::FRAME_SIZE) {
        // Alocar frame físico para esta página do heap
        let frame = pmm.allocate_frame().expect("No frames for heap");

        let flags = crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE;

        // Usar map_page_with_pmm para evitar deadlock
        // (já temos o lock do PMM, então passamos ele diretamente)
        let success =
            unsafe { crate::mm::vmm::map_page_with_pmm(page_addr as u64, frame.addr, flags, pmm) };

        if !success {
            panic!("Heap: Falha ao mapear página {:#x}", page_addr);
        }
    }

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }

    crate::kinfo!("Heap: {} KiB em {:#x}", HEAP_SIZE / 1024, HEAP_START);
}
