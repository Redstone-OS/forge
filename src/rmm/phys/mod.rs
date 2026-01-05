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
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐                     │
//! │  │ Per-CPU │  │ Per-CPU │  │ Per-CPU │  Fast Path           │
//! │  │ Cache   │  │ Cache   │  │ Cache   │  (lock-free)         │
//! │  └────┬────┘  └────┬────┘  └────┬────┘                     │
//! │       │            │            │                           │
//! │       └────────────┴────────────┘                           │
//! │                    │                                        │
//! │  ┌─────────────────▼─────────────────────────────────────┐ │
//! │  │                   Chunks                               │ │
//! │  │  ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐              │ │
//! │  │  │Chunk 0│ │Chunk 1│ │Chunk 2│ │Chunk N│  Slow Path   │ │
//! │  │  │ Lock  │ │ Lock  │ │ Lock  │ │ Lock  │  (per-chunk) │ │
//! │  │  └───────┘ └───────┘ └───────┘ └───────┘              │ │
//! │  └───────────────────────────────────────────────────────┘ │
//! │                                                             │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │              FrameInfo[] (Metadados)                 │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```

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
use crate::rmm::error::{RmmError, RmmResult};
use crate::rmm::zone::Zone;
use crate::sync::Spinlock;

use alloc::vec::Vec;

/// Flags de alocação
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AllocFlags(u32);

impl AllocFlags {
    pub const ZERO: Self = Self(0x0001);
    pub const NO_ZERO: Self = Self(0x0002);
    pub const ATOMIC: Self = Self(0x0004);
    pub const NO_WAIT: Self = Self(0x0008);
    pub const CONTIGUOUS: Self = Self(0x0010);
    pub const DMA: Self = Self(0x0020);
    pub const PINNED: Self = Self(0x0040);
    pub const HIGH: Self = Self(0x0080);
    pub const NOFAIL: Self = Self(0x0100);
    pub const MOVABLE: Self = Self(0x0200);
    pub const RECLAIMABLE: Self = Self(0x0400);

    pub const KERNEL: Self = Self(0x0002);
    pub const USER: Self = Self(0x0201);
    pub const IRQ: Self = Self(0x000C);
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
}

/// Gerenciador de frames físicos unificado
pub struct FrameManager {
    /// Metadados de cada frame
    frames: &'static mut [FrameInfo],
    /// Chunks de memória
    chunks: Vec<ChunkManager>,
    /// Base física do primeiro frame
    base_phys: PhysAddr,
    /// Total de frames
    frame_count: usize,
    /// Estatísticas
    stats: PhysStats,
}

/// Instância global do FrameManager
static FRAME_MANAGER: Spinlock<Option<FrameManager>> = Spinlock::new(None);

/// Inicializa o gerenciador de frames físicos
pub unsafe fn init(boot_info: &'static BootInfo) {
    crate::kinfo!("(RMM/Phys) Inicializando FrameManager...");

    // TODO: Implementar inicialização completa
    // 1. Alocar FrameInfo[] via early allocator
    // 2. Inicializar chunks
    // 3. Marcar regiões reservadas
    // 4. Configurar per-CPU caches

    crate::kinfo!("(RMM/Phys) FrameManager inicializado (stub)");
}

/// Aloca um frame físico
pub fn alloc(owner: FrameOwner, zone: Zone, flags: AllocFlags) -> Option<PhysAddr> {
    let mut guard = FRAME_MANAGER.lock();
    let fm = guard.as_mut()?;

    // TODO: Implementar alocação completa
    // 1. Tentar per-CPU cache
    // 2. Tentar chunk na zona
    // 3. Tentar outras zonas se permitido

    None
}

/// Aloca frames contíguos
pub fn alloc_contiguous(
    count: usize,
    owner: FrameOwner,
    zone: Zone,
    flags: AllocFlags,
) -> Option<PhysAddr> {
    // TODO: Implementar alocação contígua
    None
}

/// Libera um frame físico
pub fn free(phys: PhysAddr, expected_owner: FrameOwner) -> RmmResult<()> {
    // TODO: Implementar liberação
    Ok(())
}

/// Incrementa reference count
pub fn inc_ref(phys: PhysAddr) -> RmmResult<u32> {
    // TODO: Implementar
    Ok(1)
}

/// Decrementa reference count
pub fn dec_ref(phys: PhysAddr) -> RmmResult<u32> {
    // TODO: Implementar
    Ok(0)
}

/// Retorna o owner de um frame
pub fn get_owner(phys: PhysAddr) -> Option<FrameOwner> {
    // TODO: Implementar
    None
}
