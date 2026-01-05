//! # Device Buffer
//!
//! Buffer de I/O para comunicação com dispositivos.
//!
//! ## Uso
//!
//! DeviceBuffer encapsula memória que pode ser compartilhada
//! entre CPU e dispositivo, com tracking de direção de DMA.
//!
//! ## Direções
//!
//! - **ToDevice**: CPU escreve, dispositivo lê (ex: command buffer)
//! - **FromDevice**: Dispositivo escreve, CPU lê (ex: receive buffer)
//! - **Bidirectional**: Ambos podem ler/escrever (ex: shared memory)
//!
//! ## Exemplo
//!
//! ```rust
//! // Buffer para enviar dados ao dispositivo
//! let mut buf = DeviceBuffer::new(4096, true)?;
//! buf.write(0, &command_data);
//! buf.sync_for_device();
//!
//! // Programa dispositivo com endereço físico
//! device.set_buffer(buf.dma_addr());
//! device.start_transfer();
//!
//! // Aguarda conclusão
//! device.wait_complete();
//!
//! // Lê resposta
//! buf.sync_for_cpu();
//! buf.read(0, &mut response_data);
//! ```

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::error::{RmmError, RmmResult};
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;

// =============================================================================
// DmaDirection
// =============================================================================

/// Direção do DMA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// CPU → Device
    ToDevice,
    /// Device → CPU
    FromDevice,
    /// Bidirecional
    Bidirectional,
    /// Nenhum (buffer não usado para DMA)
    None,
}

impl DmaDirection {
    /// Precisa de sync antes de DMA?
    pub fn needs_sync_before(&self) -> bool {
        matches!(self, Self::ToDevice | Self::Bidirectional)
    }

    /// Precisa de sync depois de DMA?
    pub fn needs_sync_after(&self) -> bool {
        matches!(self, Self::FromDevice | Self::Bidirectional)
    }
}

// =============================================================================
// DeviceBuffer
// =============================================================================

/// Buffer para comunicação com dispositivo
pub struct DeviceBuffer {
    /// Endereço físico
    phys: PhysAddr,
    /// Endereço virtual
    virt: *mut u8,
    /// Tamanho em bytes
    size: usize,
    /// Direção de DMA
    direction: DmaDirection,
    /// Está mapeado para DMA?
    mapped: bool,
    /// É DMA32?
    dma32: bool,
}

impl DeviceBuffer {
    /// Cria novo buffer
    ///
    /// # Arguments
    ///
    /// * `size` - Tamanho em bytes
    /// * `dma32` - Se true, aloca em < 4GB
    pub fn new(size: usize, dma32: bool) -> RmmResult<Self> {
        let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        let alloc_size = pages * PAGE_SIZE;

        let zone = if dma32 { Zone::Dma32 } else { Zone::Normal };
        let flags = AllocFlags::ZERO.union(AllocFlags::PINNED);

        let phys = if pages == 1 {
            phys::alloc(FrameOwner::Kernel, zone, flags)
        } else {
            phys::alloc_contiguous(pages, FrameOwner::Kernel, zone, flags)
        }
        .ok_or(RmmError::OutOfMemory)?;

        let virt = hhdm::phys_to_virt(phys.as_u64()) as *mut u8;

        Ok(Self {
            phys,
            virt,
            size: alloc_size,
            direction: DmaDirection::None,
            mapped: false,
            dma32,
        })
    }

    /// Cria buffer zerado
    pub fn zeroed(size: usize, dma32: bool) -> RmmResult<Self> {
        let mut buf = Self::new(size, dma32)?;
        buf.zero();
        Ok(buf)
    }

    /// Endereço físico (para DMA)
    #[inline]
    pub fn phys_addr(&self) -> PhysAddr {
        self.phys
    }

    /// Endereço para programar no dispositivo
    #[inline]
    pub fn dma_addr(&self) -> u64 {
        self.phys.as_u64()
    }

    /// Tamanho do buffer
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Direção de DMA
    #[inline]
    pub fn direction(&self) -> DmaDirection {
        self.direction
    }

    /// Define direção de DMA
    pub fn set_direction(&mut self, direction: DmaDirection) {
        self.direction = direction;
    }

    /// Mapeia para DMA (inicia transferência)
    pub fn map_for_dma(&mut self, direction: DmaDirection) {
        self.direction = direction;
        self.mapped = true;

        if direction.needs_sync_before() {
            self.sync_for_device();
        }
    }

    /// Desmapeia de DMA (finaliza transferência)
    pub fn unmap_from_dma(&mut self) {
        if self.direction.needs_sync_after() {
            self.sync_for_cpu();
        }

        self.mapped = false;
        self.direction = DmaDirection::None;
    }

    // -------------------------------------------------------------------------
    // Sync
    // -------------------------------------------------------------------------

    /// Sync para dispositivo (antes de DMA para device)
    pub fn sync_for_device(&self) {
        // Memory barrier para garantir que escritas estão visíveis
        unsafe {
            core::arch::asm!("mfence", options(nostack, preserves_flags));
        }
        // TODO: Cache flush se necessário
    }

    /// Sync para CPU (depois de DMA do device)
    pub fn sync_for_cpu(&self) {
        // Memory barrier
        unsafe {
            core::arch::asm!("mfence", options(nostack, preserves_flags));
        }
        // TODO: Cache invalidate se necessário
    }

    // -------------------------------------------------------------------------
    // Acesso
    // -------------------------------------------------------------------------

    /// Ponteiro para offset
    #[inline]
    pub fn ptr(&self, offset: usize) -> *mut u8 {
        debug_assert!(offset < self.size);
        unsafe { self.virt.add(offset) }
    }

