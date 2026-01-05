//! # Gerenciamento de Memória Física
//!
//! Este módulo implementa o FrameManager unificado que gerencia todos os
//! frames físicos do sistema. Substitui o antigo PMM+PFM.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     FrameManager                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐                      │
//! │  │ Per-CPU │  │ Per-CPU │  │ Per-CPU │  Fast Path           │
//! │  │ Cache   │  │ Cache   │  │ Cache   │  (lock-free)         │
//! │  └────┬────┘  └────┬────┘  └────┬────┘                      │
//! │       │            │            │                           │
//! │       └────────────┴────────────┘                           │
//! │                    │                                        │
//! │  ┌─────────────────▼─────────────────────────────────────┐  │
//! │  │                   Chunks                              │  │
//! │  │  ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐              │  │
//! │  │  │Chunk 0│ │Chunk 1│ │Chunk 2│ │Chunk N│  Slow Path   │  │
//! │  │  │ Lock  │ │ Lock  │ │ Lock  │ │ Lock  │  (per-chunk) │  │
//! │  │  └───────┘ └───────┘ └───────┘ └───────┘              │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! │                                                             │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │                FrameInfo[] (Metadados)                │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Fast Path vs Slow Path
//!
//! - **Fast Path**: Aloca do cache per-CPU (lock-free, O(1))
//! - **Slow Path**: Refill do cache a partir dos chunks (per-chunk lock)
//!
//! ## Invariantes
//!
//! - INV-1: Se FrameInfo.owner == Free, então refcount == 0
//! - INV-2: Se refcount > 0, então owner != Free
//! - INV-3: Bitmap consistente com FrameInfo
//! - INV-4: Soma de frames por zona == total

pub mod bitmap;
pub mod chunk;
pub mod frame;
pub mod percpu;
pub mod stats;

pub use chunk::ChunkManager;
pub use frame::{FrameFlags, FrameInfo, FrameOwner};
pub use stats::PhysStats;

use crate::core::boot::BootInfo;
use crate::rmm::addr::PhysAddr;
use crate::rmm::config::*;
use crate::rmm::early::EARLY_ALLOCATOR;
use crate::rmm::error::{RmmError, RmmResult};
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;
use crate::sync::Spinlock;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use self::percpu::PerCpuCaches;

// =============================================================================
// AllocFlags
// =============================================================================

/// Flags de alocação
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct AllocFlags(u32);

impl AllocFlags {
    /// Zera o frame após alocar
    pub const ZERO: Self = Self(0x0001);
    /// Não zera (performance)
    pub const NO_ZERO: Self = Self(0x0002);
    /// Contexto atômico (não pode dormir)
    pub const ATOMIC: Self = Self(0x0004);
    /// Não espera por memória
    pub const NO_WAIT: Self = Self(0x0008);
    /// Frames contíguos
    pub const CONTIGUOUS: Self = Self(0x0010);
    /// Zona DMA
    pub const DMA: Self = Self(0x0020);
    /// Frame não pode ser swapped
    pub const PINNED: Self = Self(0x0040);
    /// Prefere high memory
    pub const HIGH: Self = Self(0x0080);
    /// Não pode falhar
    pub const NOFAIL: Self = Self(0x0100);
    /// Frame pode ser movido (compactação)
    pub const MOVABLE: Self = Self(0x0200);
    /// Frame pode ser reclamado
    pub const RECLAIMABLE: Self = Self(0x0400);

    // Combinações comuns
    /// Alocação do kernel (não zera)
    pub const KERNEL: Self = Self(0x0002);
    /// Alocação userspace (zera, movable)
    pub const USER: Self = Self(0x0201);
    /// Alocação em contexto IRQ
    pub const IRQ: Self = Self(0x000C);
    /// Buffer DMA (contíguo, pinned, DMA zone)
    pub const DMA_BUFFER: Self = Self(0x0070);

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
    pub const fn bits(&self) -> u32 {
        self.0
    }
}

// =============================================================================
// FrameManager
// =============================================================================

/// Gerenciador de frames físicos unificado
pub struct FrameManager {
    /// Metadados de cada frame (alocado via early allocator)
    frames: *mut FrameInfo,
    /// Número total de frames
    frame_count: usize,
    /// Chunks de memória (cada chunk = 2MB = 512 frames)
    chunks: Vec<ChunkManager>,
    /// Número de chunks
    chunk_count: usize,
    /// Base física do primeiro frame gerenciado
    base_phys: PhysAddr,
    /// Caches per-CPU para fast path
    percpu_caches: PerCpuCaches,
    /// Estatísticas
    stats: PhysStats,
    /// Limites das zonas
    zone_dma_end: usize, // Índice do último frame da zona DMA
    zone_dma32_end: usize, // Índice do último frame da zona DMA32
}

