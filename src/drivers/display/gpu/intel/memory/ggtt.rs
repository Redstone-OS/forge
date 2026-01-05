//! # Global GTT (GGTT)
//!
//! A Global GTT é o mapeamento global de memória do GPU.
//! Qualquer endereço GPU passa pela GGTT para tradução.
//!
//! ## Localização
//!
//! A GGTT está localizada no final do espaço MMIO (BAR0).
//! Offset típico: 0x800000 (8MB do início do BAR0).

#![allow(dead_code)]

use super::gtt::{GttEntry, GttRange};
use crate::rmm::addr::{PhysAddr, VirtAddr};

// =============================================================================
// GGTT
// =============================================================================

/// Global Graphics Translation Table.
pub struct Ggtt {
    /// Ponteiro para as entradas da GTT.
    entries: *mut GttEntry,
    /// Número total de entradas.
    num_entries: usize,
    /// Próxima entrada livre (alocador simples).
    next_free: usize,
}

// Safety: Apenas um thread acessa a GTT de cada vez (protegido por lock externo)
unsafe impl Send for Ggtt {}
unsafe impl Sync for Ggtt {}

impl Ggtt {
    /// Offset da GGTT do início do MMIO.
    const GTT_OFFSET: usize = 0x800000;

    /// Cria nova GGTT a partir do endereço MMIO.
    ///
    /// # Safety
    /// O caller deve garantir que `mmio_base` é válido e tem tamanho suficiente.
    pub unsafe fn new(mmio_base: VirtAddr, mmio_size: usize) -> Option<Self> {
        // GTT está após o espaço de registros
        if mmio_size <= Self::GTT_OFFSET {
            return None;
        }

        let gtt_size = mmio_size - Self::GTT_OFFSET;
        let num_entries = gtt_size / core::mem::size_of::<GttEntry>();

        let entries = (mmio_base.as_u64() + Self::GTT_OFFSET as u64) as *mut GttEntry;

        Some(Self {
            entries,
            num_entries,
            next_free: 0,
        })
    }

    /// Mapeia um range de páginas físicas na GTT.
    pub fn map_pages(&mut self, phys_base: PhysAddr, num_pages: usize) -> Option<GttRange> {
        if self.next_free + num_pages > self.num_entries {
            return None;
        }

        let start = self.next_free as u32;

        for i in 0..num_pages {
            let phys = PhysAddr::new(phys_base.as_u64() + (i as u64 * 4096));
            let entry = GttEntry::new(phys);

            unsafe {
                core::ptr::write_volatile(self.entries.add(self.next_free + i), entry);
            }
        }

        self.next_free += num_pages;

        Some(GttRange::new(start, num_pages as u32))
    }

    /// Desmapeia um range da GTT.
    pub fn unmap(&mut self, range: GttRange) {
        for i in 0..(range.count as usize) {
            unsafe {
                core::ptr::write_volatile(
                    self.entries.add(range.start as usize + i),
                    GttEntry::invalid(),
                );
            }
        }
        // TODO: Implementar free list para reutilização
    }

    /// Retorna endereço GPU correspondente a um range.
    pub fn gpu_address(range: &GttRange) -> u64 {
        (range.start as u64) * 4096
    }

    /// Número total de entradas.
    pub fn total_entries(&self) -> usize {
        self.num_entries
    }

    /// Entradas livres (aproximado).
    pub fn free_entries(&self) -> usize {
        self.num_entries.saturating_sub(self.next_free)
    }
}
