//! # HeapAllocator
//!
//! Allocator principal que combina Slab + Buddy.
//!
//! ## Estratégia de Alocação
//!
//! - **Pequeno (8B - 2KB)**: Slab Allocator
//! - **Grande (> 2KB)**: Buddy Allocator
//!
//! ## GlobalAlloc
//!
//! Implementa `GlobalAlloc` para uso com `#[global_allocator]`.

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::*;
use crate::sync::Spinlock;

use super::buddy::BuddyAllocator;
use super::slab::SlabAllocator;
use super::HeapStats;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// =============================================================================
// HeapAllocator
// =============================================================================

/// Allocator principal do kernel
///
/// Combina Slab (pequeno) e Buddy (grande) allocators.
pub struct HeapAllocator {
    /// Base do heap
    base: VirtAddr,
    /// Tamanho atual do heap
    size: usize,
    /// Slab allocator (8B - 2KB)
    slab: SlabAllocator,
    /// Buddy allocator (> 2KB)
    buddy: BuddyAllocator,
    /// Threshold entre slab e buddy
    slab_max_size: usize,
    /// Estatísticas
    total_allocs: AtomicU64,
    total_frees: AtomicU64,
    active_allocs: AtomicUsize,
    used_bytes: AtomicUsize,
    slab_allocs: AtomicU64,
    buddy_allocs: AtomicU64,
}

impl HeapAllocator {
    /// Cria novo allocator
    pub fn new(base: VirtAddr, size: usize) -> Self {
        // Divide o espaço: 1/4 para slab metadata, 3/4 para dados
        let slab_region_size = size / 4;
        let buddy_region_size = size - slab_region_size;

        let slab_base = base;
        let buddy_base = base + slab_region_size as u64;

        Self {
            base,
            size,
            slab: SlabAllocator::new(slab_base, slab_region_size),
            buddy: BuddyAllocator::new(buddy_base, buddy_region_size),
            slab_max_size: SLAB_MAX_SIZE,
            total_allocs: AtomicU64::new(0),
            total_frees: AtomicU64::new(0),
            active_allocs: AtomicUsize::new(0),
            used_bytes: AtomicUsize::new(0),
            slab_allocs: AtomicU64::new(0),
            buddy_allocs: AtomicU64::new(0),
        }
    }

    /// Aloca memória
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout.size().max(layout.align());

        let ptr = if size <= self.slab_max_size {
            // Usa slab
            let result = self.slab.alloc(size, layout.align());
            if result.is_some() {
                self.slab_allocs.fetch_add(1, Ordering::Relaxed);
            }
            result
        } else {
            // Usa buddy
            let result = self.buddy.alloc(layout);
            if result.is_some() {
                self.buddy_allocs.fetch_add(1, Ordering::Relaxed);
            }
            result
        };

        if ptr.is_some() {
            self.total_allocs.fetch_add(1, Ordering::Relaxed);
            self.active_allocs.fetch_add(1, Ordering::Relaxed);
            self.used_bytes.fetch_add(size, Ordering::Relaxed);
        }

