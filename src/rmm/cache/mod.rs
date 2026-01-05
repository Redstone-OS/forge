//! # Page Cache (Stub)
//!
//! Cache de páginas de arquivos em RAM.
//! STUB: Será implementado quando VFS e page reclaim estiverem prontos.

use crate::rmm::addr::PhysAddr;

/// ID de inode
pub type InodeId = u64;

/// Busca página no cache
pub fn lookup(_inode: InodeId, _offset: u64) -> Option<PhysAddr> {
    // TODO: Implementar quando VFS estiver pronto
    None
}

/// Insere página no cache
pub fn insert(_inode: InodeId, _offset: u64, _phys: PhysAddr) {
    // TODO: Implementar
}

/// Invalida páginas de um inode
pub fn invalidate(_inode: InodeId) {
    // TODO: Implementar
}

/// Flush páginas sujas de um inode
pub fn writeback(_inode: InodeId) {
    // TODO: Implementar
}

/// Estatísticas do cache
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub pages: u64,
}

/// Retorna estatísticas
pub fn stats() -> CacheStats {
    CacheStats {
        hits: 0,
        misses: 0,
        pages: 0,
    }
}
