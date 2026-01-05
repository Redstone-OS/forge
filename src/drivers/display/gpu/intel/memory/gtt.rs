//! # Graphics Translation Table (GTT)
//!
//! Base abstraction para a Graphics Translation Table.
//! Traduz endereços GPU para endereços físicos do sistema.
//!
//! ## Formato de Entrada GTT (Gen9+)
//!
//! ```text
//! [63:12] Physical Address (bits 51:12)
//! [11:4]  Reserved
//! [3]     Local Memory (Gen12+, 0 for system memory)
//! [2:1]   Reserved
//! [0]     Valid
//! ```

#![allow(dead_code)]

use crate::mm::PhysAddr;

// =============================================================================
// GTT ENTRY
// =============================================================================

/// Entrada da GTT (64 bits).
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct GttEntry(u64);

impl GttEntry {
    /// Bit indicando entrada válida.
    const VALID: u64 = 1 << 0;

    /// Cria entrada inválida (não mapeada).
    pub const fn invalid() -> Self {
        Self(0)
    }

    /// Cria entrada válida apontando para endereço físico.
    pub fn new(phys: PhysAddr) -> Self {
        let addr = phys.as_u64() & !0xFFF; // Alinhado a 4KB
        Self(addr | Self::VALID)
    }

    /// Verifica se entrada é válida.
    pub fn is_valid(&self) -> bool {
        (self.0 & Self::VALID) != 0
    }

    /// Retorna endereço físico (se válida).
    pub fn physical_address(&self) -> Option<PhysAddr> {
        if self.is_valid() {
            Some(PhysAddr::new(self.0 & !0xFFF))
        } else {
            None
        }
    }

    /// Retorna valor raw.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

// =============================================================================
// GTT RANGE
// =============================================================================

/// Range de páginas na GTT.
#[derive(Clone, Copy, Debug)]
pub struct GttRange {
    /// Offset inicial (em páginas).
    pub start: u32,
    /// Número de páginas.
    pub count: u32,
}

impl GttRange {
    /// Cria novo range.
    pub fn new(start: u32, count: u32) -> Self {
        Self { start, count }
    }

    /// Offset em bytes na GTT.
    pub fn byte_offset(&self) -> usize {
        self.start as usize * core::mem::size_of::<GttEntry>()
    }

    /// Tamanho em bytes que este range mapeia.
    pub fn mapped_size(&self) -> usize {
        self.count as usize * 4096
    }
}
