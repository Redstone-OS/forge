//! # Buffer Manager
//!
//! Gerenciador de buffers de display.
//!
//! Responsável por alocar, mapear e liberar buffers para renderização
//! e composição gráfica.
//!
//! # TODO
//!
//! Este módulo precisa ser refatorado para usar as novas APIs do RMM.
//! Por enquanto, está desabilitado até que phys::alloc e mapper sejam
//! expostos adequadamente.

use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::config::PAGE_SIZE;
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use gfx_types::{BufferDescriptor, BufferHandle};

// Todo: Revisar
#[allow(unused)]
// Constantes
const FRAME_SIZE: u64 = PAGE_SIZE as u64;

// ============================================================================
// ERRORS
// ============================================================================

/// Erros do gerenciador de buffers.
#[derive(Debug, Clone, Copy)]
pub enum BufferError {
    /// Memória insuficiente.
    OutOfMemory,
    /// Handle inválido.
    InvalidHandle,
    /// Falha ao mapear memória.
    MapFailed,
    /// Buffer já mapeado.
    AlreadyMapped,
}

// ============================================================================
// DISPLAY BUFFER
// ============================================================================

/// Buffer de display alocado.
pub struct DisplayBuffer {
    /// Handle único.
    pub handle: BufferHandle,
    /// Descritor do buffer.
    pub desc: BufferDescriptor,
    /// Endereço físico do buffer.
    pub phys_addr: PhysAddr,
    /// Endereço virtual se mapeado para userspace.
    pub mapped_vaddr: Option<VirtAddr>,
    /// Contagem de referência.
    pub ref_count: u32,
}

impl DisplayBuffer {
    /// Retorna ponteiro para o buffer (kernel space via HHDM).
    pub fn as_ptr(&self) -> *const u8 {
        crate::rmm::virt::hhdm::phys_to_virt(self.phys_addr.as_u64()) as *const u8
    }

    /// Retorna ponteiro mutável para o buffer (kernel space via HHDM).
    pub fn as_mut_ptr(&self) -> *mut u8 {
        crate::rmm::virt::hhdm::phys_to_virt(self.phys_addr.as_u64()) as *mut u8
    }
}

// ============================================================================
// BUFFER MANAGER
// ============================================================================

/// Gerenciador global de buffers.
pub struct BufferManager {
    /// Buffers alocados indexados por handle.
    buffers: BTreeMap<u64, DisplayBuffer>,
    /// Próximo handle a ser atribuído.
    next_handle: u64,
}

impl BufferManager {
    /// Cria novo gerenciador.
    pub const fn new() -> Self {
        Self {
            buffers: BTreeMap::new(),
            next_handle: 1,
        }
    }

    /// Aloca novo buffer de display.
    ///
    /// # TODO
    ///
    /// Implementar usando RMM phys::alloc quando API estiver disponível.
    pub fn create(&mut self, desc: BufferDescriptor) -> Result<BufferHandle, BufferError> {
        use crate::rmm::phys::{self, AllocFlags, FrameOwner};
        use crate::rmm::zone::Zone;

        let size_bytes = desc.size_bytes();
        let num_frames = (size_bytes + PAGE_SIZE - 1) / PAGE_SIZE;

        // Alocar primeiro frame via RMM
        let first_frame = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
            .ok_or(BufferError::OutOfMemory)?;

        // Para buffers maiores que um frame, aloca mais frames
        // TODO: Usar alloc_contiguous para frames contíguos
        let mut allocated_frames = Vec::with_capacity(num_frames);
        allocated_frames.push(first_frame);

        for _ in 1..num_frames {
            if let Some(frame) = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO) {
                allocated_frames.push(frame);
            } else {
                // Liberar frames já alocados
                for f in allocated_frames {
                    let _ = phys::free(f, FrameOwner::Kernel);
                }
                return Err(BufferError::OutOfMemory);
            }
        }

        // Criar handle
        let handle = BufferHandle(self.next_handle);
        self.next_handle += 1;

        let buffer = DisplayBuffer {
            handle,
            desc,
            phys_addr: first_frame,
            mapped_vaddr: None,
            ref_count: 1,
        };

        self.buffers.insert(handle.0, buffer);

        crate::ktrace!("(BufferMgr) Criado buffer:", handle.0);
        crate::ktrace!("(BufferMgr) Size:", size_bytes as u64);

        Ok(handle)
    }

    /// Obtém buffer por handle.
    pub fn get(&self, handle: BufferHandle) -> Option<&DisplayBuffer> {
        self.buffers.get(&handle.0)
    }

    /// Obtém buffer mutável por handle.
    pub fn get_mut(&mut self, handle: BufferHandle) -> Option<&mut DisplayBuffer> {
        self.buffers.get_mut(&handle.0)
    }

    /// Mapeia buffer para o address space do processo atual.
    ///
    /// # TODO
    ///
    /// Implementar mapeamento real usando RMM.
    pub fn map(&mut self, handle: BufferHandle, vaddr: u64) -> Result<VirtAddr, BufferError> {
        let buffer = self
            .buffers
            .get_mut(&handle.0)
            .ok_or(BufferError::InvalidHandle)?;

        if buffer.mapped_vaddr.is_some() {
            return Err(BufferError::AlreadyMapped);
        }

        // TODO: Implementar mapeamento real usando page table walk
        // Por enquanto, apenas registra o endereço virtual
        buffer.mapped_vaddr = Some(VirtAddr::new(vaddr));

        crate::ktrace!("(BufferMgr) Buffer mapeado em:", vaddr);

        Ok(VirtAddr::new(vaddr))
    }

    /// Libera um buffer.
    pub fn destroy(&mut self, handle: BufferHandle) -> Result<(), BufferError> {
        use crate::rmm::phys::{self, FrameOwner};

        let buffer = self
            .buffers
            .remove(&handle.0)
            .ok_or(BufferError::InvalidHandle)?;

        // Liberar frames físicos
        let _ = phys::free(buffer.phys_addr, FrameOwner::Kernel);

        crate::ktrace!("(BufferMgr) Buffer destruído:", handle.0);

        Ok(())
    }
}

// ============================================================================
// GLOBAL INSTANCE
// ============================================================================

/// Gerenciador global de buffers.
pub static BUFFER_MANAGER: Spinlock<BufferManager> = Spinlock::new(BufferManager::new());
