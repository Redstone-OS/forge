//! # Pinned Memory
//!
//! Alocação de memória pinned (não pode ser swapped ou movida).
//!
//! ## Uso
//!
//! Memória pinned é necessária para:
//! - Buffers DMA (dispositivo precisa de endereço fixo)
//! - Kernel stacks
//! - Estruturas de hardware (page tables, APIC, etc.)
//! - Memória compartilhada com IOMMU
//!
//! ## Lifetime
//!
//! Memória pinned não pode ser evicted pelo reclaimer.
//! Use com moderação para não fragmentar a RAM.
//!
//! ## Tracking
//!
//! O RMM trackeia todas as alocações pinned para:
//! - Evitar tentar evict
//! - Estatísticas de memória não-reclaimable
//! - Debugging de leaks

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::error::{RmmError, RmmResult};
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;
use crate::sync::Spinlock;

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Contador de páginas pinned
static PINNED_PAGES: AtomicUsize = AtomicUsize::new(0);

/// Registro de alocações pinned (para debug/tracking)
static PINNED_REGISTRY: Spinlock<BTreeMap<u64, PinnedInfo>> = Spinlock::new(BTreeMap::new());

/// Info sobre alocação pinned
#[derive(Debug, Clone)]
struct PinnedInfo {
    /// Tamanho em páginas
    pages: usize,
    /// Owner ID
    owner_id: u32,
    /// Descrição (para debug)
    #[cfg(debug_assertions)]
    // Todo: Revisar
    #[allow(unused)]
    description: &'static str,
}

// =============================================================================
// PinnedMemory
// =============================================================================

/// Região de memória pinned
///
/// Automaticamente libera ao dropar.
pub struct PinnedMemory {
    /// Endereço físico
    phys: PhysAddr,
    /// Endereço virtual
    virt: *mut u8,
    /// Tamanho em bytes
    size: usize,
    /// Owner ID
    owner_id: u32,
}

impl PinnedMemory {
    /// Endereço físico
    #[inline]
    pub fn phys_addr(&self) -> PhysAddr {
        self.phys
    }

    /// Endereço virtual
    #[inline]
    pub fn virt_addr(&self) -> *mut u8 {
        self.virt
    }

    /// Tamanho
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Slice de leitura
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.virt, self.size) }
    }

    /// Slice de escrita
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.virt, self.size) }
    }

    /// Zera a memória
    pub fn zero(&mut self) {
        unsafe {
            core::ptr::write_bytes(self.virt, 0, self.size);
        }
    }

    /// Ponteiro tipado
    pub fn as_ptr<T>(&self) -> *const T {
        self.virt as *const T
    }

    /// Ponteiro mutável tipado
    pub fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.virt as *mut T
    }
}

impl Drop for PinnedMemory {
    fn drop(&mut self) {
        let pages = self.size / PAGE_SIZE;

        // Libera todas as páginas
        for i in 0..pages {
            let frame_phys = self.phys + (i * PAGE_SIZE) as u64;
            let _ = phys::free(
                frame_phys,
                FrameOwner::Pinned {
                    owner: self.owner_id,
                },
            );
        }

        // Atualiza contador
        PINNED_PAGES.fetch_sub(pages, Ordering::Relaxed);

        // Remove do registry
        let mut registry = PINNED_REGISTRY.lock();
        registry.remove(&self.phys.as_u64());
    }
}

// Safety: PinnedMemory pode ser enviada entre threads
unsafe impl Send for PinnedMemory {}
unsafe impl Sync for PinnedMemory {}

// =============================================================================
// API Pública
// =============================================================================

/// Aloca memória pinned
///
/// # Arguments
///
/// * `size` - Tamanho em bytes (será arredondado para páginas)
/// * `owner_id` - ID do owner (driver, processo, etc.)
///
/// # Returns
///
/// Região de memória pinned
pub fn alloc_pinned(size: usize, owner_id: u32) -> RmmResult<PinnedMemory> {
    alloc_pinned_internal(size, owner_id, "unknown")
}

/// Aloca memória pinned com descrição (debug)
#[cfg(debug_assertions)]
pub fn alloc_pinned_with_desc(
    size: usize,
    owner_id: u32,
    description: &'static str,
) -> RmmResult<PinnedMemory> {
    alloc_pinned_internal(size, owner_id, description)
}

/// Implementação interna
fn alloc_pinned_internal(
    size: usize,
    owner_id: u32,
    #[allow(unused)] description: &'static str,
) -> RmmResult<PinnedMemory> {
    // Arredonda para páginas
    let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
    let alloc_size = pages * PAGE_SIZE;

    // Flags: pinned + zero
    let flags = AllocFlags::PINNED.union(AllocFlags::ZERO);

    // Aloca frames
    let phys = if pages == 1 {
        phys::alloc(FrameOwner::Pinned { owner: owner_id }, Zone::Normal, flags)
    } else {
        phys::alloc_contiguous(
            pages,
            FrameOwner::Pinned { owner: owner_id },
            Zone::Normal,
            flags,
        )
    }
    .ok_or(RmmError::OutOfMemory)?;

    let virt = hhdm::phys_to_virt(phys.as_u64()) as *mut u8;

    // Atualiza contador
    PINNED_PAGES.fetch_add(pages, Ordering::Relaxed);

    // Registra
    {
        let mut registry = PINNED_REGISTRY.lock();
        registry.insert(
            phys.as_u64(),
            PinnedInfo {
                pages,
                owner_id,
                #[cfg(debug_assertions)]
                description,
            },
        );
    }

    Ok(PinnedMemory {
        phys,
        virt,
        size: alloc_size,
        owner_id,
    })
}

