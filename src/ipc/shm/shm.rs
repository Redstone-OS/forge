//! # Shared Memory Implementation
//!
//! Implementação completa de memória compartilhada.

use crate::mm::pmm::{FRAME_ALLOCATOR, FRAME_SIZE};
use crate::mm::vmm::{map_page_with_pmm, MapFlags};
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

// ============================================================================
// TIPOS
// ============================================================================

/// ID único de região compartilhada
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShmId(pub u64);

impl ShmId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Região de memória compartilhada
pub struct SharedMemory {
    /// ID único
    pub id: ShmId,
    /// Frames físicos (endereços)
    pub frames: Vec<PhysAddr>,
    /// Tamanho em bytes
    pub size: usize,
    /// Contagem de referência
    pub ref_count: u32,
}

impl SharedMemory {
    /// Cria região compartilhada com frames alocados
    pub fn create(id: ShmId, size: usize) -> Result<Self, ShmError> {
        let num_frames = (size + FRAME_SIZE as usize - 1) / FRAME_SIZE as usize;

        let mut frames = Vec::with_capacity(num_frames);
        let pmm = FRAME_ALLOCATOR.lock();

        for _ in 0..num_frames {
            if let Some(frame_addr) = pmm.allocate_frame() {
                // Zerar o frame
                unsafe {
                    let ptr = frame_addr.as_u64() as *mut u8;
                    for i in 0..FRAME_SIZE as usize {
                        ptr.add(i).write_volatile(0);
                    }
                }
                frames.push(frame_addr);
            } else {
                // Liberar frames já alocados
                for f in frames {
                    pmm.deallocate_frame(f);
                }
                return Err(ShmError::OutOfMemory);
            }
        }

        Ok(Self {
            id,
            frames,
            size,
            ref_count: 1,
        })
    }

    /// Mapeia a região no address space do processo atual
    pub fn map(&self, base_vaddr: u64) -> Result<VirtAddr, ShmError> {
        let mut pmm = FRAME_ALLOCATOR.lock();
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE | MapFlags::USER;

        for (i, frame_addr) in self.frames.iter().enumerate() {
            let vaddr = base_vaddr + (i as u64 * FRAME_SIZE);
            let phys = frame_addr.as_u64();

            if let Err(_) = map_page_with_pmm(vaddr, phys, flags, &mut *pmm) {
                return Err(ShmError::MapFailed);
            }
        }

        Ok(VirtAddr::new(base_vaddr))
    }

    /// Retorna tamanho em bytes
    pub fn size(&self) -> usize {
        self.size
    }
}

// ============================================================================
// REGISTRY GLOBAL
// ============================================================================

/// Registry global de regiões SHM
pub struct ShmRegistry {
    regions: BTreeMap<ShmId, SharedMemory>,
    next_id: u64,
}

impl ShmRegistry {
    pub const fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Cria nova região SHM
    pub fn create(&mut self, size: usize) -> Result<ShmId, ShmError> {
        let id = ShmId(self.next_id);
        self.next_id += 1;

        let shm = SharedMemory::create(id, size)?;
        self.regions.insert(id, shm);

        Ok(id)
    }

    /// Obtém região por ID
    pub fn get(&self, id: ShmId) -> Option<&SharedMemory> {
        self.regions.get(&id)
    }

    /// Obtém região mutável por ID
    pub fn get_mut(&mut self, id: ShmId) -> Option<&mut SharedMemory> {
        self.regions.get_mut(&id)
    }

    /// Incrementa ref count
    pub fn add_ref(&mut self, id: ShmId) -> bool {
        if let Some(shm) = self.regions.get_mut(&id) {
            shm.ref_count += 1;
            true
        } else {
            false
        }
    }

    /// Remove referência e possivelmente libera
    pub fn release(&mut self, id: ShmId) {
        let should_free = if let Some(shm) = self.regions.get_mut(&id) {
            shm.ref_count = shm.ref_count.saturating_sub(1);
            shm.ref_count == 0
        } else {
            false
        };

        if should_free {
            if let Some(shm) = self.regions.remove(&id) {
                // Liberar frames
                let pmm = FRAME_ALLOCATOR.lock();
                for frame_addr in shm.frames {
                    pmm.deallocate_frame(frame_addr);
                }
            }
        }
    }
}

/// Registry global (protegido por spinlock)
pub static SHM_REGISTRY: Spinlock<ShmRegistry> = Spinlock::new(ShmRegistry::new());

// ============================================================================
// ERROS
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum ShmError {
    OutOfMemory,
    InvalidId,
    MapFailed,
    NotMapped,
}
