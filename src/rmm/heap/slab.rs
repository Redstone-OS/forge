//! # Slab Allocator
//!
//! Allocator para objetos pequenos (8B - 2KB) usando slab caches.
//!
//! ## Slab Design
//!
//! Cada slab é uma página (4KB) dividida em objetos de tamanho fixo.
//! Objetos livres formam uma freelist intrusiva.
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                         Slab (4KB)                         │
//! ├────────────────────────────────────────────────────────────┤
//! │ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐     │
//! │ │ Object │ │ Object │ │ Object │ │ Object │ │ Object │ ... │
//! │ │  64B   │ │  64B   │ │  64B   │ │  64B   │ │  64B   │     │
//! │ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘     │
//! │     ↓          ↓                      ↑                    │
//! │   freelist ───────────────────────────┘                    │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Size Classes
//!
//! - 8, 16, 32, 64, 128, 256, 512, 1024, 2048 bytes
//! - Objeto é promovido para próxima classe se não cabe
//!
//! ## Cache per-CPU (futuro)
//!
//! Para evitar contenção, cada CPU terá cache local de objetos.

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::*;

use core::ptr::NonNull;

/// Classes de tamanho
const SIZE_CLASSES: [usize; SLAB_SIZE_CLASSES] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Objeto livre (freelist intrusiva)
#[repr(C)]
struct FreeObject {
    next: *mut FreeObject,
}

/// Metadados de um slab
struct SlabMeta {
    /// Endereço base do slab
    base: VirtAddr,
    /// Tamanho do objeto
    object_size: usize,
    /// Número total de objetos
    total_objects: usize,
    /// Número de objetos livres
    free_objects: usize,
    /// Cabeça da freelist
    freelist: *mut FreeObject,
    /// Próximo slab com objetos livres
    next: *mut SlabMeta,
    /// Slab anterior
    prev: *mut SlabMeta,
}

impl SlabMeta {
    /// Cria novo slab
    fn new(base: VirtAddr, object_size: usize) -> Self {
        let total_objects = PAGE_SIZE / object_size;

        let mut slab = Self {
            base,
            object_size,
            total_objects,
            free_objects: total_objects,
            freelist: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        };

        // Inicializa freelist
        slab.init_freelist();

        slab
    }

    /// Inicializa freelist com todos os objetos
    fn init_freelist(&mut self) {
        let base = self.base.as_u64() as *mut FreeObject;

        for i in (0..self.total_objects).rev() {
            let obj =
                unsafe { base.add(i * self.object_size / core::mem::size_of::<FreeObject>()) };
            unsafe {
                (*obj).next = self.freelist;
            }
            self.freelist = obj;
        }

        self.free_objects = self.total_objects;
    }

    /// Aloca um objeto
    fn alloc(&mut self) -> Option<*mut u8> {
        if self.freelist.is_null() {
            return None;
        }

        let obj = self.freelist;
        unsafe {
            self.freelist = (*obj).next;
        }
        self.free_objects -= 1;

        Some(obj as *mut u8)
    }

    /// Libera um objeto
    fn free(&mut self, ptr: *mut u8) {
        let obj = ptr as *mut FreeObject;
        unsafe {
            (*obj).next = self.freelist;
        }
        self.freelist = obj;
        self.free_objects += 1;
    }

    /// Slab está cheio?
    fn is_full(&self) -> bool {
        self.free_objects == 0
    }

    /// Slab está vazio?
    // Todo: Revisar
    #[allow(unused)]
    fn is_empty(&self) -> bool {
        self.free_objects == self.total_objects
    }

    /// Contém este endereço?
    fn contains(&self, ptr: *mut u8) -> bool {
        let addr = ptr as u64;
        let base = self.base.as_u64();
        addr >= base && addr < base + PAGE_SIZE as u64
    }
}

/// Cache de uma classe de tamanho
struct SlabCache {
    /// Tamanho do objeto
    // Todo: Revisar
    #[allow(unused)]
    object_size: usize,
    /// Lista de slabs não-cheios
    partial: *mut SlabMeta,
    /// Lista de slabs cheios
    full: *mut SlabMeta,
    /// Número de slabs
    slab_count: usize,
    /// Número de objetos alocados
    allocated: usize,
}

