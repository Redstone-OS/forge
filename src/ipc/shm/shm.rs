//! Memória compartilhada

use crate::mm::{PhysFrame, VirtAddr};
use crate::sync::Spinlock;
use alloc::vec::Vec;

/// ID de região compartilhada
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShmId(u64);

/// Região de memória compartilhada
pub struct SharedMemory {
    id: ShmId,
    /// Frames físicos (compartilhados entre processos)
    frames: Vec<PhysFrame>,
    /// Tamanho em bytes
    size: usize,
    /// Contagem de referência
    ref_count: Spinlock<u32>,
}

impl SharedMemory {
    /// Cria região compartilhada
    pub fn create(size: usize) -> Result<Self, ShmError> {
        // Calcular número de frames necessários
        let page_size = crate::arch::PAGE_SIZE;
        let num_frames = (size + page_size - 1) / page_size;
        
        // Alocar frames
        let mut frames = Vec::with_capacity(num_frames);
        for _ in 0..num_frames {
            // TODO: alocar do PMM
            // frames.push(crate::mm::pmm::alloc_frame()?);
        }
        
        Ok(Self {
            id: ShmId(0), // TODO: gerar ID único
            frames,
            size,
            ref_count: Spinlock::new(1),
        })
    }
    
    /// Mapeia no address space
    pub fn map(&self, _vaddr: VirtAddr) -> Result<(), ShmError> {
        // TODO: mapear frames no VMM
        Ok(())
    }
    
    /// Desmapeia
    pub fn unmap(&self, _vaddr: VirtAddr) -> Result<(), ShmError> {
        // TODO: desmapear
        Ok(())
    }
}

#[derive(Debug)]
pub enum ShmError {
    OutOfMemory,
    InvalidAddress,
    NotMapped,
}
