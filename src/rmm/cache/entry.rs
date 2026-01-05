//! # Cache Entry
//!
//! Entrada individual no page cache, representando uma página de um arquivo em RAM.
//!
//! ## Layout
//!
//! ```text
//! CacheEntry (32 bytes):
//! ┌────────────┬────────────┬────────────┬────────────┐
//! │   inode    │   offset   │    phys    │   flags    │
//! │  8 bytes   │  8 bytes   │  8 bytes   │  4 bytes   │
//! └────────────┴────────────┴────────────┴────────────┘
//! ```

use crate::rmm::addr::PhysAddr;
use core::sync::atomic::{AtomicU32, Ordering};

// =============================================================================
// CacheFlags
// =============================================================================

/// Flags de uma entrada de cache
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheFlags(u32);

impl CacheFlags {
    /// Nenhuma flag
    pub const NONE: Self = Self(0);
    /// Página foi modificada (precisa writeback)
    pub const DIRTY: Self = Self(1 << 0);
    /// Dados da página são válidos
    pub const UPTODATE: Self = Self(1 << 1);
    /// Página está locked (não pode ser evicted)
    pub const LOCKED: Self = Self(1 << 2);
    /// Página está em writeback para disco
    pub const WRITEBACK: Self = Self(1 << 3);
    /// Página foi referenciada recentemente
    pub const REFERENCED: Self = Self(1 << 4);
    /// Página é privada (não compartilhada)
    pub const PRIVATE: Self = Self(1 << 5);
    /// Página é mapeada em espaço de usuário
    pub const MAPPED: Self = Self(1 << 6);

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    pub const fn bits(&self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}

// =============================================================================
// CacheKey
// =============================================================================

/// Chave única para entrada no cache (inode + offset)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// ID do inode
    pub inode: u64,
    /// Offset dentro do arquivo (alinhado a página)
    pub offset: u64,
}

impl CacheKey {
    /// Cria nova chave
    #[inline]
    pub const fn new(inode: u64, offset: u64) -> Self {
        Self { inode, offset }
    }

    /// Empacota para u128 (para hashing rápido)
    #[inline]
    pub const fn pack(&self) -> u128 {
        ((self.inode as u128) << 64) | (self.offset as u128)
    }
}

// =============================================================================
// CacheEntry
// =============================================================================

/// Entrada no page cache
#[repr(C)]
pub struct CacheEntry {
    /// Chave (inode + offset)
    key: CacheKey,
    /// Endereço físico da página
    phys: PhysAddr,
    /// Flags (atômicas para concurrent access)
    flags: AtomicU32,
}

impl CacheEntry {
    /// Cria nova entrada
    pub fn new(inode: u64, offset: u64, phys: PhysAddr) -> Self {
        Self {
            key: CacheKey::new(inode, offset),
            phys,
            flags: AtomicU32::new(CacheFlags::UPTODATE.bits()),
        }
    }

    /// Retorna a chave
    #[inline]
    pub fn key(&self) -> CacheKey {
        self.key
    }

    /// Retorna o inode
    #[inline]
    pub fn inode(&self) -> u64 {
        self.key.inode
    }

    /// Retorna o offset
    #[inline]
    pub fn offset(&self) -> u64 {
        self.key.offset
    }

    /// Retorna o endereço físico
    #[inline]
    pub fn phys(&self) -> PhysAddr {
        self.phys
    }

    /// Retorna flags atuais
    #[inline]
    pub fn flags(&self) -> CacheFlags {
        CacheFlags::from_bits(self.flags.load(Ordering::Acquire))
    }

    /// Define flags
    #[inline]
    pub fn set_flags(&self, flags: CacheFlags) {
        self.flags.store(flags.bits(), Ordering::Release);
    }

    /// Adiciona flags
    #[inline]
    pub fn add_flags(&self, flags: CacheFlags) {
        self.flags.fetch_or(flags.bits(), Ordering::AcqRel);
    }