impl SlabCache {
    /// Cria cache vazio
    const fn empty(object_size: usize) -> Self {
        Self {
            object_size,
            partial: core::ptr::null_mut(),
            full: core::ptr::null_mut(),
            slab_count: 0,
            allocated: 0,
        }
    }

    /// Aloca objeto desta classe
    fn alloc(&mut self) -> Option<*mut u8> {
        // Tenta alocar de slab parcial
        if !self.partial.is_null() {
            let slab = unsafe { &mut *self.partial };
            let ptr = slab.alloc()?;
            self.allocated += 1;

            // Se slab ficou cheio, move para lista de cheios
            if slab.is_full() {
                self.move_to_full(slab);
            }

            return Some(ptr);
        }

        // Retorna None - caller deve criar slab e chamar novamente
        None
    }

    /// Adiciona slab e aloca imediatamente
    fn alloc_from_new_slab(&mut self, slab: *mut SlabMeta) -> Option<*mut u8> {
        self.add_partial(slab);

        // Agora aloca
        let slab = unsafe { &mut *self.partial };
        let ptr = slab.alloc()?;
        self.allocated += 1;

        Some(ptr)
    }

    /// Libera objeto
    fn free(&mut self, ptr: *mut u8) -> bool {
        // Procura em slabs parciais
        let mut slab_ptr = self.partial;
        while !slab_ptr.is_null() {
            let slab = unsafe { &mut *slab_ptr };
            if slab.contains(ptr) {
                slab.free(ptr);
                self.allocated -= 1;
                return true;
            }
            slab_ptr = slab.next;
        }

        // Procura em slabs cheios
        slab_ptr = self.full;
        while !slab_ptr.is_null() {
            let slab = unsafe { &mut *slab_ptr };
            if slab.contains(ptr) {
                slab.free(ptr);
                self.allocated -= 1;
                // Move para parcial
                self.move_to_partial(slab);
                return true;
            }
            slab_ptr = slab.next;
        }

        false
    }

    /// Adiciona slab à lista de parciais
    fn add_partial(&mut self, slab: *mut SlabMeta) {
        unsafe {
            (*slab).next = self.partial;
            (*slab).prev = core::ptr::null_mut();

            if !self.partial.is_null() {
                (*self.partial).prev = slab;
            }

            self.partial = slab;
        }
        self.slab_count += 1;
    }

    /// Move slab de parcial para cheio
    fn move_to_full(&mut self, slab: &mut SlabMeta) {
        // Remove de partial
        unsafe {
            if !slab.prev.is_null() {
                (*slab.prev).next = slab.next;
            } else {
                self.partial = slab.next;
            }

            if !slab.next.is_null() {
                (*slab.next).prev = slab.prev;
            }
        }

        // Adiciona a full
        unsafe {
            slab.next = self.full;
            slab.prev = core::ptr::null_mut();

            if !self.full.is_null() {
                (*self.full).prev = slab as *mut SlabMeta;
            }

            self.full = slab as *mut SlabMeta;
        }
    }

    /// Move slab de cheio para parcial
    fn move_to_partial(&mut self, slab: &mut SlabMeta) {
        // Remove de full
        unsafe {
            if !slab.prev.is_null() {
                (*slab.prev).next = slab.next;
            } else {
                self.full = slab.next;
            }

            if !slab.next.is_null() {
                (*slab.next).prev = slab.prev;
            }
        }

        // Adiciona a partial
        self.add_partial(slab as *mut SlabMeta);
        self.slab_count -= 1; // add_partial incrementa, então compensa
    }
}

/// Slab Allocator principal
pub struct SlabAllocator {
    /// Base do heap para slabs
    base: VirtAddr,
    /// Tamanho disponível
    size: usize,
    /// Próximo endereço livre para criar slab
    next_slab: VirtAddr,
    /// Caches por classe de tamanho
    caches: [SlabCache; SLAB_SIZE_CLASSES],
    /// Metadados de slabs (alocados do início do heap)
    meta_pool: VirtAddr,
    /// Próximo meta livre
    next_meta: usize,
    /// Total de slabs criados
    total_slabs: usize,
}

