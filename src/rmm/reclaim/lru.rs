//! # LRU Lists
//!
//! Listas LRU (Least Recently Used) para tracking de páginas.
//!
//! ## Estrutura
//!
//! Duas listas por tipo de página (anon/file):
//! - **Active**: Páginas "hot" (acessadas recentemente)
//! - **Inactive**: Páginas "cold" (candidatas para eviction)
//!
//! ## Algoritmo
//!
//! 1. Páginas novas entram no final da inactive
//! 2. Quando acessadas (bit A), movem para active
//! 3. Periodicamente, active é demotado para inactive
//! 4. Páginas no início da inactive são evictadas

use crate::rmm::addr::PhysAddr;
use alloc::collections::VecDeque;

// =============================================================================
// LruListType
// =============================================================================

/// Tipo de lista LRU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LruListType {
    /// Páginas ativas (hot)
    Active,
    /// Páginas inativas (cold)
    Inactive,
}

// =============================================================================
// LruEntry
// =============================================================================

/// Entrada na lista LRU
#[derive(Debug, Clone, Copy)]
struct LruEntry {
    /// Endereço físico da página
    phys: PhysAddr,
    /// Flag de referência (accessed)
    referenced: bool,
}

impl LruEntry {
    fn new(phys: PhysAddr) -> Self {
        Self {
            phys,
            referenced: false,
        }
    }
}

// =============================================================================
// LruList
// =============================================================================

/// Lista LRU dupla (active + inactive)
pub struct LruList {
    /// Lista de páginas ativas
    active: VecDeque<LruEntry>,
    /// Lista de páginas inativas
    inactive: VecDeque<LruEntry>,
    /// Ratio desejado active/inactive (em %)
    active_ratio: usize,
}

impl LruList {
    /// Cria lista vazia
    pub const fn new() -> Self {
        Self {
            active: VecDeque::new(),
            inactive: VecDeque::new(),
            active_ratio: 50, // 50% active, 50% inactive
        }
    }

    /// Adiciona página à lista inactive (nova página)
    pub fn add_inactive(&mut self, phys: PhysAddr) {
        // Verifica se já existe
        if self.contains(phys) {
            return;
        }

        self.inactive.push_back(LruEntry::new(phys));
    }

    /// Adiciona página à lista active
    pub fn add_active(&mut self, phys: PhysAddr) {
        if self.contains(phys) {
            return;
        }

        self.active.push_back(LruEntry::new(phys));
        self.rebalance();
    }

    /// Remove página de qualquer lista
    pub fn remove(&mut self, phys: PhysAddr) -> bool {
        // Tenta remover de active
        if let Some(pos) = self.active.iter().position(|e| e.phys == phys) {
            self.active.remove(pos);
            return true;
        }

        // Tenta remover de inactive
        if let Some(pos) = self.inactive.iter().position(|e| e.phys == phys) {
            self.inactive.remove(pos);
            return true;
        }

        false
    }

    /// Marca página como acessada (promove para active se em inactive)
    pub fn mark_accessed(&mut self, phys: PhysAddr) {
        // Se está em inactive, move para active
        if let Some(pos) = self.inactive.iter().position(|e| e.phys == phys) {
            if let Some(entry) = self.inactive.remove(pos) {
                self.active.push_back(entry);
                self.rebalance();
            }
            return;
        }

        // Se está em active, marca como referenced
        if let Some(entry) = self.active.iter_mut().find(|e| e.phys == phys) {
            entry.referenced = true;
        }
    }

    /// Remove e retorna página do início da inactive
    pub fn pop_inactive(&mut self) -> Option<PhysAddr> {
        self.inactive.pop_front().map(|e| e.phys)
    }

    /// Remove e retorna página do final da inactive
    pub fn pop_inactive_back(&mut self) -> Option<PhysAddr> {
        self.inactive.pop_back().map(|e| e.phys)
    }