/// Instância global do FrameManager
static FRAME_MANAGER: Spinlock<Option<FrameManager>> = Spinlock::new(None);

/// Contador de frames livres (para fast check)
static FREE_FRAMES: AtomicUsize = AtomicUsize::new(0);

// =============================================================================
// Inicialização
// =============================================================================

/// Inicializa o gerenciador de frames físicos
pub unsafe fn init(boot_info: &'static BootInfo) {
    crate::kinfo!("(RMM/Phys) Inicializando FrameManager...");

    // 1. Calcular memória total e número de frames
    let total_memory = boot_info.total_memory;
    let frame_count = total_memory as usize / PAGE_SIZE;
    let chunk_count = (frame_count + FRAMES_PER_CHUNK - 1) / FRAMES_PER_CHUNK;

    crate::kinfo!(
        "(RMM/Phys) RAM: {} MB, Frames: {}, Chunks: {}",
        total_memory / (1024 * 1024),
        frame_count,
        chunk_count
    );

    // 2. Alocar array de FrameInfo via early allocator
    let frames_size = frame_count * core::mem::size_of::<FrameInfo>();
    let frames_ptr = {
        let mut early = EARLY_ALLOCATOR.lock();
        if let Some(allocator) = early.as_mut() {
            allocator.alloc(frames_size, core::mem::align_of::<FrameInfo>())
        } else {
            panic!("Early allocator not initialized!");
        }
    };

    if frames_ptr.is_null() {
        panic!("Failed to allocate FrameInfo array!");
    }

    // Converter para ponteiro virtual via HHDM
    let frames_virt = hhdm::phys_to_virt(frames_ptr as u64) as *mut FrameInfo;

    crate::kinfo!(
        "(RMM/Phys) FrameInfo array: {} KB @ 0x{:x}",
        frames_size / 1024,
        frames_virt as u64
    );

    // 3. Inicializar todos os FrameInfo como Free
    for i in 0..frame_count {
        let frame = frames_virt.add(i);
        core::ptr::write(frame, FrameInfo::new());
    }

    // 4. Criar chunks
    let mut chunks = Vec::with_capacity(chunk_count);
    for i in 0..chunk_count {
        let base = PhysAddr::new((i * CHUNK_SIZE) as u64);
        let migrate_type = crate::rmm::zone::MigrateType::Movable;
        chunks.push(ChunkManager::new(base, migrate_type));
    }

    // 5. Calcular limites das zonas
    let zone_dma_end = (ZONE_DMA_END as usize / PAGE_SIZE).min(frame_count);
    let zone_dma32_end = (ZONE_DMA32_END as usize / PAGE_SIZE).min(frame_count);

    crate::kinfo!(
        "(RMM/Phys) Zonas: DMA 0-{}, DMA32 {}-{}, Normal {}-{}",
        zone_dma_end,
        zone_dma_end,
        zone_dma32_end,
        zone_dma32_end,
        frame_count
    );

    // 6. Marcar regiões reservadas do BootInfo
    let mut reserved_frames = 0usize;
    for region in boot_info.memory_regions.iter() {
        if !region.is_usable() {
            let start_frame = region.start as usize / PAGE_SIZE;
            let end_frame = (region.start as usize + region.size) / PAGE_SIZE;
            for f in start_frame..end_frame.min(frame_count) {
                let frame_info = &mut *frames_virt.add(f);
                frame_info.set_owner(FrameOwner::Kernel);
                reserved_frames += 1;
            }
        }
    }

    let free_frames = frame_count - reserved_frames;
    FREE_FRAMES.store(free_frames, Ordering::Release);

    crate::kinfo!(
        "(RMM/Phys) Frames: {} total, {} reservados, {} livres",
        frame_count,
        reserved_frames,
        free_frames
    );

    // 7. Criar FrameManager
    let fm = FrameManager {
        frames: frames_virt,
        frame_count,
        chunks,
        chunk_count,
        base_phys: PhysAddr::new(0),
        percpu_caches: PerCpuCaches::new(),
        stats: PhysStats::new(),
        zone_dma_end,
        zone_dma32_end,
    };

    // 8. Instalar globalmente
    *FRAME_MANAGER.lock() = Some(fm);

    crate::kinfo!("(RMM/Phys) FrameManager inicializado com sucesso!");
}

// =============================================================================
// API Pública
// =============================================================================

