//! # Buffer Manager
//!
//! Gerenciador de buffers de display.
//!
//! Responsável por alocar, mapear e liberar buffers para renderização
//! e composição gráfica.

use crate::mm::pmm::{FRAME_ALLOCATOR, FRAME_SIZE};
use crate::mm::vmm::{map_page_with_pmm, MapFlags};
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use gfx_types::{BufferDescriptor, BufferHandle};

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
    /// Retorna ponteiro para o buffer (kernel space).
    pub fn as_ptr(&self) -> *const u8 {
        // Assumindo identity mapping ou mapeamento direto
        self.phys_addr.as_u64() as *const u8
    }

    /// Retorna ponteiro mutável para o buffer (kernel space).
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.phys_addr.as_u64() as *mut u8
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
    pub fn create(&mut self, desc: BufferDescriptor) -> Result<BufferHandle, BufferError> {
        let size_bytes = desc.size_bytes();
        let num_frames = (size_bytes + FRAME_SIZE as usize - 1) / FRAME_SIZE as usize;

        // Alocar frames físicos
        let pmm = FRAME_ALLOCATOR.lock();

        // Alocar primeiro frame
        let first_frame = pmm.allocate_frame().ok_or(BufferError::OutOfMemory)?;

        // Para buffers maiores que um frame, aloca mais frames
        // NOTA: Em produção, deveria alocar frames contíguos
        let mut allocated_frames = Vec::with_capacity(num_frames);
        allocated_frames.push(first_frame);

        for _ in 1..num_frames {
            if let Some(frame) = pmm.allocate_frame() {
                allocated_frames.push(frame);
            } else {
                // Liberar frames já alocados
                for f in allocated_frames {
                    pmm.deallocate_frame(f);
                }
                return Err(BufferError::OutOfMemory);
            }
        }

        // Zerar o primeiro frame (suficiente para buffers pequenos)
        unsafe {
            let ptr = first_frame.as_u64() as *mut u8;
            core::ptr::write_bytes(ptr, 0, FRAME_SIZE as usize);
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
    pub fn map(&mut self, handle: BufferHandle, vaddr: u64) -> Result<VirtAddr, BufferError> {
        let buffer = self
            .buffers
            .get_mut(&handle.0)
            .ok_or(BufferError::InvalidHandle)?;

        if buffer.mapped_vaddr.is_some() {
            return Err(BufferError::AlreadyMapped);
        }

        let num_frames = (buffer.desc.size_bytes() + FRAME_SIZE as usize - 1) / FRAME_SIZE as usize;

        let mut pmm = FRAME_ALLOCATOR.lock();
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE | MapFlags::USER;

        // Mapear cada frame
        // NOTA: Isso assume que os frames são contíguos a partir de phys_addr
        for i in 0..num_frames {
            let frame_vaddr = vaddr + (i as u64 * FRAME_SIZE);
            let frame_paddr = buffer.phys_addr.as_u64() + (i as u64 * FRAME_SIZE);

            if let Err(_) = map_page_with_pmm(frame_vaddr, frame_paddr, flags, &mut *pmm) {
                return Err(BufferError::MapFailed);
            }
        }

        buffer.mapped_vaddr = Some(VirtAddr::new(vaddr));

        crate::ktrace!("(BufferMgr) Buffer mapeado em:", vaddr);

        Ok(VirtAddr::new(vaddr))
    }

    /// Libera um buffer.
    pub fn destroy(&mut self, handle: BufferHandle) -> Result<(), BufferError> {
        let buffer = self
            .buffers
            .remove(&handle.0)
            .ok_or(BufferError::InvalidHandle)?;

        // Liberar frames físicos
        let pmm = FRAME_ALLOCATOR.lock();
        pmm.deallocate_frame(buffer.phys_addr);

        crate::ktrace!("(BufferMgr) Buffer destruído:", handle.0);

        Ok(())
    }
}

// ============================================================================
// GLOBAL INSTANCE
// ============================================================================

/// Gerenciador global de buffers.
pub static BUFFER_MANAGER: Spinlock<BufferManager> = Spinlock::new(BufferManager::new());
