//! # VMO - Virtual Memory Object
//!
//! Abstração de memória virtual com capabilities.
//!
//! ## 🎯 Propósito
//!
//! VMOs são a abstração principal de memória em sistemas capability-based:
//! - Representam uma região contígua de memória virtual
//! - Podem ser compartilhados entre processos
//! - Suportam COW (Copy-On-Write)
//! - Podem ser mapeados em diferentes address spaces
//!
//! ## 🏗️ Arquitetura
//!
//! ```text
//! VMO
//!  ├── Páginas físicas (lazy allocated)
//!  ├── Flags (read, write, exec)
//!  └── Mappings (onde está mapeado)
//! ```
//!
//! ## 🔧 Uso
//!
//! ```rust
//! // Criar VMO
//! let vmo = VMO::create(4096 * 4, VMOFlags::READ | VMOFlags::WRITE)?;
//!
//! // Mapear em address space
//! let mapping = vmo.map(address_space, 0x1000_0000)?;
//!
//! // Compartilhar via handle
//! let handle = vmo.create_handle(VMOFlags::READ)?;
//! ```

use crate::mm::addr::PhysAddr;
use crate::mm::config::PAGE_SIZE;
use crate::mm::error::{MmError, MmResult};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// =============================================================================
// FLAGS
// =============================================================================

/// Flags de VMO
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VMOFlags(u32);

impl VMOFlags {
    /// VMO pode ser lido
    pub const READ: Self = Self(1 << 0);
    /// VMO pode ser escrito
    pub const WRITE: Self = Self(1 << 1);
    /// VMO pode ser executado
    pub const EXEC: Self = Self(1 << 2);
    /// VMO pode ser compartilhado
    pub const SHARE: Self = Self(1 << 3);
    /// VMO pode ser duplicado
    pub const DUPLICATE: Self = Self(1 << 4);
    /// VMO pode ser transferido
    pub const TRANSFER: Self = Self(1 << 5);
    /// Páginas zeradas no acesso
    pub const ZERO_ON_DEMAND: Self = Self(1 << 6);
    /// Committed (páginas alocadas imediatamente)
    pub const COMMITTED: Self = Self(1 << 7);
    /// Páginas são pinned (não swappable)
    pub const PINNED: Self = Self(1 << 8);
    /// Copy-on-write
    pub const COW: Self = Self(1 << 9);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }
}

// =============================================================================
// VMO
// =============================================================================

/// Estado de uma página no VMO
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageState {
    /// Não alocada ainda
    NotPresent,
    /// Zerada sob demanda
    ZeroFill,
    /// Alocada e presente
    Present(PhysAddr),
    /// Copy-on-write (compartilha com outro VMO)
    CopyOnWrite(PhysAddr),
}

/// Virtual Memory Object
pub struct VMO {
    /// ID único
    id: u64,
    /// Tamanho em bytes (múltiplo de PAGE_SIZE)
    size: usize,
    /// Flags
    flags: VMOFlags,
    /// Estado de cada página
    pages: Vec<PageState>,
    /// Número de mappings ativos
    mapping_count: AtomicUsize,
    /// Contagem de referências
    ref_count: AtomicUsize,
}

/// ID generator global para VMOs
static NEXT_VMO_ID: AtomicU64 = AtomicU64::new(1);

impl VMO {
    /// Cria novo VMO
    pub fn create(size: usize, flags: VMOFlags) -> MmResult<Self> {
        if size == 0 {
            return Err(MmError::InvalidSize);
        }

        // Alinhar tamanho
        let aligned_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let num_pages = aligned_size / PAGE_SIZE;

        // Estado inicial das páginas
        let initial_state = if flags.contains(VMOFlags::ZERO_ON_DEMAND) {
            PageState::ZeroFill
        } else {
            PageState::NotPresent
        };

        let mut pages = Vec::with_capacity(num_pages);
        for _ in 0..num_pages {
            pages.push(initial_state);
        }

        // Se COMMITTED, alocar páginas agora
        let mut vmo = Self {
            id: NEXT_VMO_ID.fetch_add(1, Ordering::Relaxed),
            size: aligned_size,
            flags,
            pages,
            mapping_count: AtomicUsize::new(0),
            ref_count: AtomicUsize::new(1),
        };

        if flags.contains(VMOFlags::COMMITTED) {
            vmo.commit_all()?;
        }

        crate::kdebug!("(VMO) Criado VMO ID=", vmo.id);
        crate::kdebug!("(VMO) VMO páginas   =", num_pages as u64);

        Ok(vmo)
    }

    /// ID do VMO
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Tamanho em bytes
    pub fn size(&self) -> usize {
        self.size
    }

    /// Número de páginas
    pub fn num_pages(&self) -> usize {
        self.pages.len()
    }

    /// Flags
    pub fn flags(&self) -> VMOFlags {
        self.flags
    }