    /// Remove flags
    #[inline]
    pub fn remove_flags(&self, flags: CacheFlags) {
        self.flags.fetch_and(!flags.bits(), Ordering::AcqRel);
    }

    /// Verifica se contém flags
    #[inline]
    pub fn has_flags(&self, flags: CacheFlags) -> bool {
        (self.flags.load(Ordering::Acquire) & flags.bits()) == flags.bits()
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Verifica se página está suja
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.has_flags(CacheFlags::DIRTY)
    }

    /// Marca página como suja
    #[inline]
    pub fn mark_dirty(&self) {
        self.add_flags(CacheFlags::DIRTY);
    }

    /// Limpa flag dirty (após writeback)
    #[inline]
    pub fn clear_dirty(&self) {
        self.remove_flags(CacheFlags::DIRTY);
    }

    /// Verifica se página está uptodate
    #[inline]
    pub fn is_uptodate(&self) -> bool {
        self.has_flags(CacheFlags::UPTODATE)
    }

    /// Marca página como uptodate
    #[inline]
    pub fn set_uptodate(&self) {
        self.add_flags(CacheFlags::UPTODATE);
    }

    /// Verifica se página está locked
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.has_flags(CacheFlags::LOCKED)
    }

    /// Tenta lock na página
    ///
    /// Retorna true se conseguiu o lock.
    pub fn try_lock(&self) -> bool {
        let old = self
            .flags
            .fetch_or(CacheFlags::LOCKED.bits(), Ordering::AcqRel);
        (old & CacheFlags::LOCKED.bits()) == 0
    }

    /// Unlock da página
    pub fn unlock(&self) {
        self.remove_flags(CacheFlags::LOCKED);
    }

    /// Verifica se página está em writeback
    #[inline]
    pub fn is_writeback(&self) -> bool {
        self.has_flags(CacheFlags::WRITEBACK)
    }

    /// Marca início de writeback
    pub fn start_writeback(&self) {
        self.add_flags(CacheFlags::WRITEBACK);
    }

    /// Marca fim de writeback
    pub fn end_writeback(&self) {
        self.remove_flags(CacheFlags::WRITEBACK.union(CacheFlags::DIRTY));
    }

    /// Marca como referenciada
    #[inline]
    pub fn mark_referenced(&self) {
        self.add_flags(CacheFlags::REFERENCED);
    }

    /// Limpa e retorna se estava referenciada
    pub fn clear_referenced(&self) -> bool {
        let old = self
            .flags
            .fetch_and(!CacheFlags::REFERENCED.bits(), Ordering::AcqRel);
        (old & CacheFlags::REFERENCED.bits()) != 0
    }

    /// Verifica se pode ser evicted
    #[inline]
    pub fn is_evictable(&self) -> bool {
        let flags = self.flags();
        !flags.contains(CacheFlags::LOCKED)
            && !flags.contains(CacheFlags::WRITEBACK)
            && !flags.contains(CacheFlags::DIRTY)
    }
}

impl Clone for CacheEntry {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            phys: self.phys,
            flags: AtomicU32::new(self.flags.load(Ordering::Relaxed)),
        }
    }
}

// Safety: CacheEntry usa operações atômicas para flags
unsafe impl Send for CacheEntry {}
unsafe impl Sync for CacheEntry {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_flags() {
        let entry = CacheEntry::new(1, 0, PhysAddr::new(0x1000));

        assert!(entry.is_uptodate());
        assert!(!entry.is_dirty());

        entry.mark_dirty();
        assert!(entry.is_dirty());

        entry.clear_dirty();
        assert!(!entry.is_dirty());
    }

    #[test]
    fn test_cache_entry_lock() {
        let entry = CacheEntry::new(1, 0, PhysAddr::new(0x1000));

        assert!(!entry.is_locked());
        assert!(entry.try_lock());
        assert!(entry.is_locked());
        assert!(!entry.try_lock()); // Já está locked

        entry.unlock();
        assert!(!entry.is_locked());
    }
}
