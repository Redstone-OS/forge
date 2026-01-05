//! # DMA Buffer Allocation
//!
//! Alocação de buffers para DMA (Direct Memory Access).
//!
//! ## Tipos de DMA
//!
//! ### Coherent DMA
//! - CPU e dispositivo veem mesma memória
//! - Sem necessidade de flush/invalidate
//! - Mais lento (cache disabled)
//! - Uso: control structures, descriptors
//!
//! ### Streaming DMA
//! - Requer sync explícita antes/depois do DMA
//! - Usa cache (mais rápido)
//! - Direção: To Device, From Device, Bidirectional
//! - Uso: data buffers
//!
//! ## Restrições de Zona
//!
//! - **Legacy ISA**: Deve usar DMA Zone (0-16MB)
//! - **PCI 32-bit**: Deve usar DMA32 Zone (0-4GB)
//! - **PCI 64-bit**: Pode usar Normal Zone
//! - **Com IOMMU**: Qualquer memória física pode ser remapeada
//!
//! ## Contiguidade
//!
//! DMA geralmente requer memória fisicamente contígua para evitar
//! scatter-gather. Por isso, alocações DMA são feitas de chunks
//! Unmovable para evitar fragmentação.

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::error::{RmmError, RmmResult};
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::{hhdm, mapper, MapFlags};
use crate::rmm::zone::Zone;
use crate::sync::Spinlock;

use alloc::vec::Vec;
use core::ptr::NonNull;

/// DMA Pool para alocações rápidas
static DMA_POOL: Spinlock<Option<DmaPool>> = Spinlock::new(None);

/// Inicializa o DMA pool
pub fn init_pool() {
    let pool = DmaPool::new(64); // 64 buffers pré-alocados
    *DMA_POOL.lock() = Some(pool);
    crate::kinfo!("(DMA) Pool inicializado com 64 buffers");
}

// =============================================================================
// DmaBuffer
// =============================================================================

/// Buffer DMA alocado
#[derive(Debug)]
pub struct DmaBuffer {
    /// Endereço físico (para o dispositivo)
    phys: PhysAddr,
    /// Endereço virtual (para CPU)
    virt: *mut u8,
    /// Tamanho em bytes
    size: usize,
    /// É coherent?
    coherent: bool,
    /// Driver ID (para tracking)
    driver_id: u32,
}

impl DmaBuffer {
    /// Cria novo buffer DMA
    fn new(phys: PhysAddr, size: usize, coherent: bool, driver_id: u32) -> Self {
        let virt = hhdm::phys_to_virt(phys.as_u64()) as *mut u8;
        Self {
            phys,
            virt,
            size,
            coherent,
            driver_id,
        }
    }

    /// Endereço físico (para programar no dispositivo)
    #[inline]
    pub fn phys_addr(&self) -> PhysAddr {
        self.phys
    }

    /// Endereço físico como u64 (convenience)
    #[inline]
    pub fn dma_addr(&self) -> u64 {
        self.phys.as_u64()
    }

    /// Endereço virtual (para acesso CPU)
    #[inline]
    pub fn cpu_addr(&self) -> *mut u8 {
        self.virt
    }

    /// Tamanho do buffer
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// É coherent?
    #[inline]
    pub fn is_coherent(&self) -> bool {
        self.coherent
    }

    /// Slice para leitura
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.virt, self.size) }
    }

    /// Slice para escrita
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.virt, self.size) }
    }

    /// Sync para dispositivo (before DMA write)
    ///
    /// Para coherent, é no-op. Para streaming, faz cache flush.
    pub fn sync_for_device(&self) {
        if !self.coherent {
            // TODO: Implementar cache flush
            // x86: CLFLUSH ou WBINVD
            unsafe {
                core::arch::asm!("mfence", options(nostack, preserves_flags));
            }
        }
    }

    /// Sync para CPU (after DMA read)
    ///
    /// Para coherent, é no-op. Para streaming, faz cache invalidate.
    pub fn sync_for_cpu(&self) {
        if !self.coherent {
            // TODO: Implementar cache invalidate
            unsafe {
                core::arch::asm!("mfence", options(nostack, preserves_flags));
            }
        }
    }

    /// Zera o buffer
    pub fn zero(&mut self) {
        unsafe {
            core::ptr::write_bytes(self.virt, 0, self.size);
        }
        self.sync_for_device();
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        // Libera memória física
        let _ = free_dma_internal(self.phys, self.size, self.driver_id);
    }
}

// Safety: DmaBuffer pode ser enviado entre threads
unsafe impl Send for DmaBuffer {}
unsafe impl Sync for DmaBuffer {}

// =============================================================================
// API Pública
// =============================================================================

/// Aloca buffer DMA coherent
///
/// # Arguments
///
/// * `size` - Tamanho em bytes
/// * `driver_id` - ID do driver (para tracking)
///
/// # Returns
///
/// Buffer DMA coherent alocado
pub fn alloc_dma_coherent(size: usize, driver_id: u32) -> RmmResult<DmaBuffer> {
    alloc_dma_internal(size, driver_id, true, false)
}

/// Aloca buffer DMA streaming
///
/// # Arguments
///
/// * `size` - Tamanho em bytes
/// * `driver_id` - ID do driver
/// * `dma32` - Se true, aloca de DMA32 zone (< 4GB)
///
/// # Returns
///
/// Buffer DMA streaming alocado
pub fn alloc_dma(size: usize, driver_id: u32, dma32: bool) -> RmmResult<DmaBuffer> {
    alloc_dma_internal(size, driver_id, false, dma32)
}

