//! # Radix Tree
//!
//! Estrutura de dados para indexação rápida de páginas no cache.
//!
//! ## Conceito
//!
//! Usa uma árvore radix-like simplificada para mapear (inode, offset) → PhysAddr.
//! Para V1, usa HashMap interno. Versão futura terá radix tree real.
//!
//! ## Estrutura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      PageIndex                              │
//! ├─────────────────────────────────────────────────────────────┤
//! │  inode 1: { offset 0 → page, offset 4096 → page, ... }      │
//! │  inode 2: { offset 0 → page, offset 8192 → page, ... }      │
//! │  ...                                                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use super::entry::{CacheEntry, CacheKey};
use crate::rmm::addr::PhysAddr;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

// =============================================================================
// InodePages
// =============================================================================

/// Páginas de um único inode
pub struct InodePages {
    /// Mapeamento offset → entrada de cache
    pages: BTreeMap<u64, CacheEntry>,
}

impl InodePages {
    /// Cria novo container vazio
    pub fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
        }
    }

    /// Insere página
    pub fn insert(&mut self, offset: u64, entry: CacheEntry) -> Option<CacheEntry> {
        self.pages.insert(offset, entry)
    }

    /// Busca página
    pub fn get(&self, offset: u64) -> Option<&CacheEntry> {
        self.pages.get(&offset)
    }

    /// Busca página mutável
    pub fn get_mut(&mut self, offset: u64) -> Option<&mut CacheEntry> {
        self.pages.get_mut(&offset)
    }

    /// Remove página
    pub fn remove(&mut self, offset: u64) -> Option<CacheEntry> {
        self.pages.remove(&offset)
    }

    /// Número de páginas
    #[inline]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Está vazio?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Itera sobre todas as páginas
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &CacheEntry)> {
        self.pages.iter()
    }

    /// Itera mutável
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut CacheEntry)> {
        self.pages.iter_mut()
    }

    /// Limpa todas as páginas
    pub fn clear(&mut self) {
        self.pages.clear();
    }

    /// Coleta páginas em um range
    pub fn range(&self, start: u64, end: u64) -> impl Iterator<Item = (&u64, &CacheEntry)> {
        self.pages.range(start..end)
    }

    /// Coleta páginas sujas
    pub fn dirty_pages(&self) -> Vec<&CacheEntry> {
        self.pages.values().filter(|e| e.is_dirty()).collect()
    }

    /// Coleta páginas evictáveis
    pub fn evictable_pages(&self) -> Vec<&CacheEntry> {
        self.pages.values().filter(|e| e.is_evictable()).collect()
    }
}

impl Default for InodePages {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// PageIndex
// =============================================================================

/// Índice global de páginas em cache
pub struct PageIndex {
    /// Mapeamento inode → páginas
    inodes: BTreeMap<u64, InodePages>,
    /// Total de páginas
    total_pages: usize,
}

impl PageIndex {
    /// Cria novo índice
    pub fn new() -> Self {
        Self {
            inodes: BTreeMap::new(),
            total_pages: 0,
        }
    }

    /// Insere entrada
    pub fn insert(&mut self, entry: CacheEntry) -> Option<CacheEntry> {
        let inode = entry.inode();
        let offset = entry.offset();

        let inode_pages = self.inodes.entry(inode).or_insert_with(InodePages::new);
        let old = inode_pages.insert(offset, entry);

        if old.is_none() {
            self.total_pages += 1;
        }

        old
    }

    /// Busca entrada
    pub fn lookup(&self, key: CacheKey) -> Option<&CacheEntry> {
        self.inodes.get(&key.inode)?.get(key.offset)
    }

    /// Busca por inode e offset
    pub fn get(&self, inode: u64, offset: u64) -> Option<&CacheEntry> {
        self.inodes.get(&inode)?.get(offset)
    }

    /// Busca mutável
    pub fn get_mut(&mut self, inode: u64, offset: u64) -> Option<&mut CacheEntry> {
        self.inodes.get_mut(&inode)?.get_mut(offset)
    }

    /// Remove entrada
    pub fn remove(&mut self, inode: u64, offset: u64) -> Option<CacheEntry> {
        let inode_pages = self.inodes.get_mut(&inode)?;
        let entry = inode_pages.remove(offset)?;

        self.total_pages -= 1;

        // Remove inode se vazio
        if inode_pages.is_empty() {
            self.inodes.remove(&inode);
        }

        Some(entry)
    }

    /// Remove todas as páginas de um inode
    pub fn remove_inode(&mut self, inode: u64) -> Option<InodePages> {
        if let Some(pages) = self.inodes.remove(&inode) {
            self.total_pages -= pages.len();
            Some(pages)
        } else {
            None
        }
    }

    /// Número total de páginas
    #[inline]
    pub fn len(&self) -> usize {
        self.total_pages
    }

    /// Está vazio?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total_pages == 0
    }

    /// Número de inodes
    #[inline]
    pub fn inode_count(&self) -> usize {
        self.inodes.len()
    }

    /// Retorna páginas de um inode
    pub fn get_inode(&self, inode: u64) -> Option<&InodePages> {
        self.inodes.get(&inode)
    }

    /// Retorna páginas mutáveis de um inode
    pub fn get_inode_mut(&mut self, inode: u64) -> Option<&mut InodePages> {
        self.inodes.get_mut(&inode)
    }

    /// Itera sobre todos os inodes
    pub fn iter_inodes(&self) -> impl Iterator<Item = (&u64, &InodePages)> {
        self.inodes.iter()
    }

    /// Coleta todas as páginas sujas
    pub fn all_dirty_pages(&self) -> Vec<&CacheEntry> {
        self.inodes
            .values()
            .flat_map(|pages| pages.dirty_pages())
            .collect()
    }

    /// Coleta páginas evictáveis (LRU candidates)
    pub fn evictable_pages(&self, max: usize) -> Vec<CacheKey> {
        let mut result = Vec::new();

        for (inode, pages) in &self.inodes {
            for entry in pages.evictable_pages() {
                result.push(CacheKey::new(*inode, entry.offset()));
                if result.len() >= max {
                    return result;
                }
            }
        }

        result
    }

    /// Limpa todo o índice
    pub fn clear(&mut self) {
        self.inodes.clear();
        self.total_pages = 0;
    }
}

impl Default for PageIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_index_insert_lookup() {
        let mut index = PageIndex::new();

        let entry = CacheEntry::new(1, 0, PhysAddr::new(0x1000));
        index.insert(entry);

        assert_eq!(index.len(), 1);
        assert!(index.get(1, 0).is_some());
        assert!(index.get(1, 4096).is_none());
    }

    #[test]
    fn test_page_index_remove() {
        let mut index = PageIndex::new();

        index.insert(CacheEntry::new(1, 0, PhysAddr::new(0x1000)));
        index.insert(CacheEntry::new(1, 4096, PhysAddr::new(0x2000)));

        assert_eq!(index.len(), 2);

        index.remove(1, 0);
        assert_eq!(index.len(), 1);
        assert!(index.get(1, 0).is_none());
        assert!(index.get(1, 4096).is_some());
    }

    #[test]
    fn test_page_index_remove_inode() {
        let mut index = PageIndex::new();

        index.insert(CacheEntry::new(1, 0, PhysAddr::new(0x1000)));
        index.insert(CacheEntry::new(1, 4096, PhysAddr::new(0x2000)));
        index.insert(CacheEntry::new(2, 0, PhysAddr::new(0x3000)));

        assert_eq!(index.len(), 3);

        index.remove_inode(1);
        assert_eq!(index.len(), 1);
        assert_eq!(index.inode_count(), 1);
    }
}
