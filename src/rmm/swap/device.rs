//! # Swap Device
//!
//! Abstração de dispositivos de swap (partição, arquivo, ramdisk).
//!
//! ## Tipos de Device
//!
//! - **Partition**: Partição dedicada de swap
//! - **File**: Arquivo de swap (swapfile)
//! - **Ramdisk**: Swap em RAM (para testes)
//!
//! ## Interface
//!
//! Cada device implementa `SwapDeviceOps` para leitura/escrita.

use super::slot::{SlotAllocator, SwapSlot};
use super::SwapError;
use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::virt::hhdm;

// =============================================================================
// SwapDeviceOps Trait
// =============================================================================

/// Operações de dispositivo de swap
pub trait SwapDeviceOps: Send + Sync {
    /// Nome do dispositivo
    fn name(&self) -> &str;

    /// Tamanho total em bytes
    fn size(&self) -> u64;

    /// Lê página do dispositivo
    ///
    /// # Safety
    ///
    /// O endereço físico deve ser válido e ter PAGE_SIZE bytes.
    fn read(&self, offset: u64, phys: PhysAddr) -> Result<(), SwapError>;

    /// Escreve página no dispositivo
    ///
    /// # Safety
    ///
    /// O endereço físico deve ser válido e ter PAGE_SIZE bytes.
    fn write(&self, offset: u64, phys: PhysAddr) -> Result<(), SwapError>;

    /// Sincroniza escritas pendentes
    fn sync(&self) -> Result<(), SwapError>;
}

// =============================================================================
// SwapDevice
// =============================================================================

/// Dispositivo de swap com alocador de slots
pub struct SwapDevice {
    /// Nome do dispositivo
    name: &'static str,
    /// Alocador de slots
    allocator: SlotAllocator,
    /// Backend de I/O
    backend: SwapBackend,
    /// Prioridade (maior = preferido)
    priority: i32,
}

impl SwapDevice {
    /// Cria novo device a partir de backend
    pub fn new(name: &'static str, backend: SwapBackend, priority: i32) -> Self {
        let size = backend.size();
        let total_slots = size / PAGE_SIZE as u64;

        Self {
            name,
            allocator: SlotAllocator::new(total_slots),
            backend,
            priority,
        }
    }

    /// Cria device ramdisk (para testes)
    pub fn new_ramdisk(size_mb: usize) -> Self {
        let backend = SwapBackend::Ramdisk(RamdiskBackend::new(size_mb));
        Self::new("ramdisk", backend, -10) // Baixa prioridade
    }

    /// Nome do dispositivo
    pub fn name(&self) -> &str {
        self.name
    }

    /// Total de slots
    pub fn total_slots(&self) -> u64 {
        self.allocator.total()
    }

    /// Slots em uso
    pub fn used_slots(&self) -> u64 {
        self.allocator.used()
    }

    /// Slots livres
    pub fn free_slots(&self) -> u64 {
        self.allocator.free_count()
    }

    /// Aloca um slot
    pub fn alloc_slot(&self) -> Option<SwapSlot> {
        self.allocator.alloc()
    }

    /// Libera um slot
    pub fn free_slot(&self, slot: SwapSlot) {
        self.allocator.free(slot)
    }

    /// Lê página de um slot
    pub fn read_page(&self, slot: SwapSlot, phys: PhysAddr) -> Result<(), SwapError> {
        if !self.allocator.is_allocated(slot) {
            return Err(SwapError::InvalidSlot);
        }

        let offset = slot.0 * PAGE_SIZE as u64;
        self.backend.read(offset, phys)
    }

    /// Escreve página em um slot
    pub fn write_page(&self, slot: SwapSlot, phys: PhysAddr) -> Result<(), SwapError> {
        let offset = slot.0 * PAGE_SIZE as u64;
        self.backend.write(offset, phys)
    }

    /// Prioridade do device
    pub fn priority(&self) -> i32 {
        self.priority
    }
}

// =============================================================================
// SwapBackend
// =============================================================================

/// Backend de armazenamento para swap
pub enum SwapBackend {
    /// Ramdisk (para testes)
    Ramdisk(RamdiskBackend),
    /// Block device (futuro)
    Block(BlockBackend),
}

impl SwapBackend {
    fn size(&self) -> u64 {
        match self {
            Self::Ramdisk(r) => r.size(),
            Self::Block(b) => b.size(),
        }
    }

    fn read(&self, offset: u64, phys: PhysAddr) -> Result<(), SwapError> {
        match self {
            Self::Ramdisk(r) => r.read(offset, phys),
            Self::Block(b) => b.read(offset, phys),
        }
    }

    fn write(&self, offset: u64, phys: PhysAddr) -> Result<(), SwapError> {
        match self {
            Self::Ramdisk(r) => r.write(offset, phys),
            Self::Block(b) => b.write(offset, phys),
        }
    }
}

// =============================================================================
// RamdiskBackend
// =============================================================================

/// Backend de ramdisk (swap em memória, para testes)
pub struct RamdiskBackend {
    /// Endereço base
    base: PhysAddr,
    /// Tamanho em bytes
    size: u64,
}

impl RamdiskBackend {
    /// Cria novo ramdisk
    pub fn new(size_mb: usize) -> Self {
        // Aloca memória para o ramdisk
        let pages = (size_mb * 1024 * 1024) / PAGE_SIZE;

        // TODO: Alocar memória contígua via phys::alloc_contiguous
        // Por enquanto, usa placeholder
        let base = PhysAddr::new(0); // Placeholder

        Self {
            base,
            size: (size_mb * 1024 * 1024) as u64,
        }
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn read(&self, offset: u64, dest_phys: PhysAddr) -> Result<(), SwapError> {
        if offset + PAGE_SIZE as u64 > self.size {
            return Err(SwapError::InvalidSlot);
        }

        if self.base.as_u64() == 0 {
            // Ramdisk não inicializado, simula leitura
            return Ok(());
        }

        // Copia de ramdisk para página destino
        let src = hhdm::phys_to_virt(self.base.as_u64() + offset);
        let dst = hhdm::phys_to_virt(dest_phys.as_u64());

        unsafe {
            core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, PAGE_SIZE);
        }

        Ok(())
    }

    fn write(&self, offset: u64, src_phys: PhysAddr) -> Result<(), SwapError> {
        if offset + PAGE_SIZE as u64 > self.size {
            return Err(SwapError::InvalidSlot);
        }

        if self.base.as_u64() == 0 {
            // Ramdisk não inicializado, simula escrita
            return Ok(());
        }

        // Copia de página origem para ramdisk
        let src = hhdm::phys_to_virt(src_phys.as_u64());
        let dst = hhdm::phys_to_virt(self.base.as_u64() + offset);

        unsafe {
            core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, PAGE_SIZE);
        }

        Ok(())
    }
}

// =============================================================================
// BlockBackend
// =============================================================================

/// Backend de block device (partição/arquivo)
pub struct BlockBackend {
    /// Nome do device
    name: &'static str,
    /// Tamanho em bytes
    size: u64,
    // TODO: Handle para block device quando driver estiver pronto
}

impl BlockBackend {
    /// Cria novo backend de block device
    pub fn new(name: &'static str, size: u64) -> Self {
        Self { name, size }
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn read(&self, _offset: u64, _phys: PhysAddr) -> Result<(), SwapError> {
        // TODO: Implementar quando block device driver estiver pronto
        Err(SwapError::IoError)
    }

    fn write(&self, _offset: u64, _phys: PhysAddr) -> Result<(), SwapError> {
        // TODO: Implementar quando block device driver estiver pronto
        Err(SwapError::IoError)
    }
}