/// Libera buffer DMA
pub fn free_dma(buffer: DmaBuffer) {
    // Drop cuida da liberação
    drop(buffer);
}

/// Implementação interna de alocação DMA
fn alloc_dma_internal(
    size: usize,
    driver_id: u32,
    coherent: bool,
    dma32: bool,
) -> RmmResult<DmaBuffer> {
    // Arredonda para páginas
    let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
    let alloc_size = pages * PAGE_SIZE;

    // Escolhe zona
    let zone = if dma32 { Zone::Dma32 } else { Zone::Normal };

    // Flags de alocação
    // IMPORTANTE: DMA usa Unmovable para não fragmentar
    let mut flags = AllocFlags::ZERO.union(AllocFlags::PINNED);
    if pages > 1 {
        flags = flags.union(AllocFlags::CONTIGUOUS);
    }

    // Aloca frames
    let phys = if pages == 1 {
        phys::alloc(FrameOwner::Driver { id: driver_id }, zone, flags)
    } else {
        phys::alloc_contiguous(pages, FrameOwner::Driver { id: driver_id }, zone, flags)
    }
    .ok_or(RmmError::OutOfMemory)?;

    // Se coherent, precisa mapear com cache desabilitado
    if coherent {
        // TODO: Remapear com NO_CACHE
    }

    Ok(DmaBuffer::new(phys, alloc_size, coherent, driver_id))
}

/// Libera DMA internamente
fn free_dma_internal(phys: PhysAddr, size: usize, driver_id: u32) -> RmmResult<()> {
    let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;

    for i in 0..pages {
        let frame_phys = phys + (i * PAGE_SIZE) as u64;
        phys::free(frame_phys, FrameOwner::Driver { id: driver_id })?;
    }

    Ok(())
}

// =============================================================================
// DmaPool
// =============================================================================

/// Pool de buffers DMA pré-alocados
///
/// Evita latência de alocação em hot paths.
pub struct DmaPool {
    /// Buffers disponíveis (por tamanho)
    small: Vec<DmaBuffer>, // 4KB
    medium: Vec<DmaBuffer>, // 16KB
    large: Vec<DmaBuffer>,  // 64KB
    /// Capacidade máxima
    max_capacity: usize,
}

impl DmaPool {
    /// Cria pool vazio
    pub fn new(capacity: usize) -> Self {
        Self {
            small: Vec::with_capacity(capacity),
            medium: Vec::with_capacity(capacity / 2),
            large: Vec::with_capacity(capacity / 4),
            max_capacity: capacity,
        }
    }

    /// Pré-aloca buffers
    pub fn preallocate(&mut self, driver_id: u32) {
        // Pré-aloca alguns buffers pequenos
        for _ in 0..self.max_capacity.min(16) {
            if let Ok(buf) = alloc_dma_internal(PAGE_SIZE, driver_id, false, true) {
                self.small.push(buf);
            }
        }
    }

    /// Obtém buffer do pool ou aloca novo
    pub fn get(&mut self, size: usize, driver_id: u32) -> RmmResult<DmaBuffer> {
        // Escolhe categoria
        let buffer = if size <= PAGE_SIZE {
            self.small.pop()
        } else if size <= PAGE_SIZE * 4 {
            self.medium.pop()
        } else if size <= PAGE_SIZE * 16 {
            self.large.pop()
        } else {
            None
        };

        match buffer {
            Some(buf) => Ok(buf),
            None => alloc_dma_internal(size, driver_id, false, true),
        }
    }

    /// Devolve buffer ao pool
    pub fn put(&mut self, buffer: DmaBuffer) {
        let size = buffer.size();

        // Escolhe categoria e adiciona se há espaço
        if size <= PAGE_SIZE && self.small.len() < self.max_capacity {
            // Não faz drop, guarda no pool
            self.small.push(buffer);
        } else if size <= PAGE_SIZE * 4 && self.medium.len() < self.max_capacity / 2 {
            self.medium.push(buffer);
        } else if size <= PAGE_SIZE * 16 && self.large.len() < self.max_capacity / 4 {
            self.large.push(buffer);
        }
        // else: drop acontece automaticamente
    }

    /// Limpa pool (libera todos os buffers)
    pub fn clear(&mut self) {
        self.small.clear();
        self.medium.clear();
        self.large.clear();
    }

    /// Estatísticas
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.small.len(), self.medium.len(), self.large.len())
    }
}

// =============================================================================
// Scatter-Gather
// =============================================================================

/// Entrada de scatter-gather list
#[derive(Debug, Clone, Copy)]
pub struct ScatterEntry {
    pub phys: PhysAddr,
    pub size: usize,
}

/// Lista scatter-gather
pub struct ScatterGatherList {
    entries: Vec<ScatterEntry>,
    total_size: usize,
}

impl ScatterGatherList {
    /// Cria lista vazia
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_size: 0,
        }
    }

    /// Adiciona entrada
    pub fn push(&mut self, phys: PhysAddr, size: usize) {
        self.entries.push(ScatterEntry { phys, size });
        self.total_size += size;
    }

    /// Número de entradas
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Está vazia?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Tamanho total
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Itera sobre entradas
    pub fn iter(&self) -> impl Iterator<Item = &ScatterEntry> {
        self.entries.iter()
    }

    /// Converte para slice de (addr, len) para hardware
    pub fn to_hardware_format(&self) -> Vec<(u64, u32)> {
        self.entries
            .iter()
            .map(|e| (e.phys.as_u64(), e.size as u32))
            .collect()
    }
}

impl Default for ScatterGatherList {
    fn default() -> Self {
        Self::new()
    }
}