impl SlabAllocator {
    /// Cria novo slab allocator
    pub fn new(base: VirtAddr, size: usize) -> Self {
        // Reserva início para metadados
        let meta_size = 4096 * 16; // 64KB para metadados
        let meta_pool = base;
        let slab_base = base + meta_size as u64;
        let slab_size = size - meta_size;

        Self {
            base: slab_base,
            size: slab_size,
            next_slab: slab_base,
            caches: [
                SlabCache::empty(8),
                SlabCache::empty(16),
                SlabCache::empty(32),
                SlabCache::empty(64),
                SlabCache::empty(128),
                SlabCache::empty(256),
                SlabCache::empty(512),
                SlabCache::empty(1024),
                SlabCache::empty(2048),
            ],
            meta_pool,
            next_meta: 0,
            total_slabs: 0,
        }
    }

    /// Aloca objeto de tamanho específico
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let effective_size = size.max(align);
        let class = self.size_class(effective_size);

        if class >= SLAB_SIZE_CLASSES {
            return None; // Muito grande para slab
        }

        // Tenta alocar de cache existente
        if let Some(ptr) = self.caches[class].alloc() {
            return NonNull::new(ptr);
        }

        // Precisa criar novo slab
        let object_size = SIZE_CLASSES[class];
        let slab = self.create_slab(object_size)?;
        let ptr = self.caches[class].alloc_from_new_slab(slab)?;
        NonNull::new(ptr)
    }

    /// Libera objeto
    pub fn dealloc(&mut self, ptr: NonNull<u8>, size: usize) {
        let class = self.size_class(size);

        if class < SLAB_SIZE_CLASSES {
            self.caches[class].free(ptr.as_ptr());
        }
    }

    /// Retorna índice da classe de tamanho
    pub fn size_class(&self, size: usize) -> usize {
        for (i, &class_size) in SIZE_CLASSES.iter().enumerate() {
            if size <= class_size {
                return i;
            }
        }
        SLAB_SIZE_CLASSES // Indica que é muito grande
    }

    /// Cria novo slab para uma classe de tamanho
    fn create_slab(&mut self, object_size: usize) -> Option<*mut SlabMeta> {
        // Verifica se há espaço
        if self.next_slab.as_u64() + PAGE_SIZE as u64 > self.base.as_u64() + self.size as u64 {
            return None;
        }

        // Aloca metadados
        let meta = self.alloc_meta()?;

        // Inicializa slab
        let slab_addr = self.next_slab;
        self.next_slab = self.next_slab + PAGE_SIZE as u64;

        unsafe {
            core::ptr::write(meta, SlabMeta::new(slab_addr, object_size));
        }

        self.total_slabs += 1;

        Some(meta)
    }

    /// Aloca metadados de slab
    fn alloc_meta(&mut self) -> Option<*mut SlabMeta> {
        let meta_size = core::mem::size_of::<SlabMeta>();
        let max_metas = (4096 * 16) / meta_size;

        if self.next_meta >= max_metas {
            return None;
        }

        let ptr = self.meta_pool.as_u64() + (self.next_meta * meta_size) as u64;
        self.next_meta += 1;

        Some(ptr as *mut SlabMeta)
    }

    /// Estatísticas
    pub fn stats(&self) -> SlabStats {
        let mut total_allocated = 0;
        let mut total_slabs = 0;

        for cache in &self.caches {
            total_allocated += cache.allocated;
            total_slabs += cache.slab_count;
        }

        SlabStats {
            total_slabs,
            total_allocated,
            slab_memory: total_slabs * PAGE_SIZE,
        }
    }
}

/// Estatísticas do slab allocator
#[derive(Debug, Default)]
pub struct SlabStats {
    pub total_slabs: usize,
    pub total_allocated: usize,
    pub slab_memory: usize,
}

// Safety: SlabAllocator é protegido por lock externo
unsafe impl Send for SlabAllocator {}
