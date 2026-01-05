//! # Shared Memory Region Implementation
//!
//! Implementação de regiões de memória compartilhada.
//!
//! ## Estruturas Principais
//!
//! - [`SharedMemory`] - Região contendo frames físicos compartilhados
//! - [`ShmRegistry`] - Registry global para gerenciar todas as regiões
//! - [`ShmId`] - Identificador único de região
//! - [`ShmError`] - Tipos de erro

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

// =============================================================================
// ShmId
// =============================================================================

/// Identificador único de região de memória compartilhada
///
/// IDs são atribuídos sequencialmente pelo registry e nunca reutilizados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShmId(pub u64);

impl ShmId {
    /// Cria novo ShmId
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Retorna o valor numérico do ID
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

// =============================================================================
// ShmError
// =============================================================================

/// Erros de operações com memória compartilhada
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmError {
    /// Memória insuficiente para alocar frames
    OutOfMemory,
    /// ID de região inválido ou não encontrado
    InvalidId,
    /// Falha ao mapear região no address space
    MapFailed,
    /// Região não está mapeada
    NotMapped,
    /// Tamanho inválido (zero ou muito grande)
    InvalidSize,
    /// Endereço de mapeamento inválido
    InvalidAddress,
}

impl ShmError {
    /// Retorna string descritiva do erro
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OutOfMemory => "out of memory",
            Self::InvalidId => "invalid shm id",
            Self::MapFailed => "failed to map shm",
            Self::NotMapped => "shm not mapped",
            Self::InvalidSize => "invalid size",
            Self::InvalidAddress => "invalid address",
        }
    }
}

// =============================================================================
// SharedMemory
// =============================================================================

/// Região de memória compartilhada
///
/// Contém frames físicos que podem ser mapeados em múltiplos processos.
/// Os frames são alocados com `FrameOwner::Shared` para permitir
/// rastreamento pelo RMM.
pub struct SharedMemory {
    /// ID único desta região
    pub id: ShmId,
    /// Frames físicos alocados (endereços)
    frames: Vec<PhysAddr>,
    /// Tamanho total em bytes
    size: usize,
    /// Contagem de referência
    ref_count: u32,
}

impl SharedMemory {
    /// Tamanho máximo de uma região SHM (16 MB)
    pub const MAX_SIZE: usize = 16 * 1024 * 1024;

    /// Cria região compartilhada com frames alocados e zerados
    ///
    /// # Argumentos
    ///
    /// * `id` - ID único da região
    /// * `size` - Tamanho desejado em bytes (será arredondado para páginas)
    ///
    /// # Retorna
    ///
    /// * `Ok(SharedMemory)` - Região criada com sucesso
    /// * `Err(ShmError::InvalidSize)` - Tamanho zero ou maior que MAX_SIZE
    /// * `Err(ShmError::OutOfMemory)` - Sem memória disponível
    pub fn create(id: ShmId, size: usize) -> Result<Self, ShmError> {
        // Validar tamanho
        if size == 0 || size > Self::MAX_SIZE {
            return Err(ShmError::InvalidSize);
        }

        let num_frames = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        let mut frames = Vec::with_capacity(num_frames);

        // Alocar frames zerados
        for _ in 0..num_frames {
            match phys::alloc(FrameOwner::Shared, Zone::Normal, AllocFlags::ZERO) {
                Some(frame_addr) => {
                    frames.push(frame_addr);
                }
                None => {
                    // Liberar frames já alocados (rollback)
                    for f in frames {
                        let _ = phys::free(f, FrameOwner::Shared);
                    }
                    return Err(ShmError::OutOfMemory);
                }
            }
        }

        Ok(Self {
            id,
            frames,
            size,
            ref_count: 1,
        })
    }

    /// Retorna tamanho em bytes
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Retorna número de frames
    #[inline]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Retorna contagem de referência atual
    #[inline]
    pub fn ref_count(&self) -> u32 {
        self.ref_count
    }

    /// Mapeia a região no address space do processo
    ///
    /// # Argumentos
    ///
    /// * `target_cr3` - CR3 do address space alvo
    /// * `base_vaddr` - Endereço virtual base para mapeamento
    ///
    /// # Retorna
    ///
    /// * `Ok(())` - Mapeamento bem sucedido
    /// * `Err(ShmError::MapFailed)` - Falha ao mapear
    ///
    /// # Safety
    ///
    /// O caller deve garantir que `base_vaddr` é válido e não sobrepõe
    /// outras regiões mapeadas.
    pub fn map_at(&self, target_cr3: u64, base_vaddr: u64) -> Result<(), ShmError> {
        let page_size = PAGE_SIZE as u64;
        let flags: u64 = 0x7; // Present | Writable | User

        for (i, frame_addr) in self.frames.iter().enumerate() {
            let vaddr = base_vaddr + (i as u64) * page_size;
            let phys = frame_addr.as_u64();

            // Mapear página no address space alvo
            if let Err(_) = map_page_in_target(target_cr3, vaddr, phys, flags) {
                return Err(ShmError::MapFailed);
            }
        }

        Ok(())
    }

    /// Incrementa reference count
    pub(crate) fn add_ref(&mut self) {
        self.ref_count = self.ref_count.saturating_add(1);
    }

    /// Decrementa reference count e retorna se deve ser liberado
    pub(crate) fn release(&mut self) -> bool {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count == 0
    }

