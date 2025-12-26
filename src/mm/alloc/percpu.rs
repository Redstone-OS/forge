//! # Per-CPU Object Caches
//!
//! Caches locais por CPU para reduzir contenção no allocator global.
//!
//! ## 🎯 Propósito
//!
//! Em sistemas multicore, um único lock global para alocação cria
//! contenção severa. Per-CPU caches permitem alocações sem lock
//! na maioria dos casos.
//!
//! ## 🏗️ Arquitetura
//!
//! Cada CPU mantém um cache LIFO de objetos previamente liberados
//! para cada size-class do Slab. Fluxo:
//!
//! 1. alloc() → tenta cache local (sem lock!)
//! 2. Se vazio → busca do Slab global (com lock)
//! 3. dealloc() → devolve ao cache local
//! 4. Se cheio → devolve ao Slab global
//!
//! ## Performance
//!
//! - Fast path (cache hit): ~10-20 ciclos
//! - Slow path (global): ~100-500 ciclos
//! - Hit rate típico: 90%+

use crate::mm::config::{CACHE_LINE_SIZE, MAX_CPUS};
use core::sync::atomic::{AtomicUsize, Ordering};

// =============================================================================
// CONFIGURAÇÃO
// =============================================================================

/// Número de objetos no cache por size-class
pub const PERCPU_CACHE_SIZE: usize = 32;

/// Número de size-classes suportadas (16, 32, 64, ..., 2048)
pub const NUM_SIZE_CLASSES: usize = 8;

/// Threshold para refill do cache local
pub const REFILL_THRESHOLD: usize = PERCPU_CACHE_SIZE / 4;

/// Batch size para refill/flush
pub const BATCH_SIZE: usize = PERCPU_CACHE_SIZE / 2;

// =============================================================================
// CACHE POR CPU
// =============================================================================

/// Cache local de objetos para uma size-class
///
/// Alinhado a cache line para evitar false sharing entre CPUs.
#[repr(C, align(64))]
pub struct PerCpuCache {
    /// Slots do cache (LIFO stack)
    slots: [*mut u8; PERCPU_CACHE_SIZE],
    /// Número de slots ocupados
    count: usize,
    /// Estatísticas
    hits: u64,
    misses: u64,
    flushes: u64,
}

// Safety: Cada CPU acessa apenas seu próprio cache
unsafe impl Send for PerCpuCache {}
unsafe impl Sync for PerCpuCache {}

impl PerCpuCache {
    /// Cria cache vazio
    pub const fn new() -> Self {
        Self {
            slots: [core::ptr::null_mut(); PERCPU_CACHE_SIZE],
            count: 0,
            hits: 0,
            misses: 0,
            flushes: 0,
        }
    }

    /// Tenta obter objeto do cache local (sem lock!)
    ///
    /// Retorna None se cache vazio.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<*mut u8> {
        if self.count > 0 {
            self.count -= 1;
            self.hits += 1;
            Some(self.slots[self.count])
        } else {
            self.misses += 1;
            None
        }
    }

    /// Tenta devolver objeto ao cache local
    ///
    /// Retorna false se cache cheio (deve ir pro global).
    #[inline(always)]
    pub fn push(&mut self, ptr: *mut u8) -> bool {
        if self.count < PERCPU_CACHE_SIZE {
            self.slots[self.count] = ptr;
            self.count += 1;
            true
        } else {
            false
        }
    }

    /// Verifica se precisa de refill
    #[inline]
    pub fn needs_refill(&self) -> bool {
        self.count < REFILL_THRESHOLD
    }

    /// Verifica se precisa de flush
    #[inline]
    pub fn needs_flush(&self) -> bool {
        self.count >= PERCPU_CACHE_SIZE - 2
    }

    /// Retorna contagem atual
    pub fn len(&self) -> usize {
        self.count
    }

    /// Retorna estatísticas
    pub fn stats(&self) -> (u64, u64, u64) {
        (self.hits, self.misses, self.flushes)
    }

    /// Flush: remove batch de objetos do cache
    ///
    /// Retorna vetor de ponteiros para devolver ao global.
    pub fn flush_batch(&mut self, batch: &mut [*mut u8; BATCH_SIZE]) -> usize {
        let to_flush = core::cmp::min(BATCH_SIZE, self.count);

        for i in 0..to_flush {
            self.count -= 1;
            batch[i] = self.slots[self.count];
        }

        self.flushes += 1;
        to_flush
    }

    /// Refill: adiciona batch de objetos ao cache
    pub fn refill_batch(&mut self, batch: &[*mut u8], count: usize) {
        for i in 0..count {
            if self.count >= PERCPU_CACHE_SIZE {
                break;
            }
            self.slots[self.count] = batch[i];
            self.count += 1;
        }
    }
}

// =============================================================================
// GERENCIADOR DE CACHES
// =============================================================================

/// Gerenciador global de caches per-CPU
///
/// Mantém caches para todas as CPUs e todas as size-classes.
pub struct PerCpuAllocator {
    /// Caches indexados por [cpu_id][size_class]
    caches: [[PerCpuCache; NUM_SIZE_CLASSES]; MAX_CPUS],
    /// Indica se foi inicializado
    initialized: bool,
}

// Safety: Cada CPU acessa apenas seus próprios caches
unsafe impl Send for PerCpuAllocator {}
unsafe impl Sync for PerCpuAllocator {}

