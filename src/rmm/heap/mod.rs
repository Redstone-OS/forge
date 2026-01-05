//! # Kernel Heap
//!
//! Heap do kernel com Buddy Allocator + Slab Allocator.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                     Kernel Heap                            │
//! ├────────────────────────────────────────────────────────────┤
//! │                                                            │
//! │  ┌─────────────┐     ┌─────────────────────────────────┐   │
//! │  │ GlobalAlloc │────►│ LockedHeap                      │   │
//! │  │ (Rust API)  │     │ (wrapper thread-safe)           │   │
//! │  └─────────────┘     └──────────────┬──────────────────┘   │
//! │                                     │                      │
//! │                    ┌────────────────┴────────────────┐     │
//! │                    ▼                                 ▼     │
//! │  ┌─────────────────────────┐   ┌────────────────────────┐  │
//! │  │     Slab Allocator      │   │    Buddy Allocator     │  │
//! │  │  (objetos pequenos)     │   │   (alocações grandes)  │  │
//! │  │  8B - 2KB               │   │   4KB+                 │  │
//! │  └─────────────────────────┘   └────────────────────────┘  │
//! │                    │                         │             │
//! │                    └────────────┬────────────┘             │
//! │                                 ▼                          │
//! │                    ┌────────────────────────┐              │
//! │                    │    phys::alloc()       │              │
//! │                    │    (frames físicos)    │              │
//! │                    └────────────────────────┘              │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Slab Allocator
//!
//! Para objetos pequenos (8B - 2KB):
//! - Classes: 8, 16, 32, 64, 128, 256, 512, 1024, 2048 bytes
//! - Cada slab é uma página com freelists
//! - Cache por tamanho para alocação O(1)
//!
//! ## Buddy Allocator
//!
//! Para alocações grandes (> 2KB):
//! - Ordens: 0 (4KB) até 10 (4MB)
//! - Merge de blocos adjacentes na liberação
//! - Split de blocos maiores quando necessário

pub mod allocator;
pub mod aslr;
pub mod buddy;
pub mod slab;

pub use allocator::{HeapAllocator, LockedHeap};
pub use aslr::generate_aslr_offset;
pub use buddy::BuddyAllocator;
pub use slab::SlabAllocator;

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::*;
use crate::rmm::error::RmmResult;
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
// Todo: Revisar
#[allow(unused)]
use crate::rmm::virt::{hhdm, mapper, MapFlags};
use crate::rmm::zone::Zone;

// Todo: Revisar
#[allow(unused)]
use core::alloc::{GlobalAlloc, Layout};
#[allow(unused)]
use core::ptr::NonNull;

/// Heap global (unsafe static para GlobalAlloc)
static mut KERNEL_HEAP: Option<HeapAllocator> = None;

/// LockedHeap como GlobalAllocator
#[global_allocator]
pub static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Inicializa o heap do kernel
///
/// # Safety
///
/// Deve ser chamada apenas uma vez, após HHDM e phys estarem inicializados.
pub unsafe fn init() {
    crate::kinfo!("(RMM/Heap) Inicializando kernel heap...");

    // Calcula base do heap com ASLR
    let aslr_offset = aslr::generate_aslr_offset();
    let heap_start = VirtAddr::new(HEAP_BASE + aslr_offset as u64);

    crate::kinfo!(
        "(RMM/Heap) Base: 0x{:x}, ASLR offset: 0x{:x}",
        heap_start.as_u64(),
        aslr_offset
    );

    // Aloca páginas iniciais para o heap
    let initial_pages = HEAP_INITIAL_SIZE / PAGE_SIZE;

    for i in 0..initial_pages {
        let virt = heap_start + (i * PAGE_SIZE) as u64;

        // Aloca frame físico
        let phys = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
            .expect("Failed to allocate heap pages");

        // Mapeia
        mapper::map_page(virt, phys, MapFlags::KERNEL_RW).expect("Failed to map heap pages");
    }

    crate::kinfo!(
        "(RMM/Heap) Mapeado {} KB iniciais",
        HEAP_INITIAL_SIZE / 1024
    );

    // Cria o allocator
    let allocator = HeapAllocator::new(heap_start, HEAP_INITIAL_SIZE);

    KERNEL_HEAP = Some(allocator);

    // Inicializa o LockedHeap global
    GLOBAL_ALLOCATOR.init(heap_start.as_u64() as *mut u8, HEAP_INITIAL_SIZE);

    crate::kinfo!("(RMM/Heap) Kernel heap inicializado!");
}

/// Expande o heap (quando necessário)
pub fn expand_heap(additional_bytes: usize) -> RmmResult<()> {
    let allocator =
        unsafe { KERNEL_HEAP.as_mut() }.ok_or(crate::rmm::error::RmmError::NotInitialized)?;

    let pages_needed = (additional_bytes + PAGE_SIZE - 1) / PAGE_SIZE;
    let current_end = allocator.end();

    for i in 0..pages_needed {
        let virt = current_end + (i * PAGE_SIZE) as u64;

        // Aloca frame
        let phys = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
            .ok_or(crate::rmm::error::RmmError::OutOfMemory)?;

        // Mapeia
        mapper::map_page(virt, phys, MapFlags::KERNEL_RW)?;
    }

    allocator.extend(pages_needed * PAGE_SIZE);

    Ok(())
}

/// Retorna estatísticas do heap
pub fn stats() -> HeapStats {
    unsafe {
        KERNEL_HEAP
            .as_ref()
            .map(|h| h.stats())
            .unwrap_or(HeapStats::default())
    }
}

/// Estatísticas do heap
#[derive(Debug, Clone, Copy, Default)]
pub struct HeapStats {
    /// Total de memória do heap
    pub total_bytes: usize,
    /// Memória usada
    pub used_bytes: usize,
    /// Memória livre
    pub free_bytes: usize,
    /// Número de alocações ativas
    pub active_allocs: usize,
    /// Total de alocações feitas
    pub total_allocs: u64,
    /// Total de liberações feitas
    pub total_frees: u64,
    /// Alocações via slab
    pub slab_allocs: u64,
    /// Alocações via buddy
    pub buddy_allocs: u64,
}

impl HeapStats {
    /// Porcentagem de uso
    pub fn usage_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Dump para log
    pub fn dump(&self) {
        crate::kinfo!("=== Heap Statistics ===");
        crate::kinfo!(
            "  Memory: {} KB total, {} KB used ({:.1}%)",
            self.total_bytes / 1024,
            self.used_bytes / 1024,
            self.usage_percent()
        );
        crate::kinfo!(
            "  Allocs: {} active, {} total, {} frees",
            self.active_allocs,
            self.total_allocs,
            self.total_frees
        );
        crate::kinfo!(
            "  By type: {} slab, {} buddy",
            self.slab_allocs,
            self.buddy_allocs
        );
    }
}
