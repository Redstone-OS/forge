//! # Writeback
//!
//! Escrita de páginas sujas de volta para armazenamento.
//!
//! ## Conceito
//!
//! Quando páginas no cache são modificadas, ficam marcadas como DIRTY.
//! O writeback é o processo de sincronizar essas páginas com o disco.
//!
//! ## Modos
//!
//! - **Background**: Thread periódica escreve páginas antigas
//! - **Sync**: Flush imediato (fsync, sync)
//! - **Pressure**: Writeback forçado sob pressão de memória
//!
//! ## Integração
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Writeback Flow                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  Page Cache ──► Writeback Queue ──► VFS Write ──► Disk      │
//! │     (DIRTY)         (sorted)          (async)               │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use super::entry::{CacheEntry, CacheKey};
use crate::rmm::addr::PhysAddr;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

// =============================================================================
// WritebackRequest
// =============================================================================

/// Requisição de writeback
#[derive(Debug, Clone)]
pub struct WritebackRequest {
    /// Inode do arquivo
    pub inode: u64,
    /// Offset no arquivo
    pub offset: u64,
    /// Endereço físico da página
    pub phys: PhysAddr,
    /// Timestamp de quando ficou dirty
    pub dirty_time: u64,
}

impl WritebackRequest {
    /// Cria nova requisição
    pub fn new(inode: u64, offset: u64, phys: PhysAddr) -> Self {
        Self {
            inode,
            offset,
            phys,
            dirty_time: 0, // Será preenchido pelo caller
        }
    }

    /// Cria a partir de CacheEntry
    pub fn from_entry(entry: &CacheEntry) -> Self {
        Self {
            inode: entry.inode(),
            offset: entry.offset(),
            phys: entry.phys(),
            dirty_time: 0,
        }
    }

    /// Retorna chave do cache
    pub fn key(&self) -> CacheKey {
        CacheKey::new(self.inode, self.offset)
    }
}

// =============================================================================
// WritebackQueue
// =============================================================================

/// Fila de writeback pendente
pub struct WritebackQueue {
    /// Requisições pendentes
    queue: VecDeque<WritebackRequest>,
    /// Páginas em writeback atualmente
    in_flight: usize,
    /// Máximo de páginas em flight
    max_in_flight: usize,
}

impl WritebackQueue {
    /// Cria nova fila (const fn)
    pub const fn new(max_in_flight: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            in_flight: 0,
            max_in_flight,
        }
    }

    /// Adiciona requisição à fila
    pub fn enqueue(&mut self, request: WritebackRequest) {
        self.queue.push_back(request);
    }

    /// Remove próxima requisição
    pub fn dequeue(&mut self) -> Option<WritebackRequest> {
        if self.in_flight >= self.max_in_flight {
            return None;
        }

        self.queue.pop_front().map(|req| {
            self.in_flight += 1;
            req
        })
    }

    /// Marca requisição como completa
    pub fn complete(&mut self) {
        self.in_flight = self.in_flight.saturating_sub(1);
    }

    /// Número de requisições pendentes
    #[inline]
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Número de requisições em flight
    #[inline]
    pub fn in_flight(&self) -> usize {
        self.in_flight
    }

    /// Está vazia?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty() && self.in_flight == 0
    }

    /// Remove todas requisições de um inode
    pub fn remove_inode(&mut self, inode: u64) {
        self.queue.retain(|r| r.inode != inode);
    }

    /// Limpa a fila
    pub fn clear(&mut self) {
        self.queue.clear();
        self.in_flight = 0;
    }
}

impl Default for WritebackQueue {
    fn default() -> Self {
        Self::new(32) // 32 páginas em flight por default
    }
}

// =============================================================================
// WritebackStats
// =============================================================================

/// Estatísticas de writeback
#[derive(Debug, Clone, Copy, Default)]
pub struct WritebackStats {
    /// Páginas escritas com sucesso
    pub pages_written: u64,
    /// Erros de escrita
    pub write_errors: u64,
    /// Bytes escritos
    pub bytes_written: u64,
}

// =============================================================================
// Writeback Control
// =============================================================================

