//! # FrameInfo - Metadados de Frame Físico
//!
//! Cada frame físico tem um FrameInfo associado que armazena:
//! - Owner (quem possui o frame)
//! - Reference count (para sharing/CoW)
//! - Flags (DIRTY, ACCESSED, LOCKED)
//! - Rmap (reverse mapping para PTEs)
//!
//! ## Concorrência e Rmap Overflow
//!
//! O rmap_overflow usa AtomicPtr para lista encadeada dinamicamente alocada.
//!
//! **CRÍTICO TODO** (Próxima etapa): Implementar scheme de reclamation seguro
//! para nós de rmap_overflow. Opções:
//! - Hazard Pointers: Proteção de ponteiros em uso
//! - Epoch-based Reclamation: Similar ao crossbeam-epoch
//! - RCU (Read-Copy-Update): Para leitura frequente
//!
//! Por enquanto, nós são alocados mas nunca liberados (leak intencional para safety).
//! Isso é aceitável para V1 pois shared pages são raras no início.

use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};

/// Owner de um frame físico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameOwner {
    /// Livre para alocação
    Free,
    /// Kernel core
    Kernel,
    /// Processo userspace
    Process { pid: u32 },
    /// Driver de dispositivo
    Driver { id: u32 },
    /// Compartilhado (CoW, mmap shared)
    Shared,
    /// Hardware (framebuffer, MMIO)
    Device,
    /// Não pode ser swapped
    Pinned { owner: u32 },
}

impl FrameOwner {
    /// Empacota owner em u64
    pub fn pack(&self) -> u64 {
        match self {
            Self::Free => 0,
            Self::Kernel => 1,
            Self::Process { pid } => 2 | ((*pid as u64) << 8),
            Self::Driver { id } => 3 | ((*id as u64) << 8),
            Self::Shared => 4,
            Self::Device => 5,
            Self::Pinned { owner } => 6 | ((*owner as u64) << 8),
        }
    }

    /// Desempacota de u64
    pub fn unpack(val: u64) -> Self {
        match val & 0xFF {
            0 => Self::Free,
            1 => Self::Kernel,
            2 => Self::Process {
                pid: (val >> 8) as u32,
            },
            3 => Self::Driver {
                id: (val >> 8) as u32,
            },
            4 => Self::Shared,
            5 => Self::Device,
            6 => Self::Pinned {
                owner: (val >> 8) as u32,
            },
            _ => Self::Free,
        }
    }
}

/// Flags do frame
#[derive(Clone, Copy)]
pub struct FrameFlags(u32);

impl FrameFlags {
    pub const NONE: Self = Self(0);
    pub const DIRTY: Self = Self(1 << 0);
    pub const ACCESSED: Self = Self(1 << 1);
    pub const LOCKED: Self = Self(1 << 2);
    pub const MOVING: Self = Self(1 << 3);

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Nó de overflow para rmap
pub struct RmapNode {
    pub pte: u64,
    pub next: *mut RmapNode,
}

/// Metadados de um frame físico
#[repr(C)]
pub struct FrameInfo {
    /// Owner empacotado
    state: AtomicU64,
    /// Reference count
    ref_count: AtomicU32,
    /// Flags
    flags: AtomicU32,
    /// Rmap inline (2 slots)
    rmap_inline: [AtomicU64; 2],
    /// Overflow pointer
    rmap_overflow: AtomicPtr<RmapNode>,
}

impl FrameInfo {
    pub const fn new() -> Self {
        Self {
            state: AtomicU64::new(0),
            ref_count: AtomicU32::new(0),
            flags: AtomicU32::new(0),
            rmap_inline: [AtomicU64::new(0), AtomicU64::new(0)],
            rmap_overflow: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn owner(&self) -> FrameOwner {
        FrameOwner::unpack(self.state.load(Ordering::Acquire))
    }

    pub fn set_owner(&self, owner: FrameOwner) {
        self.state.store(owner.pack(), Ordering::Release);
    }

    pub fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::Acquire)
    }

    pub fn inc_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn dec_ref(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::AcqRel) - 1
    }

    pub fn flags(&self) -> FrameFlags {
        FrameFlags(self.flags.load(Ordering::Acquire))
    }

    pub fn set_flags(&self, flags: FrameFlags) {
        self.flags.store(flags.0, Ordering::Release);
    }

    /// Adiciona PTE ao rmap
    pub fn rmap_add(&self, pte: u64) {
        // Tenta slot inline 0
        if self.rmap_inline[0]
            .compare_exchange(0, pte, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return;
        }
        // Tenta slot inline 1
        if self.rmap_inline[1]
            .compare_exchange(0, pte, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return;
        }
        // TODO: Overflow list
    }

    /// Remove PTE do rmap
    pub fn rmap_remove(&self, pte: u64) {
        if self.rmap_inline[0]
            .compare_exchange(pte, 0, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return;
        }
        if self.rmap_inline[1]
            .compare_exchange(pte, 0, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return;
        }
        // TODO: Overflow list
    }
}