    /// Ponteiro tipado para offset
    #[inline]
    pub fn ptr_at<T>(&self, offset: usize) -> *mut T {
        debug_assert!(offset + core::mem::size_of::<T>() <= self.size);
        unsafe { self.virt.add(offset) as *mut T }
    }

    /// Lê bytes do buffer
    pub fn read(&self, offset: usize, buf: &mut [u8]) {
        debug_assert!(offset + buf.len() <= self.size);
        unsafe {
            core::ptr::copy_nonoverlapping(self.virt.add(offset), buf.as_mut_ptr(), buf.len());
        }
    }

    /// Escreve bytes no buffer
    pub fn write(&mut self, offset: usize, buf: &[u8]) {
        debug_assert!(offset + buf.len() <= self.size);
        unsafe {
            core::ptr::copy_nonoverlapping(buf.as_ptr(), self.virt.add(offset), buf.len());
        }
    }

    /// Lê valor tipado
    pub fn read_val<T: Copy>(&self, offset: usize) -> T {
        debug_assert!(offset + core::mem::size_of::<T>() <= self.size);
        unsafe { core::ptr::read_volatile(self.virt.add(offset) as *const T) }
    }

    /// Escreve valor tipado
    pub fn write_val<T>(&mut self, offset: usize, value: T) {
        debug_assert!(offset + core::mem::size_of::<T>() <= self.size);
        unsafe {
            core::ptr::write_volatile(self.virt.add(offset) as *mut T, value);
        }
    }

    /// Zera o buffer
    pub fn zero(&mut self) {
        unsafe {
            core::ptr::write_bytes(self.virt, 0, self.size);
        }
    }

    /// Preenche com valor
    pub fn fill(&mut self, value: u8) {
        unsafe {
            core::ptr::write_bytes(self.virt, value, self.size);
        }
    }

    /// Slice de leitura
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.virt, self.size) }
    }

    /// Slice de escrita
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.virt, self.size) }
    }

    /// Slice tipado
    pub fn as_slice_of<T>(&self) -> &[T] {
        let count = self.size / core::mem::size_of::<T>();
        unsafe { core::slice::from_raw_parts(self.virt as *const T, count) }
    }

    /// Slice tipado mutável
    pub fn as_slice_of_mut<T>(&mut self) -> &mut [T] {
        let count = self.size / core::mem::size_of::<T>();
        unsafe { core::slice::from_raw_parts_mut(self.virt as *mut T, count) }
    }
}

impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        // Unmap se ainda mapeado
        if self.mapped {
            self.unmap_from_dma();
        }

        // Libera memória
        let pages = self.size / PAGE_SIZE;
        for i in 0..pages {
            let frame_phys = self.phys + (i * PAGE_SIZE) as u64;
            let _ = phys::free(frame_phys, FrameOwner::Kernel);
        }
    }
}

// Safety: DeviceBuffer pode ser enviado entre threads
unsafe impl Send for DeviceBuffer {}
unsafe impl Sync for DeviceBuffer {}

// =============================================================================
// Ring Buffer
// =============================================================================

/// Ring buffer para comunicação com dispositivo
///
/// Comum em drivers de rede e storage para descriptor rings.
pub struct DeviceRing {
    /// Buffer backing
    buffer: DeviceBuffer,
    /// Número de entradas
    entries: usize,
    /// Tamanho de cada entrada
    entry_size: usize,
    /// Índice de produção
    prod_idx: usize,
    /// Índice de consumo
    cons_idx: usize,
}

impl DeviceRing {
    /// Cria novo ring
    pub fn new(entries: usize, entry_size: usize, dma32: bool) -> RmmResult<Self> {
        let size = entries * entry_size;
        let buffer = DeviceBuffer::zeroed(size, dma32)?;

        Ok(Self {
            buffer,
            entries,
            entry_size,
            prod_idx: 0,
            cons_idx: 0,
        })
    }

    /// Endereço DMA
    #[inline]
    pub fn dma_addr(&self) -> u64 {
        self.buffer.dma_addr()
    }

    /// Número de entradas livres
    pub fn free_count(&self) -> usize {
        if self.prod_idx >= self.cons_idx {
            self.entries - (self.prod_idx - self.cons_idx) - 1
        } else {
            self.cons_idx - self.prod_idx - 1
        }
    }

    /// Número de entradas usadas
    pub fn used_count(&self) -> usize {
        self.entries - 1 - self.free_count()
    }

    /// Ring está cheio?
    pub fn is_full(&self) -> bool {
        self.free_count() == 0
    }

    /// Ring está vazio?
    pub fn is_empty(&self) -> bool {
        self.prod_idx == self.cons_idx
    }

    /// Obtém ponteiro para entrada
    pub fn entry(&self, idx: usize) -> *mut u8 {
        let offset = (idx % self.entries) * self.entry_size;
        self.buffer.ptr(offset)
    }

    /// Avança índice de produção
    pub fn advance_prod(&mut self) {
        self.prod_idx = (self.prod_idx + 1) % self.entries;
    }

    /// Avança índice de consumo
    pub fn advance_cons(&mut self) {
        self.cons_idx = (self.cons_idx + 1) % self.entries;
    }

    /// Índice de produção atual
    pub fn prod_index(&self) -> usize {
        self.prod_idx
    }

    /// Índice de consumo atual
    pub fn cons_index(&self) -> usize {
        self.cons_idx
    }

    /// Sync para dispositivo
    pub fn sync_for_device(&self) {
        self.buffer.sync_for_device();
    }

    /// Sync para CPU
    pub fn sync_for_cpu(&self) {
        self.buffer.sync_for_cpu();
    }
}
