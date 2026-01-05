//! # Slab Allocator
//!
//! Alocador para objetos pequenos de tamanho fixo.

use crate::rmm::config::{PAGE_SIZE, SLAB_CANARY, SLAB_MAX_SIZE, SLAB_MIN_SIZE, SLAB_SIZE_CLASSES};

/// Slab allocator com classes de tamanho
pub struct SlabAllocator {
    /// Caches por classe de tamanho (8, 16, 32, 64, 128, 256, 512, 1024, 2048)
    caches: [SlabCache; SLAB_SIZE_CLASSES],
}

/// Cache para um tamanho específico
struct SlabCache {
    /// Tamanho dos objetos
    obj_size: usize,
    /// Lista de slabs parcialmente usados
    partial: Option<*mut Slab>,
    /// Lista de slabs vazios
    empty: Option<*mut Slab>,
}

/// Um slab (uma página com objetos)
struct Slab {
    /// Próximo slab na lista
    next: Option<*mut Slab>,
    /// Free list dentro do slab
    free: Option<*mut SlabObject>,
    /// Contagem de objetos usados
    used: usize,
}

/// Header de objeto com canary
#[repr(C)]
struct SlabObject {
    canary: u64,
    next: Option<*mut SlabObject>,
}

impl SlabAllocator {
    pub const fn new() -> Self {
        const EMPTY_CACHE: SlabCache = SlabCache::empty();
        Self {
            caches: [EMPTY_CACHE; SLAB_SIZE_CLASSES],
        }
    }

    pub fn init(&mut self) {
        let sizes = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
        for (i, &size) in sizes.iter().enumerate() {
            self.caches[i].obj_size = size;
        }
    }

    /// Aloca objeto
    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        if size > SLAB_MAX_SIZE {
            return None;
        }
        let class = Self::size_to_class(size)?;
        self.caches[class].alloc()
    }

    /// Libera objeto
    pub fn free(&mut self, ptr: *mut u8, size: usize) {
        if let Some(class) = Self::size_to_class(size) {
            self.caches[class].free(ptr);
        }
    }

    /// Mapeia tamanho para classe
    fn size_to_class(size: usize) -> Option<usize> {
        let size = core::cmp::max(size, SLAB_MIN_SIZE);
        match size {
            0..=8 => Some(0),
            9..=16 => Some(1),
            17..=32 => Some(2),
            33..=64 => Some(3),
            65..=128 => Some(4),
            129..=256 => Some(5),
            257..=512 => Some(6),
            513..=1024 => Some(7),
            1025..=2048 => Some(8),
            _ => None,
        }
    }
}

impl SlabCache {
    const fn empty() -> Self {
        Self {
            obj_size: 0,
            partial: None,
            empty: None,
        }
    }

    fn alloc(&mut self) -> Option<*mut u8> {
        // TODO: Implementar alocação de slab
        None
    }

    fn free(&mut self, ptr: *mut u8) {
        // TODO: Implementar liberação de slab
    }
}
