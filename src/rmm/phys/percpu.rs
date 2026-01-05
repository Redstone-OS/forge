//! # Per-CPU Cache
//!
//! Cache de frames por CPU para fast-path de alocação sem lock global.
//!
//! ## Funcionamento
//!
//! Cada CPU tem um cache local de frames pré-alocados. Isso permite:
//! - Alocação O(1) sem contenção de lock
//! - Melhor localidade de cache
//! - Redução de tráfego de coerência de cache
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │                 CPU 0                       │
//! │  ┌─────────────────────────────────────┐    │
//! │  │ frames[0..31]  │  count: 15         │    │
//! │  └─────────────────────────────────────┘    │
//! └─────────────────────────────────────────────┘
//!                      ↓ pop/push
//! ┌─────────────────────────────────────────────┐
//! │                 Chunks                      │
//! │  (slow path com lock quando cache vazio)   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Refill e Drain
//!
//! - **Refill**: Quando cache está vazio, busca batch de frames dos chunks
//! - **Drain**: Quando cache está cheio, devolve batch para os chunks

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::{MAX_CPUS, PERCPU_CACHE_BATCH, PERCPU_CACHE_SIZE};

/// Cache local de uma CPU
///
/// Usa array fixo para evitar alocação dinâmica.
/// Push/pop são O(1).
#[repr(C, align(64))] // Cache-line aligned para evitar false sharing
pub struct PerCpuCache {
    /// Frames em cache (stack)
    frames: [PhysAddr; PERCPU_CACHE_SIZE],
    /// Quantidade de frames no cache
    count: usize,
    /// High watermark (para stats)
    high_watermark: usize,
    /// Hits de cache
    hits: u64,
    /// Misses de cache
    misses: u64,
}

impl PerCpuCache {
    /// Cria cache vazio
    pub const fn new() -> Self {
        Self {
            frames: [PhysAddr::new(0); PERCPU_CACHE_SIZE],
            count: 0,
            high_watermark: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Remove e retorna frame do cache (O(1))
    ///
    /// Retorna None se cache vazio.
    #[inline]
    pub fn pop(&mut self) -> Option<PhysAddr> {
        if self.count == 0 {
            self.misses += 1;
            return None;
        }

        self.count -= 1;
        self.hits += 1;

        Some(self.frames[self.count])
    }

    /// Adiciona frame ao cache (O(1))
    ///
    /// Retorna false se cache cheio.
    #[inline]
    pub fn push(&mut self, frame: PhysAddr) -> bool {
        if self.count >= PERCPU_CACHE_SIZE {
            return false;
        }

        self.frames[self.count] = frame;
        self.count += 1;

        // Atualiza high watermark
        if self.count > self.high_watermark {
            self.high_watermark = self.count;
        }

        true
    }

    /// Retorna batch de frames para refill de chunk (drain)
    ///
    /// Remove até `max_count` frames e retorna em `out`.
    /// Retorna número de frames copiados.
    pub fn drain(&mut self, out: &mut [PhysAddr], max_count: usize) -> usize {
        let to_drain = self.count.min(max_count).min(out.len());

        for i in 0..to_drain {
            self.count -= 1;
            out[i] = self.frames[self.count];
        }

        to_drain
    }

    /// Adiciona batch de frames (refill do cache)
    ///
    /// Adiciona frames de `src` até encher ou acabar source.
    /// Retorna número de frames adicionados.
    pub fn fill(&mut self, src: &[PhysAddr]) -> usize {
        let available = PERCPU_CACHE_SIZE - self.count;
        let to_fill = src.len().min(available);

        for i in 0..to_fill {
            self.frames[self.count] = src[i];
            self.count += 1;
        }

        if self.count > self.high_watermark {
            self.high_watermark = self.count;
        }

        to_fill
    }

    /// Verifica se cache está vazio
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Verifica se cache está cheio
    #[inline]
    pub fn is_full(&self) -> bool {
        self.count >= PERCPU_CACHE_SIZE
    }

    /// Retorna quantidade de frames no cache
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Verifica se precisa de refill
    #[inline]
    pub fn needs_refill(&self) -> bool {
        self.count < PERCPU_CACHE_BATCH
    }

    /// Verifica se precisa de drain
    #[inline]
    pub fn needs_drain(&self) -> bool {
        self.count > PERCPU_CACHE_SIZE - PERCPU_CACHE_BATCH
    }

    /// Retorna estatísticas
    pub fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }

    /// Hit rate (0.0 - 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// High watermark
    #[inline]
    pub fn high_watermark(&self) -> usize {
        self.high_watermark
    }

    /// Limpa cache (retorna frames para caller processar)
    pub fn clear(&mut self) -> impl Iterator<Item = PhysAddr> + '_ {
        let count = self.count;
        self.count = 0;
        self.frames[..count].iter().copied()
    }
}

impl Default for PerCpuCache {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// PerCpuCaches - Array de caches para todas as CPUs
// =============================================================================

/// Gerenciador de caches per-CPU
///
/// Mantém um cache para cada CPU possível.
pub struct PerCpuCaches {
    /// Array de caches indexado por CPU ID
    caches: [PerCpuCache; MAX_CPUS],
}

impl PerCpuCaches {
    /// Cria caches para todas as CPUs
    pub const fn new() -> Self {
        const EMPTY_CACHE: PerCpuCache = PerCpuCache::new();
        Self {
            caches: [EMPTY_CACHE; MAX_CPUS],
        }
    }

    /// Retorna referência mutável ao cache de uma CPU
    #[inline]
    pub fn get(&mut self, cpu: usize) -> &mut PerCpuCache {
        &mut self.caches[cpu % MAX_CPUS]
    }

    /// Retorna referência ao cache de uma CPU
    #[inline]
    pub fn get_ref(&self, cpu: usize) -> &PerCpuCache {
        &self.caches[cpu % MAX_CPUS]
    }

    /// Estatísticas agregadas de todos os caches
    pub fn total_stats(&self) -> (u64, u64, usize) {
        let mut total_hits = 0u64;
        let mut total_misses = 0u64;
        let mut total_cached = 0usize;

        for cache in &self.caches {
            total_hits += cache.hits;
            total_misses += cache.misses;
            total_cached += cache.count;
        }

        (total_hits, total_misses, total_cached)
    }

    /// Draina todos os caches (shutdown ou emergência)
    pub fn drain_all(&mut self, mut callback: impl FnMut(PhysAddr)) {
        for cache in &mut self.caches {
            for frame in cache.clear() {
                callback(frame);
            }
        }
    }
}

impl Default for PerCpuCaches {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: PerCpuCaches deve ser acessado apenas pela CPU correspondente
// ou com lock externo
unsafe impl Send for PerCpuCaches {}
unsafe impl Sync for PerCpuCaches {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percpu_cache_push_pop() {
        let mut cache = PerCpuCache::new();

        // Push frames
        for i in 0..10 {
            assert!(cache.push(PhysAddr::new(i * 4096)));
        }
        assert_eq!(cache.len(), 10);

        // Pop frames (LIFO)
        for i in (0..10).rev() {
            let frame = cache.pop().expect("should pop");
            assert_eq!(frame.as_u64(), i * 4096);
        }
        assert!(cache.is_empty());
    }

    #[test]
    fn test_percpu_cache_full() {
        let mut cache = PerCpuCache::new();

        // Enche o cache
        for i in 0..PERCPU_CACHE_SIZE {
            assert!(cache.push(PhysAddr::new(i as u64 * 4096)));
        }
        assert!(cache.is_full());

        // Não deve aceitar mais
        assert!(!cache.push(PhysAddr::new(0)));
    }
}
