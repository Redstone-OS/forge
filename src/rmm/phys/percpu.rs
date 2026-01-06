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
//! ┌───────────────────────────────────────────┐
//! │                 CPU 0                     │
//! │  ┌─────────────────────────────────────┐  │
//! │  │ frames[0..31]  │  count: 15         │  │
//! │  └─────────────────────────────────────┘  │
//! └───────────────────────────────────────────┘
//!                      ↓ pop/push
//! ┌───────────────────────────────────────────┐
//! │                 Chunks                    │
//! │  (slow path com lock quando cache vazio)  │
//! └───────────────────────────────────────────┘
//! ```
//!
//! ## Alocação Dinâmica
//!
//! A partir da v2, PerCpuCaches é alocado dinamicamente baseado no
//! número real de CPUs detectadas via ACPI, não mais um array fixo.

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PERCPU_CACHE_SIZE;

/// Número máximo de CPUs suportadas (limite de segurança)
pub const MAX_SUPPORTED_CPUS: usize = 256;

// =============================================================================
// PerCpuCache - Cache individual de uma CPU
// =============================================================================

/// Cache local de uma CPU
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
    #[inline]
    pub fn push(&mut self, frame: PhysAddr) -> bool {
        if self.count >= PERCPU_CACHE_SIZE {
            return false;
        }
        self.frames[self.count] = frame;
        self.count += 1;
        if self.count > self.high_watermark {
            self.high_watermark = self.count;
        }
        true
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

    /// Quantidade de frames em cache
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Taxa de hit (0-100)
    pub fn hit_rate_pct(&self) -> u64 {
        let total = self.hits + self.misses;
        if total == 0 {
            100
        } else {
            (self.hits * 100) / total
        }
    }

    /// Número de hits
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Número de misses
    pub fn misses(&self) -> u64 {
        self.misses
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
// PerCpuCaches - Gerenciador de caches para todas as CPUs
// =============================================================================

/// Gerenciador de caches per-CPU (alocação dinâmica via early allocator)
pub struct PerCpuCaches {
    /// Slice de caches alocado dinamicamente
    caches: Option<&'static mut [PerCpuCache]>,
}

// Safety: cada CPU acessa apenas seu próprio cache
unsafe impl Send for PerCpuCaches {}
unsafe impl Sync for PerCpuCaches {}

impl PerCpuCaches {
    /// Cria gerenciador vazio
    pub fn empty() -> Self {
        Self { caches: None }
    }

    /// Cria para single-core (boot inicial)
    pub fn new_single() -> Self {
        let slice = crate::rmm::early::alloc_slice::<PerCpuCache>(1);

        // Inicializa o cache
        slice[0] = PerCpuCache::new();

        Self {
            caches: Some(slice),
        }
    }

    /// Cria para N CPUs (após ACPI detectar quantidade real)
    ///
    /// # Safety
    ///
    /// O early allocator deve estar ativo.
    pub unsafe fn new_for_cpus(cpu_count: usize) -> Self {
        let count = cpu_count.min(MAX_SUPPORTED_CPUS);
        if count == 0 {
            return Self::empty();
        }

        let slice = crate::rmm::early::alloc_slice::<PerCpuCache>(count);

        // Inicializa cada cache
        for cache in slice.iter_mut() {
            *cache = PerCpuCache::new();
        }

        crate::kdebug!("(PerCpuCaches) Alocado para", count as u64, "CPUs");

        Self {
            caches: Some(slice),
        }
    }

    /// Verifica se está inicializado
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.caches.is_some()
    }

    /// Retorna referência mutável ao cache de uma CPU
    #[inline]
    pub fn get(&mut self, cpu: usize) -> Option<&mut PerCpuCache> {
        self.caches.as_mut().and_then(|s| s.get_mut(cpu))
    }

    /// Retorna referência ao cache de uma CPU
    #[inline]
    pub fn get_ref(&self, cpu: usize) -> Option<&PerCpuCache> {
        self.caches.as_ref().and_then(|s| s.get(cpu))
    }

    /// Número de CPUs
    #[inline]
    pub fn cpu_count(&self) -> usize {
        self.caches.as_ref().map_or(0, |s| s.len())
    }

    /// Estatísticas agregadas de todos os caches
    pub fn total_stats(&self) -> (u64, u64, usize) {
        let mut total_hits = 0u64;
        let mut total_misses = 0u64;
        let mut total_cached = 0usize;

        if let Some(caches) = &self.caches {
            for cache in caches.iter() {
                total_hits += cache.hits;
                total_misses += cache.misses;
                total_cached += cache.count;
            }
        }

        (total_hits, total_misses, total_cached)
    }

    /// Taxa de hit agregada (0-100)
    pub fn hit_rate_pct(&self) -> u64 {
        let (hits, misses, _) = self.total_stats();
        let total = hits + misses;
        if total == 0 {
            100
        } else {
            (hits * 100) / total
        }
    }

    /// Draina todos os caches
    pub fn drain_all<F: FnMut(PhysAddr)>(&mut self, mut callback: F) {
        if let Some(caches) = &mut self.caches {
            for cache in caches.iter_mut() {
                for frame in cache.clear() {
                    callback(frame);
                }
            }
        }
    }
}