/// Aloca um frame físico
pub fn alloc(owner: FrameOwner, zone: Zone, flags: AllocFlags) -> Option<PhysAddr> {
    let mut guard = FRAME_MANAGER.lock();
    let fm = guard.as_mut()?;

    // Fast path: tentar cache per-CPU
    let cpu_id = current_cpu_id();
    if let Some(frame) = fm.percpu_caches.get(cpu_id).pop() {
        // Verificar se está na zona correta
        let frame_idx = (frame.as_u64() as usize) / PAGE_SIZE;
        if fm.is_in_zone(frame_idx, zone) {
            fm.setup_frame(frame_idx, owner, flags);
            fm.stats.record_alloc(true);
            return Some(frame);
        }
        // Devolver ao cache se zona errada
        fm.percpu_caches.get(cpu_id).push(frame);
    }

    // Slow path: alocar de chunk
    let frame = fm.alloc_from_zone(zone, flags)?;
    let frame_idx = (frame.as_u64() as usize) / PAGE_SIZE;
    fm.setup_frame(frame_idx, owner, flags);
    fm.stats.record_alloc(false);

    FREE_FRAMES.fetch_sub(1, Ordering::Relaxed);

    Some(frame)
}

/// Aloca frames contíguos
pub fn alloc_contiguous(
    count: usize,
    owner: FrameOwner,
    zone: Zone,
    flags: AllocFlags,
) -> Option<PhysAddr> {
    if count == 0 {
        return None;
    }
    if count == 1 {
        return alloc(owner, zone, flags);
    }

    let mut guard = FRAME_MANAGER.lock();
    let fm = guard.as_mut()?;

    // Encontrar range contíguo
    let (start_idx, end_idx) = fm.zone_range(zone);
    let start_frame = fm.find_contiguous_run(start_idx, end_idx, count)?;

    // Marcar todos como alocados
    for i in 0..count {
        fm.setup_frame(start_frame + i, owner, flags);
    }

    fm.stats
        .alloc_count
        .fetch_add(count as u64, Ordering::Relaxed);
    FREE_FRAMES.fetch_sub(count, Ordering::Relaxed);

    Some(PhysAddr::new((start_frame * PAGE_SIZE) as u64))
}

/// Libera um frame físico
pub fn free(phys: PhysAddr, expected_owner: FrameOwner) -> RmmResult<()> {
    let mut guard = FRAME_MANAGER.lock();
    let fm = guard.as_mut().ok_or(RmmError::NotInitialized)?;

    let frame_idx = (phys.as_u64() as usize) / PAGE_SIZE;
    if frame_idx >= fm.frame_count {
        return Err(RmmError::InvalidAddress);
    }

    let frame_info = unsafe { &mut *fm.frames.add(frame_idx) };

    // Verificar owner
    let current_owner = frame_info.owner();
    if current_owner != expected_owner {
        #[cfg(debug_assertions)]
        panic!(
            "Owner mismatch: expected {:?}, got {:?}",
            expected_owner, current_owner
        );
        #[cfg(not(debug_assertions))]
        return Err(RmmError::OwnerMismatch);
    }

    // Verificar refcount
    let refcount = frame_info.dec_ref();
    if refcount > 0 {
        // Ainda tem referências, não libera
        return Ok(());
    }

    // Limpar frame
    frame_info.set_owner(FrameOwner::Free);
    frame_info.set_flags(FrameFlags::NONE);

    // Fast path: adicionar ao cache per-CPU
    let cpu_id = current_cpu_id();
    if !fm.percpu_caches.get(cpu_id).push(phys) {
        // Cache cheio, marcar como livre no chunk
        let chunk_idx = frame_idx / FRAMES_PER_CHUNK;
        if chunk_idx < fm.chunk_count {
            fm.chunks[chunk_idx].free(phys);
        }
    }

    fm.stats.record_free();
    FREE_FRAMES.fetch_add(1, Ordering::Relaxed);

    Ok(())
}

/// Incrementa reference count de um frame
pub fn inc_ref(phys: PhysAddr) -> RmmResult<u32> {
    let guard = FRAME_MANAGER.lock();
    let fm = guard.as_ref().ok_or(RmmError::NotInitialized)?;

    let frame_idx = (phys.as_u64() as usize) / PAGE_SIZE;
    if frame_idx >= fm.frame_count {
        return Err(RmmError::InvalidAddress);
    }

    let frame_info = unsafe { &*fm.frames.add(frame_idx) };
    Ok(frame_info.inc_ref())
}

/// Decrementa reference count de um frame
pub fn dec_ref(phys: PhysAddr) -> RmmResult<u32> {
    let guard = FRAME_MANAGER.lock();
    let fm = guard.as_ref().ok_or(RmmError::NotInitialized)?;

    let frame_idx = (phys.as_u64() as usize) / PAGE_SIZE;
    if frame_idx >= fm.frame_count {
        return Err(RmmError::InvalidAddress);
    }

    let frame_info = unsafe { &*fm.frames.add(frame_idx) };
    Ok(frame_info.dec_ref())
}