    /// Aloca todas as páginas
    fn commit_all(&mut self) -> MmResult<()> {
        let mut pmm = crate::mm::pmm::FRAME_ALLOCATOR.lock();

        for i in 0..self.pages.len() {
            if matches!(self.pages[i], PageState::NotPresent | PageState::ZeroFill) {
                let frame = pmm.allocate_frame().ok_or(MmError::OutOfMemory)?;

                // Zerar se necessário
                if matches!(self.pages[i], PageState::ZeroFill) {
                    unsafe {
                        let virt = crate::mm::addr::phys_to_virt(PhysAddr::new(frame.addr()));
                        crate::mm::ops::memops::memzero(virt.as_mut_ptr(), PAGE_SIZE);
                    }
                }

                self.pages[i] = PageState::Present(PhysAddr::new(frame.addr()));
            }
        }

        Ok(())
    }

    /// Obtém página por índice
    pub fn get_page(&self, index: usize) -> Option<PageState> {
        self.pages.get(index).copied()
    }

    /// Resolve page fault (aloca página sob demanda)
    pub fn fault(&mut self, page_index: usize) -> MmResult<PhysAddr> {
        if page_index >= self.pages.len() {
            return Err(MmError::OutOfBounds);
        }

        match self.pages[page_index] {
            PageState::Present(addr) => Ok(addr),

            PageState::NotPresent | PageState::ZeroFill => {
                let mut pmm = crate::mm::pmm::FRAME_ALLOCATOR.lock();
                let frame = pmm.allocate_frame().ok_or(MmError::OutOfMemory)?;
                let addr = PhysAddr::new(frame.addr());

                // Zerar
                unsafe {
                    let virt = crate::mm::addr::phys_to_virt(addr);
                    crate::mm::ops::memops::memzero(virt.as_mut_ptr(), PAGE_SIZE);
                }

                self.pages[page_index] = PageState::Present(addr);
                Ok(addr)
            }

            PageState::CopyOnWrite(original) => {
                // Alocar nova página e copiar
                let mut pmm = crate::mm::pmm::FRAME_ALLOCATOR.lock();
                let frame = pmm.allocate_frame().ok_or(MmError::OutOfMemory)?;
                let new_addr = PhysAddr::new(frame.addr());

                unsafe {
                    let src = crate::mm::addr::phys_to_virt(original);
                    let dst = crate::mm::addr::phys_to_virt(new_addr);
                    crate::mm::ops::memops::memcpy(dst.as_mut_ptr(), src.as_ptr(), PAGE_SIZE);
                }

                self.pages[page_index] = PageState::Present(new_addr);
                Ok(new_addr)
            }
        }
    }

    /// Cria handle para este VMO
    pub fn create_handle(&self, rights: VMOFlags) -> VMOHandle {
        self.add_ref();
        VMOHandle {
            vmo_id: self.id,
            rights,
        }
    }

    /// Incrementa referência
    pub fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrementa referência
    pub fn release(&self) -> usize {
        self.ref_count.fetch_sub(1, Ordering::Relaxed) - 1
    }

    /// Contagem de referências
    pub fn ref_count(&self) -> usize {
        self.ref_count.load(Ordering::Relaxed)
    }
}

impl Drop for VMO {
    fn drop(&mut self) {
        // Liberar páginas físicas
        let mut pmm = crate::mm::pmm::FRAME_ALLOCATOR.lock();

        for page in &self.pages {
            if let PageState::Present(addr) = page {
                let frame = crate::mm::pmm::PhysFrame::from_start_address(*addr);
                pmm.deallocate_frame(frame);
            }
        }

        crate::kdebug!("(VMO) Destruído VMO ID=", self.id);
    }
}

// =============================================================================
// VMO HANDLE
// =============================================================================

/// Handle para um VMO
///
/// Representa permissão para acessar um VMO com direitos específicos.
/// Handles podem ser passados entre processos.
#[derive(Clone, Debug)]
pub struct VMOHandle {
    /// ID do VMO referenciado
    pub vmo_id: u64,
    /// Direitos concedidos por este handle
    pub rights: VMOFlags,
}

impl VMOHandle {
    /// Verifica se tem direito de leitura
    pub fn can_read(&self) -> bool {
        self.rights.contains(VMOFlags::READ)
    }

    /// Verifica se tem direito de escrita
    pub fn can_write(&self) -> bool {
        self.rights.contains(VMOFlags::WRITE)
    }

    /// Verifica se tem direito de execução
    pub fn can_exec(&self) -> bool {
        self.rights.contains(VMOFlags::EXEC)
    }

    /// Cria handle derivado com direitos restritos
    pub fn derive(&self, new_rights: VMOFlags) -> Option<Self> {
        // Não pode adicionar direitos
        if new_rights.bits() & !self.rights.bits() != 0 {
            return None;
        }

        Some(Self {
            vmo_id: self.vmo_id,
            rights: new_rights,
        })
    }
}