    /// Libera os frames físicos (chamado quando ref_count chega a zero)
    pub(crate) fn free_frames(&mut self) {
        for frame in self.frames.drain(..) {
            let _ = phys::free(frame, FrameOwner::Shared);
        }
    }
}

// =============================================================================
// ShmRegistry
// =============================================================================

/// Registry global de regiões de memória compartilhada
///
/// Gerencia criação, lookup e lifecycle de todas as regiões SHM do sistema.
pub struct ShmRegistry {
    /// Mapa de ID → Região
    regions: BTreeMap<ShmId, SharedMemory>,
    /// Próximo ID a ser atribuído
    next_id: u64,
}

impl ShmRegistry {
    /// Cria registry vazio
    pub const fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Cria nova região de memória compartilhada
    ///
    /// # Argumentos
    ///
    /// * `size` - Tamanho em bytes
    ///
    /// # Retorna
    ///
    /// * `Ok(ShmId)` - ID da região criada
    /// * `Err(ShmError)` - Falha na criação
    pub fn create(&mut self, size: usize) -> Result<ShmId, ShmError> {
        let id = ShmId::new(self.next_id);
        self.next_id += 1;

        let shm = SharedMemory::create(id, size)?;
        self.regions.insert(id, shm);

        crate::ktrace!("(SHM) Created region id=", id.as_u64());
        crate::ktrace!("(SHM)   size=", size as u64);

        Ok(id)
    }

    /// Obtém referência a região por ID
    pub fn get(&self, id: ShmId) -> Option<&SharedMemory> {
        self.regions.get(&id)
    }

    /// Obtém referência mutável a região por ID
    pub fn get_mut(&mut self, id: ShmId) -> Option<&mut SharedMemory> {
        self.regions.get_mut(&id)
    }

    /// Incrementa reference count de uma região
    ///
    /// Usado quando um novo processo mapeia a região.
    pub fn add_ref(&mut self, id: ShmId) -> bool {
        if let Some(shm) = self.regions.get_mut(&id) {
            shm.add_ref();
            true
        } else {
            false
        }
    }

    /// Libera referência a uma região
    ///
    /// Se o reference count chegar a zero, a região é destruída
    /// e os frames físicos são liberados.
    pub fn release(&mut self, id: ShmId) {
        let should_free = if let Some(shm) = self.regions.get_mut(&id) {
            shm.release()
        } else {
            false
        };

        if should_free {
            if let Some(mut shm) = self.regions.remove(&id) {
                crate::ktrace!("(SHM) Freeing region id=", id.as_u64());
                shm.free_frames();
            }
        }
    }

    /// Retorna número de regiões ativas
    pub fn count(&self) -> usize {
        self.regions.len()
    }
}

/// Registry global protegido por spinlock
pub static SHM_REGISTRY: Spinlock<ShmRegistry> = Spinlock::new(ShmRegistry::new());

// =============================================================================
// Funções Auxiliares de Mapeamento
// =============================================================================

/// Mapeia uma página física no address space alvo
///
/// Usa page table walk para inserir a entrada diretamente.
fn map_page_in_target(
    target_cr3: u64,
    vaddr: u64,
    phys_frame: u64,
    flags: u64,
) -> Result<(), ShmError> {
    let table_flags: u64 = 0x7; // Present | Writable | User

    unsafe {
        let pml4_phys = target_cr3 & !0xFFF;
        let pml4 = hhdm::phys_to_virt(pml4_phys) as *mut u64;

        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pd_idx = ((vaddr >> 21) & 0x1FF) as usize;
        let pt_idx = ((vaddr >> 12) & 0x1FF) as usize;

        // Ensure PDPT exists
        if (*pml4.add(pml4_idx)) & 1 == 0 {
            let new_pdpt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ShmError::OutOfMemory)?;
            *pml4.add(pml4_idx) = new_pdpt.as_u64() | table_flags;
        }

        let pdpt_phys = (*pml4.add(pml4_idx)) & 0x000F_FFFF_FFFF_F000;
        let pdpt = hhdm::phys_to_virt(pdpt_phys) as *mut u64;

        // Check for 1GB huge page and clear if present
        if (*pdpt.add(pdpt_idx)) & 0x80 != 0 {
            crate::kwarn!("(SHM) Clearing 1GB huge page at PDPT");
            *pdpt.add(pdpt_idx) = 0;
        }

        // Ensure PD exists
        if (*pdpt.add(pdpt_idx)) & 1 == 0 {
            let new_pd = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ShmError::OutOfMemory)?;
            *pdpt.add(pdpt_idx) = new_pd.as_u64() | table_flags;
        }

        let pd_phys = (*pdpt.add(pdpt_idx)) & 0x000F_FFFF_FFFF_F000;
        let pd = hhdm::phys_to_virt(pd_phys) as *mut u64;

        // Check for 2MB huge page and clear if present
        if (*pd.add(pd_idx)) & 0x80 != 0 {
            crate::kwarn!("(SHM) Clearing 2MB huge page at PD");
            *pd.add(pd_idx) = 0;
        }

        // Ensure PT exists
        if (*pd.add(pd_idx)) & 1 == 0 {
            let new_pt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ShmError::OutOfMemory)?;
            *pd.add(pd_idx) = new_pt.as_u64() | table_flags;
        }

        let pt_phys = (*pd.add(pd_idx)) & 0x000F_FFFF_FFFF_F000;
        let pt = hhdm::phys_to_virt(pt_phys) as *mut u64;

        // Map the page
        *pt.add(pt_idx) = phys_frame | flags;
    }

    Ok(())
}