        ptr
    }

    /// Libera memória
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = layout.size().max(layout.align());
        // Todo: Revisar
        #[allow(unused)]
        let addr = ptr.as_ptr() as u64;

        if size <= self.slab_max_size {
            self.slab.dealloc(ptr, size);
        } else {
            self.buddy.dealloc(ptr, layout);
        }

        self.total_frees.fetch_add(1, Ordering::Relaxed);
        self.active_allocs.fetch_sub(1, Ordering::Relaxed);
        self.used_bytes.fetch_sub(size, Ordering::Relaxed);
    }

    /// Realoca memória
    pub fn realloc(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_size: usize,
    ) -> Option<NonNull<u8>> {
        let new_layout = Layout::from_size_align(new_size, old_layout.align()).ok()?;

        // Se cabe no mesmo slot, não precisa mover
        let old_size = old_layout.size().max(old_layout.align());
        if old_size <= self.slab_max_size && new_size <= self.slab_max_size {
            let old_class = self.slab.size_class(old_size);
            let new_class = self.slab.size_class(new_size);
            if old_class == new_class {
                // Mesmo slab class, só atualiza contadores
                self.used_bytes
                    .fetch_add(new_size.saturating_sub(old_size), Ordering::Relaxed);
                return Some(ptr);
            }
        }

        // Precisa alocar novo e copiar
        let new_ptr = self.alloc(new_layout)?;

        unsafe {
            core::ptr::copy_nonoverlapping(
                ptr.as_ptr(),
                new_ptr.as_ptr(),
                old_layout.size().min(new_size),
            );
        }

        self.dealloc(ptr, old_layout);

        Some(new_ptr)
    }

    /// Estende o heap
    pub fn extend(&mut self, additional: usize) {
        self.size += additional;
        self.buddy.extend(additional);
    }

    /// Retorna end do heap
    pub fn end(&self) -> VirtAddr {
        self.base + self.size as u64
    }

    /// Retorna estatísticas
    pub fn stats(&self) -> HeapStats {
        HeapStats {
            total_bytes: self.size,
            used_bytes: self.used_bytes.load(Ordering::Relaxed),
            free_bytes: self.size - self.used_bytes.load(Ordering::Relaxed),
            active_allocs: self.active_allocs.load(Ordering::Relaxed),
            total_allocs: self.total_allocs.load(Ordering::Relaxed),
            total_frees: self.total_frees.load(Ordering::Relaxed),
            slab_allocs: self.slab_allocs.load(Ordering::Relaxed),
            buddy_allocs: self.buddy_allocs.load(Ordering::Relaxed),
        }
    }
}

// =============================================================================
// LockedHeap
// =============================================================================

/// Wrapper thread-safe para HeapAllocator
///
/// Implementa `GlobalAlloc` para uso como `#[global_allocator]`.
pub struct LockedHeap {
    inner: Spinlock<HeapInner>,
}

struct HeapInner {
    heap_start: *mut u8,
    heap_size: usize,
    allocator: Option<SimpleAllocator>,
}

/// Allocator simples para bootstrap (antes do HeapAllocator completo)
struct SimpleAllocator {
    next: *mut u8,
    end: *mut u8,
    allocations: usize,
}

impl LockedHeap {
    /// Cria heap vazio (não inicializado)
    pub const fn empty() -> Self {
        Self {
            inner: Spinlock::new(HeapInner {
                heap_start: core::ptr::null_mut(),
                heap_size: 0,
                allocator: None,
            }),
        }
    }

    /// Inicializa o heap
    pub fn init(&self, heap_start: *mut u8, heap_size: usize) {
        let mut inner = self.inner.lock();
        inner.heap_start = heap_start;
        inner.heap_size = heap_size;
        inner.allocator = Some(SimpleAllocator {
            next: heap_start,
            end: unsafe { heap_start.add(heap_size) },
            allocations: 0,
        });
    }

    /// Verifica se está inicializado
    pub fn is_initialized(&self) -> bool {
        let inner = self.inner.lock();
        inner.allocator.is_some()
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = self.inner.lock();

        if let Some(ref mut allocator) = inner.allocator {
            let size = layout.size();
            let align = layout.align();

            // Alinha o ponteiro
            let aligned = (allocator.next as usize + align - 1) & !(align - 1);
            let new_next = aligned + size;

            if new_next <= allocator.end as usize {
                allocator.next = new_next as *mut u8;
                allocator.allocations += 1;
                return aligned as *mut u8;
            }
        }

        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // SimpleAllocator não suporta dealloc real
        // Os bytes são "perdidos" até reset do heap
        // Isso é aceitável para boot, depois o HeapAllocator completo assume

        let mut inner = self.inner.lock();
        if let Some(ref mut allocator) = inner.allocator {
            allocator.allocations = allocator.allocations.saturating_sub(1);
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Implementação simples: aloca novo, copia, "libera" antigo
        let new_layout = match Layout::from_size_align(new_size, layout.align()) {
            Ok(l) => l,
            Err(_) => return core::ptr::null_mut(),
        };

        let new_ptr = self.alloc(new_layout);
        if !new_ptr.is_null() {
            core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
            self.dealloc(ptr, layout);
        }
        new_ptr
    }
}

// Safety: LockedHeap usa lock interno
unsafe impl Send for LockedHeap {}
unsafe impl Sync for LockedHeap {}

// Safety: HeapInner é protegido por lock
unsafe impl Send for HeapInner {}
