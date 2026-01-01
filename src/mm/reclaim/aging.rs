//! # Page Aging
//!
//! CLOCK-Pro / Two-list LRU para distinguir páginas quentes de frias.

use core::sync::atomic::{AtomicU64, Ordering};

/// Estatísticas de aging
pub static PAGES_AGED: AtomicU64 = AtomicU64::new(0);
pub static PAGES_PROMOTED: AtomicU64 = AtomicU64::new(0);
pub static PAGES_DEMOTED: AtomicU64 = AtomicU64::new(0);

/// Page Ager - implementa CLOCK algorithm
pub struct PageAger {
    /// Hand position
    hand: usize,
    /// Número de páginas no sistema
    page_count: usize,
}

impl PageAger {
    pub fn new(page_count: usize) -> Self {
        Self {
            hand: 0,
            page_count,
        }
    }

    /// Avança o clock e retorna páginas candidatas a eviction
    pub fn tick(&mut self) -> Option<u64> {
        if self.page_count == 0 {
            return None;
        }

        // Avançar hand
        self.hand = (self.hand + 1) % self.page_count;
        PAGES_AGED.fetch_add(1, Ordering::Relaxed);

        // TODO: Verificar bit de acesso e decidir
        None
    }

    /// Marca página como acessada recentemente
    pub fn mark_accessed(&mut self, _page_idx: usize) {
        PAGES_PROMOTED.fetch_add(1, Ordering::Relaxed);
    }

    /// Reseta bit de acesso de uma página
    pub fn clear_accessed(&mut self, _page_idx: usize) {
        PAGES_DEMOTED.fetch_add(1, Ordering::Relaxed);
    }
}

/// Lista LRU simples
pub struct LruList {
    active: alloc::collections::VecDeque<u64>,
    inactive: alloc::collections::VecDeque<u64>,
}

extern crate alloc;

impl LruList {
    pub fn new() -> Self {
        Self {
            active: alloc::collections::VecDeque::new(),
            inactive: alloc::collections::VecDeque::new(),
        }
    }

    /// Adiciona página como ativa
    pub fn add_active(&mut self, page: u64) {
        self.active.push_back(page);
    }

    /// Move página de active para inactive
    pub fn demote(&mut self) {
        if let Some(page) = self.active.pop_front() {
            self.inactive.push_back(page);
        }
    }

    /// Retorna página candidata a eviction
    pub fn get_victim(&mut self) -> Option<u64> {
        self.inactive.pop_front()
    }
}

impl Default for LruList {
    fn default() -> Self {
        Self::new()
    }
}