impl PerCpuAllocator {
    /// Cria gerenciador vazio
    pub const fn new() -> Self {
        const EMPTY_CACHE: PerCpuCache = PerCpuCache::new();
        const EMPTY_ROW: [PerCpuCache; NUM_SIZE_CLASSES] = [EMPTY_CACHE; NUM_SIZE_CLASSES];

        Self {
            caches: [EMPTY_ROW; MAX_CPUS],
            initialized: false,
        }
    }

    /// Inicializa o allocator
    pub fn init(&mut self) {
        self.initialized = true;
        crate::kinfo!(
            "(PerCPU) Inicializado: {} CPUs x {} size-classes",
            MAX_CPUS,
            NUM_SIZE_CLASSES
        );
    }

    /// Aloca objeto do cache local
    ///
    /// # Safety
    /// Deve ser chamado com interrupts desabilitadas para evitar
    /// preemption entre obter CPU ID e acessar cache.
    #[inline(always)]
    pub unsafe fn alloc(&mut self, size_class: usize) -> Option<*mut u8> {
        if !self.initialized || size_class >= NUM_SIZE_CLASSES {
            return None;
        }

        let cpu = get_cpu_id();
        if cpu >= MAX_CPUS {
            return None;
        }

        self.caches[cpu][size_class].pop()
    }

    /// Libera objeto para cache local
    ///
    /// Retorna false se cache cheio (deve ir pro global).
    #[inline(always)]
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, size_class: usize) -> bool {
        if !self.initialized || size_class >= NUM_SIZE_CLASSES {
            return false;
        }

        let cpu = get_cpu_id();
        if cpu >= MAX_CPUS {
            return false;
        }

        self.caches[cpu][size_class].push(ptr)
    }

    /// Obtém cache para CPU e size-class específicos
    pub fn get_cache(&mut self, cpu: usize, size_class: usize) -> Option<&mut PerCpuCache> {
        if cpu < MAX_CPUS && size_class < NUM_SIZE_CLASSES {
            Some(&mut self.caches[cpu][size_class])
        } else {
            None
        }
    }

    /// Imprime estatísticas de todos os caches
    pub fn print_stats(&self) {
        crate::kinfo!("╔═══════════════════════════════════════════╗");
        crate::kinfo!("║         ESTATÍSTICAS PER-CPU              ║");
        crate::kinfo!("╚═══════════════════════════════════════════╝");

        for cpu in 0..MAX_CPUS {
            let mut total_hits = 0u64;
            let mut total_misses = 0u64;

            for sc in 0..NUM_SIZE_CLASSES {
                let (hits, misses, _) = self.caches[cpu][sc].stats();
                total_hits += hits;
                total_misses += misses;
            }

            if total_hits + total_misses > 0 {
                let hit_rate = (total_hits * 100) / (total_hits + total_misses);
                crate::kinfo!(
                    "  CPU {}: {} hits, {} misses ({}%)",
                    cpu,
                    total_hits,
                    total_misses,
                    hit_rate
                );
            }
        }
    }
}

/// Allocator Per-CPU global
#[cfg(feature = "percpu_caches")]
pub static mut PERCPU_ALLOCATOR: PerCpuAllocator = PerCpuAllocator::new();

// =============================================================================
// CONVERSÃO SIZE → SIZE-CLASS
// =============================================================================

/// Converte tamanho em bytes para índice de size-class
///
/// Size classes: 16, 32, 64, 128, 256, 512, 1024, 2048
#[inline(always)]
pub fn size_to_class(size: usize) -> usize {
    match size {
        0..=16 => 0,
        17..=32 => 1,
        33..=64 => 2,
        65..=128 => 3,
        129..=256 => 4,
        257..=512 => 5,
        513..=1024 => 6,
        1025..=2048 => 7,
        _ => NUM_SIZE_CLASSES, // Inválido para per-cpu
    }
}

/// Retorna tamanho da size-class
#[inline(always)]
pub fn class_to_size(class: usize) -> usize {
    match class {
        0 => 16,
        1 => 32,
        2 => 64,
        3 => 128,
        4 => 256,
        5 => 512,
        6 => 1024,
        7 => 2048,
        _ => 0,
    }
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Obtém ID do CPU atual
///
/// TODO: Implementar corretamente via APIC ou per-CPU variable
#[inline(always)]
fn get_cpu_id() -> usize {
    // Placeholder - deve usar GS segment ou APIC ID
    0
}

// =============================================================================
// TESTES
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_push_pop() {
        let mut cache = PerCpuCache::new();

        let ptr = 0x1000 as *mut u8;
        assert!(cache.push(ptr));
        assert_eq!(cache.len(), 1);

        let popped = cache.pop();
        assert_eq!(popped, Some(ptr));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_full() {
        let mut cache = PerCpuCache::new();

        for i in 0..PERCPU_CACHE_SIZE {
            assert!(cache.push((i * 0x1000) as *mut u8));
        }

        // Cache cheio
        assert!(!cache.push(0xDEAD as *mut u8));
    }

    #[test]
    fn test_size_to_class() {
        assert_eq!(size_to_class(16), 0);
        assert_eq!(size_to_class(32), 1);
        assert_eq!(size_to_class(64), 2);
        assert_eq!(size_to_class(2048), 7);
        assert_eq!(size_to_class(4096), NUM_SIZE_CLASSES);
    }
}
