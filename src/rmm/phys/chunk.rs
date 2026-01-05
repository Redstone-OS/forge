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
//!
//! ## Bitmap
//!
//! Usa 8 x u64 = 512 bits para rastrear 512 frames.
//! Bit 1 = usado, bit 0 = livre.

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::{FRAMES_PER_CHUNK, PAGE_SIZE};
use crate::rmm::zone::MigrateType;
use crate::sync::Spinlock;
use core::sync::atomic::{AtomicU64, Ordering};

/// Número de palavras no bitmap (512 frames / 64 bits = 8)
const BITMAP_WORDS: usize = FRAMES_PER_CHUNK / 64;

/// Gerenciador de um chunk de 2MB (512 frames)
pub struct ChunkManager {
    /// Lock deste chunk (para slow path)
    lock: Spinlock<()>,
    /// Bitmap de frames (8 x u64 = 512 bits, 1 = usado, 0 = livre)
    bitmap: [AtomicU64; BITMAP_WORDS],
    /// Base física deste chunk
    base: PhysAddr,
    /// MigrateType predominante deste chunk
    migrate_type: MigrateType,
    /// Contador de frames livres neste chunk
    free_count: AtomicU64,
}

impl ChunkManager {
    /// Cria novo ChunkManager
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
            free_count: AtomicU64::new(FRAMES_PER_CHUNK as u64),
        }
    }

    /// Tenta alocar um frame (com lock)
    ///
    /// Retorna o endereço físico do frame alocado, ou None se chunk cheio.
    pub fn alloc(&self) -> Option<PhysAddr> {
        let _guard = self.lock.lock();

        // Procurar em cada word do bitmap
        for word_idx in 0..BITMAP_WORDS {
            let bitmap = self.bitmap[word_idx].load(Ordering::Acquire);

            // Pula se word está cheia
            if bitmap == u64::MAX {
                continue;
            }

            // Encontra primeiro bit zero usando trailing_zeros
            let inverted = !bitmap;
            let bit_idx = inverted.trailing_zeros() as usize;

            if bit_idx < 64 {
                // Marca como usado (set bit)
                let mask = 1u64 << bit_idx;
                self.bitmap[word_idx].fetch_or(mask, Ordering::Release);
                self.free_count.fetch_sub(1, Ordering::Relaxed);

                // Calcula endereço físico
                let frame_idx = word_idx * 64 + bit_idx;
                let phys = self.base + (frame_idx * PAGE_SIZE) as u64;
                return Some(phys);
            }
        }

        None
    }

    /// Tenta alocar sem bloquear (para contexto IRQ)
    pub fn try_alloc(&self) -> Option<PhysAddr> {
        match self.lock.try_lock() {
            Some(_guard) => {
                // Mesmo código de alloc, mas sem bloquear
                for word_idx in 0..BITMAP_WORDS {
                    let bitmap = self.bitmap[word_idx].load(Ordering::Acquire);

                    if bitmap == u64::MAX {
                        continue;
                    }

                    let inverted = !bitmap;
                    let bit_idx = inverted.trailing_zeros() as usize;

                    if bit_idx < 64 {
                        let mask = 1u64 << bit_idx;
                        self.bitmap[word_idx].fetch_or(mask, Ordering::Release);
                        self.free_count.fetch_sub(1, Ordering::Relaxed);

                        let frame_idx = word_idx * 64 + bit_idx;
                        let phys = self.base + (frame_idx * PAGE_SIZE) as u64;
                        return Some(phys);
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Aloca frames contíguos dentro deste chunk
    ///
    /// Usa busca de run contíguo no bitmap.
    pub fn alloc_contiguous(&self, count: usize) -> Option<PhysAddr> {
        if count == 0 || count > FRAMES_PER_CHUNK {
            return None;
        }

        let _guard = self.lock.lock();

        // Buscar run contíguo
        if let Some(start_bit) = self.find_contiguous_run(count) {
            // Marcar todos como usados
            self.set_range(start_bit, count);
            self.free_count.fetch_sub(count as u64, Ordering::Relaxed);

            let phys = self.base + (start_bit * PAGE_SIZE) as u64;
            return Some(phys);
        }

        None
    }

    /// Libera um frame
    pub fn free(&self, phys: PhysAddr) -> bool {
        let offset = phys.as_u64().saturating_sub(self.base.as_u64()) as usize;
        let frame_idx = offset / PAGE_SIZE;

        if frame_idx >= FRAMES_PER_CHUNK {
            return false;
        }

        let word_idx = frame_idx / 64;
        let bit_idx = frame_idx % 64;
        let mask = 1u64 << bit_idx;

        // Clear bit (marca como livre)
        let old = self.bitmap[word_idx].fetch_and(!mask, Ordering::Release);

        // Verifica se estava realmente alocado
        if (old & mask) != 0 {
            self.free_count.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            // Double free!
            #[cfg(debug_assertions)]
            crate::kwarn!("(RMM) Double free detected: phys 0x{:x}", phys.as_u64());
            false
        }
    }

    /// Libera range contíguo de frames
    pub fn free_contiguous(&self, phys: PhysAddr, count: usize) -> bool {
        let offset = phys.as_u64().saturating_sub(self.base.as_u64()) as usize;
        let start_frame = offset / PAGE_SIZE;

        if start_frame + count > FRAMES_PER_CHUNK {
            return false;
        }

        let _guard = self.lock.lock();

        self.clear_range(start_frame, count);
        self.free_count.fetch_add(count as u64, Ordering::Relaxed);

        true
    }

    /// Conta frames livres neste chunk
    #[inline]
    pub fn free_count(&self) -> usize {
        self.free_count.load(Ordering::Relaxed) as usize
    }

    /// Verifica se chunk está cheio
    #[inline]
    pub fn is_full(&self) -> bool {
        self.free_count.load(Ordering::Relaxed) == 0
    }

    /// Verifica se chunk está vazio (todos livres)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.free_count.load(Ordering::Relaxed) == FRAMES_PER_CHUNK as u64
    }

    /// Retorna MigrateType deste chunk
    #[inline]
    pub fn migrate_type(&self) -> MigrateType {
        self.migrate_type
    }

    /// Retorna base física deste chunk
    #[inline]
    pub fn base(&self) -> PhysAddr {
        self.base
    }

    /// Verifica se frame específico está alocado
    pub fn is_allocated(&self, frame_idx: usize) -> bool {
        if frame_idx >= FRAMES_PER_CHUNK {
            return false;
        }

        let word_idx = frame_idx / 64;
        let bit_idx = frame_idx % 64;
        let mask = 1u64 << bit_idx;

        (self.bitmap[word_idx].load(Ordering::Relaxed) & mask) != 0
    }

    // -------------------------------------------------------------------------
    // Helpers Internos
    // -------------------------------------------------------------------------

    /// Encontra run contíguo de bits zero no bitmap
    fn find_contiguous_run(&self, count: usize) -> Option<usize> {
        let mut run_start = 0;
        let mut run_len = 0;

        for global_bit in 0..FRAMES_PER_CHUNK {
            let word_idx = global_bit / 64;
            let bit_idx = global_bit % 64;
            let mask = 1u64 << bit_idx;

            let is_free = (self.bitmap[word_idx].load(Ordering::Relaxed) & mask) == 0;

            if is_free {
                if run_len == 0 {
                    run_start = global_bit;
                }
                run_len += 1;
                if run_len >= count {
                    return Some(run_start);
                }
            } else {
                run_len = 0;
            }
        }

        None
    }

    /// Define range de bits como usados
    fn set_range(&self, start: usize, count: usize) {
        for i in 0..count {
            let bit = start + i;
            let word_idx = bit / 64;
            let bit_idx = bit % 64;
            let mask = 1u64 << bit_idx;
            self.bitmap[word_idx].fetch_or(mask, Ordering::Relaxed);
        }
    }

    /// Limpa range de bits (marca como livres)
    fn clear_range(&self, start: usize, count: usize) {
        for i in 0..count {
            let bit = start + i;
            let word_idx = bit / 64;
            let bit_idx = bit % 64;
            let mask = 1u64 << bit_idx;
            self.bitmap[word_idx].fetch_and(!mask, Ordering::Relaxed);
        }
    }
}

// Safety: ChunkManager usa lock interno e operações atômicas
unsafe impl Send for ChunkManager {}
unsafe impl Sync for ChunkManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_alloc_free() {
        let chunk = ChunkManager::new(PhysAddr::new(0), MigrateType::Movable);

        // Aloca frame
        let frame = chunk.alloc().expect("should alloc");
        assert_eq!(frame.as_u64(), 0);
        assert_eq!(chunk.free_count(), FRAMES_PER_CHUNK - 1);

        // Libera frame
        assert!(chunk.free(frame));
        assert_eq!(chunk.free_count(), FRAMES_PER_CHUNK);
    }

    #[test]
    fn test_chunk_contiguous() {
        let chunk = ChunkManager::new(PhysAddr::new(0), MigrateType::Movable);

        // Aloca 10 frames contíguos
        let frames = chunk.alloc_contiguous(10).expect("should alloc contiguous");
        assert_eq!(frames.as_u64(), 0);
        assert_eq!(chunk.free_count(), FRAMES_PER_CHUNK - 10);

        // Libera todos
        assert!(chunk.free_contiguous(frames, 10));
        assert_eq!(chunk.free_count(), FRAMES_PER_CHUNK);
    }
}