/// Libera memória pinned (alternativa a drop)
pub fn free_pinned(memory: PinnedMemory) {
    drop(memory);
}

/// Aloca stack do kernel (já pinned)
pub fn alloc_kernel_stack(size: usize) -> RmmResult<PinnedMemory> {
    alloc_pinned_internal(size, 0, "kernel_stack")
}

/// Aloca página para page table (já pinned)
pub fn alloc_page_table() -> RmmResult<PinnedMemory> {
    alloc_pinned_internal(PAGE_SIZE, 0, "page_table")
}

// =============================================================================
// Estatísticas
// =============================================================================

/// Número de páginas pinned
pub fn pinned_page_count() -> usize {
    PINNED_PAGES.load(Ordering::Relaxed)
}

/// Bytes totais pinned
pub fn pinned_bytes() -> usize {
    pinned_page_count() * PAGE_SIZE
}

/// Número de alocações pinned
pub fn pinned_allocation_count() -> usize {
    PINNED_REGISTRY.lock().len()
}

/// Dump de alocações pinned (debug)
#[cfg(debug_assertions)]
pub fn dump_pinned() {
    let registry = PINNED_REGISTRY.lock();

    crate::kinfo!("=== Pinned Memory Allocations ===");
    crate::kinfo!(
        "Total: {} pages ({} KB)",
        pinned_page_count(),
        pinned_bytes() / 1024
    );

    for (phys, info) in registry.iter() {
        crate::kinfo!(
            "  phys:",
            *phys as u64,
            "pages:",
            info.pages as u64,
            "owner:",
            info.owner_id as u64
        );
    }
}

#[cfg(not(debug_assertions))]
pub fn dump_pinned() {
    crate::kinfo!("=== Pinned Memory ===");
    crate::kinfo!(
        "Total: {} pages ({} KB), {} allocations",
        pinned_page_count(),
        pinned_bytes() / 1024,
        pinned_allocation_count()
    );
}

// =============================================================================
// Pinned Pool
// =============================================================================

/// Pool de páginas pinned pré-alocadas
///
/// Para alocações frequentes onde latência importa.
pub struct PinnedPool {
    /// Páginas disponíveis
    pages: Spinlock<alloc::vec::Vec<PhysAddr>>,
    /// Capacidade máxima
    max_size: usize,
    /// Owner ID
    owner_id: u32,
}

impl PinnedPool {
    /// Cria pool vazio
    pub fn new(max_size: usize, owner_id: u32) -> Self {
        Self {
            pages: Spinlock::new(alloc::vec::Vec::with_capacity(max_size)),
            max_size,
            owner_id,
        }
    }

    /// Pré-aloca páginas
    pub fn preallocate(&self, count: usize) -> usize {
        let mut pages = self.pages.lock();
        let to_alloc = count.min(self.max_size - pages.len());
        let mut allocated = 0;

        for _ in 0..to_alloc {
            if let Some(phys) = phys::alloc(
                FrameOwner::Pinned {
                    owner: self.owner_id,
                },
                Zone::Normal,
                AllocFlags::PINNED.union(AllocFlags::ZERO),
            ) {
                pages.push(phys);
                allocated += 1;
            } else {
                break;
            }
        }

        PINNED_PAGES.fetch_add(allocated, Ordering::Relaxed);
        allocated
    }

    /// Obtém página do pool
    pub fn get(&self) -> Option<PhysAddr> {
        let mut pages = self.pages.lock();
        pages.pop()
    }

    /// Devolve página ao pool
    pub fn put(&self, phys: PhysAddr) -> bool {
        let mut pages = self.pages.lock();

        if pages.len() < self.max_size {
            pages.push(phys);
            true
        } else {
            // Pool cheio, libera
            let _ = phys::free(
                phys,
                FrameOwner::Pinned {
                    owner: self.owner_id,
                },
            );
            PINNED_PAGES.fetch_sub(1, Ordering::Relaxed);
            false
        }
    }

    /// Páginas disponíveis no pool
    pub fn available(&self) -> usize {
        self.pages.lock().len()
    }

    /// Limpa pool
    pub fn clear(&self) {
        let mut pages = self.pages.lock();
        let count = pages.len();

        for phys in pages.drain(..) {
            let _ = phys::free(
                phys,
                FrameOwner::Pinned {
                    owner: self.owner_id,
                },
            );
        }

        PINNED_PAGES.fetch_sub(count, Ordering::Relaxed);
    }
}

impl Drop for PinnedPool {
    fn drop(&mut self) {
        self.clear();
    }
}