    /// Verifica se página está em alguma lista
    pub fn contains(&self, phys: PhysAddr) -> bool {
        self.active.iter().any(|e| e.phys == phys) || self.inactive.iter().any(|e| e.phys == phys)
    }

    /// Retorna em qual lista a página está
    pub fn which_list(&self, phys: PhysAddr) -> Option<LruListType> {
        if self.active.iter().any(|e| e.phys == phys) {
            return Some(LruListType::Active);
        }
        if self.inactive.iter().any(|e| e.phys == phys) {
            return Some(LruListType::Inactive);
        }
        None
    }

    /// Número de páginas na active
    #[inline]
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Número de páginas na inactive
    #[inline]
    pub fn inactive_count(&self) -> usize {
        self.inactive.len()
    }

    /// Total de páginas
    #[inline]
    pub fn total_count(&self) -> usize {
        self.active.len() + self.inactive.len()
    }

    /// Rebalanceia listas para manter ratio
    fn rebalance(&mut self) {
        let total = self.total_count();
        if total == 0 {
            return;
        }

        let target_active = (total * self.active_ratio) / 100;

        // Se active muito grande, demote para inactive
        while self.active.len() > target_active {
            if let Some(mut entry) = self.active.pop_front() {
                if entry.referenced {
                    // Segunda chance: limpa bit e reinsere no final
                    entry.referenced = false;
                    self.active.push_back(entry);
                } else {
                    // Demove para inactive
                    self.inactive.push_back(entry);
                }
            }
        }
    }

    /// Demove N páginas de active para inactive
    pub fn demote(&mut self, count: usize) {
        for _ in 0..count {
            if let Some(mut entry) = self.active.pop_front() {
                if entry.referenced {
                    entry.referenced = false;
                    self.active.push_back(entry);
                } else {
                    self.inactive.push_back(entry);
                }
            } else {
                break;
            }
        }
    }

    /// Rotate: move páginas do início para o final (para scanning)
    pub fn rotate_inactive(&mut self, count: usize) {
        for _ in 0..count {
            if let Some(entry) = self.inactive.pop_front() {
                self.inactive.push_back(entry);
            } else {
                break;
            }
        }
    }

    /// Limpa a bit referenced de todas as páginas
    pub fn clear_referenced(&mut self) {
        for entry in self.active.iter_mut() {
            entry.referenced = false;
        }
        for entry in self.inactive.iter_mut() {
            entry.referenced = false;
        }
    }

    /// Itera sobre páginas inactive (para scanning)
    pub fn iter_inactive(&self) -> impl Iterator<Item = PhysAddr> + '_ {
        self.inactive.iter().map(|e| e.phys)
    }

    /// Itera sobre páginas active
    pub fn iter_active(&self) -> impl Iterator<Item = PhysAddr> + '_ {
        self.active.iter().map(|e| e.phys)
    }

    /// Define ratio active/inactive
    pub fn set_ratio(&mut self, active_percent: usize) {
        self.active_ratio = active_percent.min(90).max(10);
    }

    /// Limpa todas as listas
    pub fn clear(&mut self) {
        self.active.clear();
        self.inactive.clear();
    }
}

impl Default for LruList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_add_remove() {
        let mut lru = LruList::new();

        let p1 = PhysAddr::new(0x1000);
        let p2 = PhysAddr::new(0x2000);

        lru.add_inactive(p1);
        lru.add_inactive(p2);

        assert_eq!(lru.inactive_count(), 2);
        assert!(lru.contains(p1));

        lru.remove(p1);
        assert!(!lru.contains(p1));
        assert_eq!(lru.inactive_count(), 1);
    }

    #[test]
    fn test_lru_promote() {
        let mut lru = LruList::new();

        let p1 = PhysAddr::new(0x1000);
        lru.add_inactive(p1);

        assert_eq!(lru.which_list(p1), Some(LruListType::Inactive));

        lru.mark_accessed(p1);

        assert_eq!(lru.which_list(p1), Some(LruListType::Active));
    }
}
