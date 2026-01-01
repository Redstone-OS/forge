//! # RBTree para VMAs
//!
//! Árvore Red-Black balanceada para VMAs (placeholder).
//! Por enquanto usa Vec, será substituído por RBTree real.

use super::vma::VMA;
use crate::mm::VirtAddr;

/// RBTree simplificada (usa Vec internamente)
pub struct RBTree<K, V> {
    items: alloc::vec::Vec<(K, V)>,
}

extern crate alloc;

impl<K: Ord + Copy, V> RBTree<K, V> {
    pub const fn new() -> Self {
        Self {
            items: alloc::vec::Vec::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let pos = self
            .items
            .iter()
            .position(|(k, _)| *k >= key)
            .unwrap_or(self.items.len());
        self.items.insert(pos, (key, value));
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.items
            .iter()
            .position(|(k, _)| k == key)
            .map(|i| self.items.remove(i).1)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn find_containing(&self, addr: VirtAddr) -> Option<&V>
    where
        V: AsRef<VMA>,
    {
        self.items
            .iter()
            .find(|(_, v)| v.as_ref().contains(addr))
            .map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.items.iter().map(|(k, v)| (k, v))
    }
}

impl<K: Ord + Copy, V> Default for RBTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
