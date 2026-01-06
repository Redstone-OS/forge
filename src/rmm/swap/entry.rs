//! # Swap Entry
//!
//! Representação de uma entrada de swap no PTE.
//!
//! ## Layout do PTE quando página está em swap
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ 63                           8   7        1   0             │
//! │ ├──────────────────────────────┼───────────┼───┤            │
//! │ │         Slot Offset          │  Device   │ P │            │
//! │ │         (56 bits)            │  (7 bits) │ 0 │            │
//! │ └──────────────────────────────┴───────────┴───┘            │
//! │                                                             │
//! │ P = 0: Not present (indica swap entry)                      │
//! │ Device: Índice do dispositivo de swap (0-127)               │
//! │ Slot Offset: Offset no dispositivo                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use super::slot::SwapSlot;

// =============================================================================
// SwapEntry
// =============================================================================

/// Entrada de swap (armazenada no PTE quando página está em disco)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SwapEntry(u64);

impl SwapEntry {
    /// Bit que indica swap entry (presente = 0)
    const PRESENT_BIT: u64 = 1 << 0;

    /// Máscara para device (bits 1-7)
    const DEVICE_MASK: u64 = 0x7F << 1;
    const DEVICE_SHIFT: u64 = 1;

    // Todo: Revisar
    #[allow(unused)]
    /// Máscara para slot (bits 8-63)
    const SLOT_MASK: u64 = !0xFF;
    const SLOT_SHIFT: u64 = 8;

    /// Cria nova entrada de swap
    #[inline]
    pub fn new(device: u8, slot: SwapSlot) -> Self {
        debug_assert!(device < 128, "device index too large");

        let value = ((device as u64) << Self::DEVICE_SHIFT) | ((slot.0 as u64) << Self::SLOT_SHIFT);

        // Bit 0 = 0 indica não presente (swap entry)
        Self(value)
    }

    /// Cria entrada vazia (inválida)
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Verifica se é entrada de swap válida
    #[inline]
    pub fn is_valid(&self) -> bool {
        // Entrada válida tem pelo menos slot > 0 ou device > 0
        self.0 != 0 && (self.0 & Self::PRESENT_BIT) == 0
    }

    /// Verifica se é PTE presente (não é swap entry)
    #[inline]
    pub fn is_present(&self) -> bool {
        (self.0 & Self::PRESENT_BIT) != 0
    }

    /// Retorna índice do dispositivo
    #[inline]
    pub fn device(&self) -> u8 {
        ((self.0 & Self::DEVICE_MASK) >> Self::DEVICE_SHIFT) as u8
    }

    /// Retorna slot no dispositivo
    #[inline]
    pub fn slot(&self) -> SwapSlot {
        SwapSlot((self.0 >> Self::SLOT_SHIFT) as u64)
    }

    /// Retorna valor raw para armazenar no PTE
    #[inline]
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Cria a partir de valor raw do PTE
    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

impl From<u64> for SwapEntry {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<SwapEntry> for u64 {
    fn from(entry: SwapEntry) -> Self {
        entry.0
    }
}

// =============================================================================
// SwapType
// =============================================================================

/// Tipo de swap entry para distinguir de outros valores
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapType {
    /// Página em swap device
    Swap(SwapEntry),
    /// Página migrada (não implementado)
    Migration,
    /// Arquivo (file-backed, não usa swap)
    File,
    /// Nenhum (página vazia)
    None,
}

impl SwapType {
    /// Detecta tipo a partir do valor raw do PTE
    pub fn from_pte(pte: u64) -> Self {
        if pte == 0 {
            return Self::None;
        }

        // Bit 0 = 1 significa presente (não é swap)
        if (pte & 1) != 0 {
            return Self::File; // Assume file-backed se presente
        }

        // Bits 1-7 são device index
        let entry = SwapEntry::from_raw(pte);
        if entry.is_valid() {
            Self::Swap(entry)
        } else {
            Self::None
        }
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_entry_create() {
        let entry = SwapEntry::new(0, SwapSlot(42));

        assert!(!entry.is_present());
        assert!(entry.is_valid());
        assert_eq!(entry.device(), 0);
        assert_eq!(entry.slot().0, 42);
    }

    #[test]
    fn test_swap_entry_device() {
        let entry = SwapEntry::new(5, SwapSlot(100));

        assert_eq!(entry.device(), 5);
        assert_eq!(entry.slot().0, 100);
    }

    #[test]
    fn test_swap_entry_roundtrip() {
        let original = SwapEntry::new(3, SwapSlot(12345));
        let raw = original.raw();
        let recovered = SwapEntry::from_raw(raw);

        assert_eq!(original, recovered);
        assert_eq!(recovered.device(), 3);
        assert_eq!(recovered.slot().0, 12345);
    }
}
