//! # Swap Slot Allocator
//!
//! Gerencia alocação de slots no dispositivo de swap.
//!
//! ## Conceito
//!
//! Cada slot corresponde a uma página (4KB). O alocador usa um bitmap
//! para rastrear quais slots estão em uso.
//!
//! ## Estrutura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Swap Device (ex: 1GB = 262144 slots)           │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Bitmap: [11110000][10101010][00000000]...                   │
//! │          ↑↑↑↑      ↑ ↑ ↑ ↑                                 │
//! │          used      mixed    free                            │
//! │                                                             │
//! │ Next Free Hint: slot 12 (otimização)                       │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

// =============================================================================
// SwapSlot
// =============================================================================

/// Slot individual no swap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SwapSlot(pub u64);

impl SwapSlot {
    /// Slot inválido
    pub const INVALID: Self = Self(u64::MAX);

    /// Cria novo slot
    #[inline]
    pub const fn new(index: u64) -> Self {
        Self(index)
    }

    /// Retorna índice do slot
    #[inline]
    pub const fn index(&self) -> u64 {
        self.0
    }

    /// Verifica se slot é válido
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 != u64::MAX
    }
}

// =============================================================================
// SlotAllocator
// =============================================================================

/// Alocador de slots para swap device
pub struct SlotAllocator {
    /// Bitmap de slots (1 = usado, 0 = livre)
    bitmap: Vec<AtomicU64>,
    /// Total de slots
    total_slots: u64,
    /// Slots em uso
    used_slots: AtomicU64,
    /// Hint para próximo slot livre (otimização)
    next_free_hint: AtomicU64,
}

impl SlotAllocator {
    /// Bits por elemento do bitmap
    const BITS_PER_WORD: u64 = 64;

    /// Cria novo alocador para `total_slots` slots
    pub fn new(total_slots: u64) -> Self {
        let words = ((total_slots + Self::BITS_PER_WORD - 1) / Self::BITS_PER_WORD) as usize;
        let mut bitmap = Vec::with_capacity(words);

        for _ in 0..words {
            bitmap.push(AtomicU64::new(0));
        }

        Self {
            bitmap,
            total_slots,
            used_slots: AtomicU64::new(0),
            next_free_hint: AtomicU64::new(0),
        }
    }

    /// Aloca um slot livre
    pub fn alloc(&self) -> Option<SwapSlot> {
        let hint = self.next_free_hint.load(Ordering::Relaxed);
        let start_word = (hint / Self::BITS_PER_WORD) as usize;

        // Procura a partir do hint
        for offset in 0..self.bitmap.len() {
            let word_idx = (start_word + offset) % self.bitmap.len();

            if let Some(bit) = self.try_alloc_from_word(word_idx) {
                let slot = (word_idx as u64 * Self::BITS_PER_WORD) + bit;

                if slot < self.total_slots {
                    // Atualiza hint
                    self.next_free_hint.store(slot + 1, Ordering::Relaxed);
                    self.used_slots.fetch_add(1, Ordering::Relaxed);

                    return Some(SwapSlot(slot));
                }
            }
        }

        None
    }

    /// Tenta alocar um bit de uma word específica
    fn try_alloc_from_word(&self, word_idx: usize) -> Option<u64> {
        let word = &self.bitmap[word_idx];

        loop {
            let current = word.load(Ordering::Acquire);

            // Procura primeiro bit livre (0)
            let free_bit = (!current).trailing_zeros() as u64;

            if free_bit >= Self::BITS_PER_WORD {
                return None; // Word cheia
            }

            // Tenta setar o bit
            let new_value = current | (1 << free_bit);

            if word
                .compare_exchange(current, new_value, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return Some(free_bit);
            }

            // Falhou, outra thread modificou, tenta novamente
        }
    }

    /// Libera um slot
    pub fn free(&self, slot: SwapSlot) {
        if slot.0 >= self.total_slots {
            return;
        }

        let word_idx = (slot.0 / Self::BITS_PER_WORD) as usize;
        let bit_idx = slot.0 % Self::BITS_PER_WORD;

        // Limpa o bit
        self.bitmap[word_idx].fetch_and(!(1 << bit_idx), Ordering::Release);
        self.used_slots.fetch_sub(1, Ordering::Relaxed);

        // Atualiza hint se este slot é menor
        let _ = self.next_free_hint.fetch_min(slot.0, Ordering::Relaxed);
    }

    /// Verifica se slot está alocado
    pub fn is_allocated(&self, slot: SwapSlot) -> bool {
        if slot.0 >= self.total_slots {
            return false;
        }

        let word_idx = (slot.0 / Self::BITS_PER_WORD) as usize;
        let bit_idx = slot.0 % Self::BITS_PER_WORD;

        (self.bitmap[word_idx].load(Ordering::Relaxed) & (1 << bit_idx)) != 0
    }

    /// Total de slots
    #[inline]
    pub fn total(&self) -> u64 {
        self.total_slots
    }

    /// Slots em uso
    #[inline]
    pub fn used(&self) -> u64 {
        self.used_slots.load(Ordering::Relaxed)
    }

    /// Slots livres
    #[inline]
    pub fn free_count(&self) -> u64 {
        self.total_slots.saturating_sub(self.used())
    }

    /// Percentual de uso
    pub fn usage_percent(&self) -> u64 {
        if self.total_slots == 0 {
            0
        } else {
            (self.used() * 100) / self.total_slots
        }
    }

    /// Reseta todos os slots para livre
    pub fn reset(&self) {
        for word in &self.bitmap {
            word.store(0, Ordering::Release);
        }
        self.used_slots.store(0, Ordering::Release);
        self.next_free_hint.store(0, Ordering::Release);
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_alloc_free() {
        let allocator = SlotAllocator::new(100);

        assert_eq!(allocator.used(), 0);

        let slot1 = allocator.alloc().unwrap();
        assert_eq!(slot1.0, 0);
        assert_eq!(allocator.used(), 1);
        assert!(allocator.is_allocated(slot1));

        let slot2 = allocator.alloc().unwrap();
        assert_eq!(slot2.0, 1);
        assert_eq!(allocator.used(), 2);

        allocator.free(slot1);
        assert_eq!(allocator.used(), 1);
        assert!(!allocator.is_allocated(slot1));

        // Realloc deve dar slot1 de volta
        let slot3 = allocator.alloc().unwrap();
        assert_eq!(slot3.0, 0);
    }

    #[test]
    fn test_slot_exhaustion() {
        let allocator = SlotAllocator::new(3);

        let _ = allocator.alloc().unwrap();
        let _ = allocator.alloc().unwrap();
        let _ = allocator.alloc().unwrap();

        assert!(allocator.alloc().is_none());
    }
}