/// Controle de writeback global
pub struct WritebackControl {
    /// Fila de writeback
    queue: WritebackQueue,
    /// Estatísticas
    stats: WritebackStats,
    /// Intervalo entre background writebacks (ms)
    pub interval_ms: u64,
    /// Idade máxima de página dirty antes de writeback (ms)
    pub dirty_expire_ms: u64,
    /// Ratio de páginas dirty que trigger writeback (%)
    pub dirty_ratio: u32,
    /// Ratio que trigger writeback em background (%)
    pub dirty_background_ratio: u32,
}

impl WritebackControl {
    /// Cria novo controle com defaults (const fn)
    pub const fn new() -> Self {
        Self {
            queue: WritebackQueue::new(32),
            stats: WritebackStats {
                pages_written: 0,
                write_errors: 0,
                bytes_written: 0,
            },
            interval_ms: 5000,          // 5 segundos
            dirty_expire_ms: 30000,     // 30 segundos
            dirty_ratio: 20,            // 20%
            dirty_background_ratio: 10, // 10%
        }
    }

    /// Enfileira páginas para writeback
    pub fn queue_pages(&mut self, entries: &[&CacheEntry]) {
        for entry in entries {
            if entry.is_dirty() && !entry.is_writeback() {
                let request = WritebackRequest::from_entry(entry);
                self.queue.enqueue(request);
            }
        }
    }

    /// Processa próxima requisição
    ///
    /// Retorna requisição se disponível. Caller deve chamar `complete_write`
    /// após escrever a página.
    pub fn next_request(&mut self) -> Option<WritebackRequest> {
        self.queue.dequeue()
    }

    /// Marca escrita como completa
    pub fn complete_write(&mut self, success: bool, bytes: usize) {
        self.queue.complete();

        if success {
            self.stats.pages_written += 1;
            self.stats.bytes_written += bytes as u64;
        } else {
            self.stats.write_errors += 1;
        }
    }

    /// Retorna estatísticas
    pub fn stats(&self) -> WritebackStats {
        self.stats
    }

    /// Requisições pendentes
    pub fn pending(&self) -> usize {
        self.queue.pending()
    }

    /// Limpa fila de um inode
    pub fn remove_inode(&mut self, inode: u64) {
        self.queue.remove_inode(inode);
    }

    /// Limpa tudo
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Default for WritebackControl {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Writeback Functions
// =============================================================================

/// Contador global de timestamp
static WRITEBACK_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// Retorna timestamp atual (incrementado a cada tick)
pub fn current_timestamp() -> u64 {
    WRITEBACK_TIMESTAMP.load(Ordering::Relaxed)
}

/// Incrementa timestamp (chamado pelo timer)
pub fn tick_timestamp() {
    WRITEBACK_TIMESTAMP.fetch_add(1, Ordering::Relaxed);
}

/// Verifica se página está velha o suficiente para writeback
pub fn is_page_expired(dirty_time: u64, expire_threshold: u64) -> bool {
    let now = current_timestamp();
    now.saturating_sub(dirty_time) >= expire_threshold
}

/// Coleta páginas para writeback background
///
/// Retorna lista de chaves de páginas que devem ser escritas.
pub fn collect_for_writeback(
    dirty_pages: &[&CacheEntry],
    max_pages: usize,
    expire_threshold: u64,
) -> Vec<CacheKey> {
    let mut result = Vec::with_capacity(max_pages.min(dirty_pages.len()));

    for entry in dirty_pages {
        if result.len() >= max_pages {
            break;
        }

        // Só páginas não locked e não em writeback
        if entry.is_evictable() || entry.is_dirty() {
            result.push(entry.key());
        }
    }

    result
}

// =============================================================================
// Placeholder Write Function
// =============================================================================

/// Escreve página para disco (STUB)
///
/// TODO: Integrar com VFS quando estiver pronto
pub fn write_page_to_disk(_inode: u64, _offset: u64, _phys: PhysAddr) -> Result<(), ()> {
    // STUB: Não faz nada por enquanto
    // Quando VFS estiver pronto:
    // 1. Obter file handle do inode
    // 2. Seek para offset
    // 3. Escrever PAGE_SIZE bytes
    Ok(())
}