/// Retorna o owner de um frame
pub fn get_owner(phys: PhysAddr) -> Option<FrameOwner> {
    let guard = FRAME_MANAGER.lock();
    let fm = guard.as_ref()?;

    let frame_idx = (phys.as_u64() as usize) / PAGE_SIZE;
    if frame_idx >= fm.frame_count {
        return None;
    }

    let frame_info = unsafe { &*fm.frames.add(frame_idx) };
    Some(frame_info.owner())
}

/// Retorna número de frames livres
pub fn free_count() -> usize {
    FREE_FRAMES.load(Ordering::Relaxed)
}

/// Tenta alocar sem bloquear (para contexto IRQ)
pub fn try_alloc(owner: FrameOwner, zone: Zone, flags: AllocFlags) -> Option<PhysAddr> {
    let mut guard = match FRAME_MANAGER.try_lock() {
        Some(g) => g,
        None => return None,
    };

    let fm = guard.as_mut()?;
    let cpu_id = current_cpu_id();

    // Apenas fast path
    if let Some(frame) = fm.percpu_caches.get(cpu_id).pop() {
        let frame_idx = (frame.as_u64() as usize) / PAGE_SIZE;
        if fm.is_in_zone(frame_idx, zone) {
            fm.setup_frame(frame_idx, owner, flags);
            fm.stats.record_alloc(true);
            FREE_FRAMES.fetch_sub(1, Ordering::Relaxed);
            return Some(frame);
        }
        fm.percpu_caches.get(cpu_id).push(frame);
    }

    None
}

// =============================================================================
// Métodos Internos do FrameManager
// =============================================================================

impl FrameManager {
    /// Aloca de uma zona específica
    fn alloc_from_zone(&mut self, zone: Zone, _flags: AllocFlags) -> Option<PhysAddr> {
        let (start_chunk, end_chunk) = self.zone_chunk_range(zone);

        // Lock ordering: sempre ascendente
        for chunk_idx in start_chunk..end_chunk {
            if let Some(frame) = self.chunks[chunk_idx].alloc() {
                return Some(frame);
            }
        }

        // Fallback: tentar outras zonas se permitido
        // Normal pode cair para DMA32, DMA32 pode cair para DMA
        match zone {
            Zone::Normal => self.alloc_from_zone(Zone::Dma32, _flags),
            Zone::Dma32 => self.alloc_from_zone(Zone::Dma, _flags),
            Zone::Dma => None,
        }
    }

    /// Encontra run contíguo de frames livres
    fn find_contiguous_run(&self, start_idx: usize, end_idx: usize, count: usize) -> Option<usize> {
        let mut run_start = start_idx;
        let mut run_len = 0;

        for idx in start_idx..end_idx {
            let frame_info = unsafe { &*self.frames.add(idx) };
            if frame_info.owner() == FrameOwner::Free {
                if run_len == 0 {
                    run_start = idx;
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

    /// Configura um frame após alocação
    fn setup_frame(&mut self, frame_idx: usize, owner: FrameOwner, flags: AllocFlags) {
        let frame_info = unsafe { &mut *self.frames.add(frame_idx) };
        frame_info.set_owner(owner);
        frame_info.inc_ref();

        if flags.contains(AllocFlags::ZERO) || DEBUG_ZERO_ON_ALLOC {
            let phys = (frame_idx * PAGE_SIZE) as u64;
            unsafe {
                hhdm::zero_page(PhysAddr::new(phys));
            }
        }
    }

    /// Verifica se frame está na zona
    fn is_in_zone(&self, frame_idx: usize, zone: Zone) -> bool {
        match zone {
            Zone::Dma => frame_idx < self.zone_dma_end,
            Zone::Dma32 => frame_idx < self.zone_dma32_end,
            Zone::Normal => true,
        }
    }

    /// Retorna range de índices de frame para uma zona
    fn zone_range(&self, zone: Zone) -> (usize, usize) {
        match zone {
            Zone::Dma => (0, self.zone_dma_end),
            Zone::Dma32 => (self.zone_dma_end, self.zone_dma32_end),
            Zone::Normal => (self.zone_dma32_end, self.frame_count),
        }
    }

    /// Retorna range de chunks para uma zona
    fn zone_chunk_range(&self, zone: Zone) -> (usize, usize) {
        let (start_frame, end_frame) = self.zone_range(zone);
        let start_chunk = start_frame / FRAMES_PER_CHUNK;
        let end_chunk = (end_frame + FRAMES_PER_CHUNK - 1) / FRAMES_PER_CHUNK;
        (start_chunk, end_chunk.min(self.chunk_count))
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Retorna CPU ID atual (stub - implementar com arch-specific code)
#[inline]
fn current_cpu_id() -> usize {
    // TODO: Implementar leitura real do APIC ID ou similar
    0
}
