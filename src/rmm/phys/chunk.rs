//! # ChunkManager - Gerenciador de Chunk
//!
//! Cada chunk gerencia 2MB (512 frames) com seu próprio lock.
//!
//! ## Por que 2MB?
//!
//! O tamanho do chunk deve ser >= maior alocação contígua comum (Huge Page).
//! Com chunks de 2MB, uma Huge Page cabe inteira em um chunk, exigindo apenas 1 lock.
//!
//! ## Lock Ordering
//!
//! Para alocações que precisam de múltiplos chunks (raro, > 2MB):
//! - SEMPRE adquira locks em ordem ASCENDENTE de endereço físico
//! - Isso evita deadlocks entre CPUs
//!
//! ## MigrateType
//!
//! Cada chunk tem um MigrateType predominante:
//! - Unmovable: página não pode ser movida (kernel, page tables)
//! - Movable: pode ser compactada (userspace)
//! - Reclaimable: pode ser liberada (page cache)
//!
//! DMA DEVE alocar de chunks Unmovable para não travar fragmentação.

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::{FRAMES_PER_CHUNK, PAGE_SIZE};
use crate::rmm::zone::MigrateType;
use crate::sync::Spinlock;
use core::sync::atomic::{AtomicU64, Ordering};

/// Gerenciador de um chunk de 2MB (512 frames)
pub struct ChunkManager {
    /// Lock deste chunk
    lock: Spinlock<()>,
    /// Bitmap de frames (8 x u64 = 512 bits, 1 = usado, 0 = livre)
    bitmap: [AtomicU64; 8],
    /// Base física deste chunk
    base: PhysAddr,
    /// MigrateType predominante
    migrate_type: MigrateType,
}

impl ChunkManager {
    pub fn new(base: PhysAddr, migrate_type: MigrateType) -> Self {
        Self {
            lock: Spinlock::new(()),
            bitmap: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            base,
            migrate_type,
        }
    }

    /// Tenta alocar um frame (com lock)
    pub fn alloc(&self) -> Option<PhysAddr> {
        let _guard = self.lock.lock();

        for word_idx in 0..8 {
            let bitmap = self.bitmap[word_idx].load(Ordering::Acquire);

            if bitmap == u64::MAX {
                continue; // Word cheio
            }

            // Encontra primeiro bit zero
            let bit_idx = (!bitmap).trailing_zeros() as usize;
            if bit_idx < 64 {
                self.bitmap[word_idx].fetch_or(1 << bit_idx, Ordering::Release);
                let frame_idx = word_idx * 64 + bit_idx;
                return Some(self.base + (frame_idx * PAGE_SIZE) as u64);
            }
        }
        None
    }

    /// Tenta alocar sem bloquear (para contexto IRQ)
    pub fn try_alloc(&self) -> Option<PhysAddr> {
        if self.lock.try_lock().is_some() {
            self.alloc()
        } else {
            None
        }
    }

    /// Aloca frames contíguos dentro deste chunk
    ///
    /// TODO: Implementar busca de run contíguo no bitmap
    pub fn alloc_contiguous(&self, count: usize) -> Option<PhysAddr> {
        if count > FRAMES_PER_CHUNK {
            return None;
        }
        // TODO: Implementar busca de run
        None
    }

    /// Libera um frame
    pub fn free(&self, phys: PhysAddr) -> bool {
        let offset = (phys.as_u64() - self.base.as_u64()) as usize;
        let idx = offset / PAGE_SIZE;
        if idx >= FRAMES_PER_CHUNK {
            return false;
        }
        let word_idx = idx / 64;
        let bit_idx = idx % 64;
        self.bitmap[word_idx].fetch_and(!(1 << bit_idx), Ordering::Release);
        true
    }

    /// Conta frames livres
    pub fn free_count(&self) -> usize {
        let used: u32 = self
            .bitmap
            .iter()
            .map(|w| w.load(Ordering::Relaxed).count_ones())
            .sum();
        FRAMES_PER_CHUNK - used as usize
    }

    /// Verifica se chunk está cheio
    pub fn is_full(&self) -> bool {
        self.bitmap
            .iter()
            .all(|w| w.load(Ordering::Relaxed) == u64::MAX)
    }

    /// Verifica se chunk está vazio
    pub fn is_empty(&self) -> bool {
        self.bitmap.iter().all(|w| w.load(Ordering::Relaxed) == 0)
    }

    /// Retorna MigrateType
    pub fn migrate_type(&self) -> MigrateType {
        self.migrate_type
    }
}
