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
//! Para V1: rmap usa apenas os 2 slots inline. Se overflow, os mappings
//! extras são logados mas não trackados. Isso é aceitável pois shared pages
//! com muitos mappers são raras no início.
//!
//! ## Layout de Memória
//!
//! ```text
//! FrameInfo (48 bytes, cache-aligned):
//! ┌────────────┬────────────┬────────────┬────────────────────────┐
//! │   state    │  refcount  │   flags    │      rmap_inline[2]    │
//! │   8 bytes  │  4 bytes   │  4 bytes   │       16 bytes         │
//! └────────────┴────────────┴────────────┴────────────────────────┘
//! │                    rmap_overflow (8 bytes)                     │
//! └────────────────────────────────────────────────────────────────┘
//! ```

use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};

// =============================================================================
// FrameOwner
// =============================================================================

/// Owner de um frame físico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameOwner {
    /// Livre para alocação
    Free = 0,
    /// Kernel core (código, dados, page tables)
    Kernel = 1,
    /// Processo userspace
    Process { pid: u32 } = 2,
    /// Driver de dispositivo
    Driver { id: u32 } = 3,
    /// Compartilhado (CoW, mmap shared)
    Shared = 4,
    /// Hardware (framebuffer, MMIO)
    Device = 5,
    /// Não pode ser swapped
    Pinned { owner: u32 } = 6,
    /// Page cache (arquivo)
    Cache { inode: u32 } = 7,
    /// Slab allocator
    Slab = 8,
    /// Buddy allocator (metadados)
    Buddy = 9,
}

impl FrameOwner {
    /// Empacota owner em u64 para armazenamento atômico
    ///
    /// Layout: [tag: 8 bits][payload: 32 bits][reserved: 24 bits]
    pub fn pack(&self) -> u64 {
        match self {
            Self::Free => 0,
            Self::Kernel => 1,
            Self::Process { pid } => 2 | ((*pid as u64) << 8),
            Self::Driver { id } => 3 | ((*id as u64) << 8),
            Self::Shared => 4,
            Self::Device => 5,
            Self::Pinned { owner } => 6 | ((*owner as u64) << 8),
            Self::Cache { inode } => 7 | ((*inode as u64) << 8),
            Self::Slab => 8,
            Self::Buddy => 9,
        }
    }

    /// Desempacota de u64
    pub fn unpack(val: u64) -> Self {
        let tag = val & 0xFF;
        let payload = ((val >> 8) & 0xFFFFFFFF) as u32;

        match tag {
            0 => Self::Free,
            1 => Self::Kernel,
            2 => Self::Process { pid: payload },
            3 => Self::Driver { id: payload },
            4 => Self::Shared,
            5 => Self::Device,
            6 => Self::Pinned { owner: payload },
            7 => Self::Cache { inode: payload },
            8 => Self::Slab,
            9 => Self::Buddy,
            _ => Self::Free,
        }
    }

    /// Verifica se pode ser movido (compactação)
    pub fn is_movable(&self) -> bool {
        matches!(self, Self::Process { .. } | Self::Cache { .. })
    }

    /// Verifica se pode ser reclamado
    pub fn is_reclaimable(&self) -> bool {
        matches!(self, Self::Cache { .. })
    }

    /// Verifica se é pinned (não pode sair da RAM)
    pub fn is_pinned(&self) -> bool {
        matches!(
            self,
            Self::Kernel | Self::Device | Self::Pinned { .. } | Self::Slab | Self::Buddy
        )
    }
}

// =============================================================================
// FrameFlags
// =============================================================================

/// Flags do frame
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameFlags(u32);

impl FrameFlags {
    /// Nenhuma flag
    pub const NONE: Self = Self(0);
    /// Página foi modificada
    pub const DIRTY: Self = Self(1 << 0);
    /// Página foi acessada
    pub const ACCESSED: Self = Self(1 << 1);
    /// Página está locked (não pode ser evicted)
    pub const LOCKED: Self = Self(1 << 2);
    /// Página está sendo movida
    pub const MOVING: Self = Self(1 << 3);
    /// Página está em writeback
    pub const WRITEBACK: Self = Self(1 << 4);
    /// Página é uptodate (dados válidos)
    pub const UPTODATE: Self = Self(1 << 5);
    /// Página pertence a slab
    pub const SLAB: Self = Self(1 << 6);
    /// Página é compound head
    pub const HEAD: Self = Self(1 << 7);
    /// Página é compound tail
    pub const TAIL: Self = Self(1 << 8);

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
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    pub const fn bits(&self) -> u32 {
        self.0
    }
}

// =============================================================================
// RmapNode (para overflow)
// =============================================================================

/// Nó de overflow para rmap quando inline está cheio
pub struct RmapNode {
    /// Endereço do PTE
    pub pte: u64,
    /// Próximo nó na lista
    pub next: *mut RmapNode,
}

// =============================================================================
// FrameInfo
// =============================================================================

/// Metadados de um frame físico
///
/// Esta estrutura é armazenada em um array contíguo alocado durante o boot.
/// Cada frame físico tem exatamente um FrameInfo correspondente.
#[repr(C, align(64))] // Alinhado a cache line
pub struct FrameInfo {
    /// Owner empacotado (atomicamente acessível)
    state: AtomicU64,
    /// Reference count
    ref_count: AtomicU32,
    /// Flags
    flags: AtomicU32,
    /// Rmap inline (2 slots para PTEs comuns)
    /// Valor 0 significa slot vazio
    rmap_inline: [AtomicU64; 2],
    /// Overflow pointer para mais de 2 mappers
    rmap_overflow: AtomicPtr<RmapNode>,
    /// Padding para alinhar a 64 bytes
    _padding: [u8; 16],
}

impl FrameInfo {
    /// Cria novo FrameInfo (frame livre)
    pub const fn new() -> Self {
        Self {
            state: AtomicU64::new(0),
            ref_count: AtomicU32::new(0),
            flags: AtomicU32::new(0),
            rmap_inline: [AtomicU64::new(0), AtomicU64::new(0)],
            rmap_overflow: AtomicPtr::new(ptr::null_mut()),
            _padding: [0; 16],
        }
    }

    // -------------------------------------------------------------------------
    // Owner
    // -------------------------------------------------------------------------

    /// Retorna o owner atual
    #[inline]
    pub fn owner(&self) -> FrameOwner {
        FrameOwner::unpack(self.state.load(Ordering::Acquire))
    }

    /// Define o owner
    #[inline]
    pub fn set_owner(&self, owner: FrameOwner) {
        self.state.store(owner.pack(), Ordering::Release);
    }

    /// Compare-and-swap owner
    #[inline]
    pub fn cas_owner(&self, expected: FrameOwner, new: FrameOwner) -> bool {
        self.state
            .compare_exchange(
                expected.pack(),
                new.pack(),
                Ordering::AcqRel,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    // -------------------------------------------------------------------------
    // Reference Count
    // -------------------------------------------------------------------------

    /// Retorna reference count atual
    #[inline]
    pub fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::Acquire)
    }

    /// Incrementa reference count
    #[inline]
    pub fn inc_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// Decrementa reference count
    #[inline]
    pub fn dec_ref(&self) -> u32 {
        let old = self.ref_count.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(old > 0, "dec_ref on zero refcount");
        old - 1
    }

    /// Define reference count (apenas para inicialização)
    #[inline]
    pub fn set_ref_count(&self, count: u32) {
        self.ref_count.store(count, Ordering::Release);
    }

    // -------------------------------------------------------------------------
    // Flags
    // -------------------------------------------------------------------------

    /// Retorna flags atuais
    #[inline]
    pub fn flags(&self) -> FrameFlags {
        FrameFlags(self.flags.load(Ordering::Acquire))
    }

    /// Define flags
    #[inline]
    pub fn set_flags(&self, flags: FrameFlags) {
        self.flags.store(flags.0, Ordering::Release);
    }

    /// Adiciona flags
    #[inline]
    pub fn add_flags(&self, flags: FrameFlags) {
        self.flags.fetch_or(flags.0, Ordering::AcqRel);
    }

    /// Remove flags
    #[inline]
    pub fn remove_flags(&self, flags: FrameFlags) {
        self.flags.fetch_and(!flags.0, Ordering::AcqRel);
    }

    /// Testa se contém flags
    #[inline]
    pub fn has_flags(&self, flags: FrameFlags) -> bool {
        (self.flags.load(Ordering::Acquire) & flags.0) == flags.0
    }

    // -------------------------------------------------------------------------
    // Rmap (Reverse Mapping)
    // -------------------------------------------------------------------------

    /// Adiciona PTE ao rmap
    ///
    /// Retorna true se adicionado com sucesso, false se overflow
    pub fn rmap_add(&self, pte: u64) -> bool {
        // Tenta slot inline 0
        if self.rmap_inline[0]
            .compare_exchange(0, pte, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return true;
        }

        // Tenta slot inline 1
        if self.rmap_inline[1]
            .compare_exchange(0, pte, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return true;
        }

        // Overflow - para V1, apenas loga e ignora
        // TODO V2: Implementar lista de overflow com hazard pointers
        #[cfg(debug_assertions)]
        crate::kwarn!("(RMM) rmap overflow for frame, PTE 0x{:x}", pte);

        false
    }

    /// Remove PTE do rmap
    pub fn rmap_remove(&self, pte: u64) -> bool {
        // Tenta inline 0
        if self.rmap_inline[0]
            .compare_exchange(pte, 0, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return true;
        }

        // Tenta inline 1
        if self.rmap_inline[1]
            .compare_exchange(pte, 0, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return true;
        }

        // Não encontrado nos slots inline
        // TODO V2: Procurar na lista de overflow
        false
    }

    /// Itera sobre todos os PTEs mapeados
    ///
    /// Callback recebe endereço do PTE. Retorna true para continuar, false para parar.
    pub fn rmap_for_each<F>(&self, mut callback: F)
    where
        F: FnMut(u64) -> bool,
    {
        // Inline slots
        let pte0 = self.rmap_inline[0].load(Ordering::Acquire);
        if pte0 != 0 && !callback(pte0) {
            return;
        }

        let pte1 = self.rmap_inline[1].load(Ordering::Acquire);
        if pte1 != 0 && !callback(pte1) {
            return;
        }

        // Overflow list
        let mut node = self.rmap_overflow.load(Ordering::Acquire);
        while !node.is_null() {
            unsafe {
                if !callback((*node).pte) {
                    return;
                }
                node = (*node).next;
            }
        }
    }

    /// Conta número de mappers
    pub fn rmap_count(&self) -> usize {
        let mut count = 0;

        if self.rmap_inline[0].load(Ordering::Relaxed) != 0 {
            count += 1;
        }
        if self.rmap_inline[1].load(Ordering::Relaxed) != 0 {
            count += 1;
        }

        // Contar overflow
        let mut node = self.rmap_overflow.load(Ordering::Acquire);
        while !node.is_null() {
            count += 1;
            unsafe {
                node = (*node).next;
            }
        }

        count
    }

    /// Limpa todos os rmap entries
    pub fn rmap_clear(&self) {
        self.rmap_inline[0].store(0, Ordering::Release);
        self.rmap_inline[1].store(0, Ordering::Release);

        // TODO: Liberar nós de overflow
        self.rmap_overflow.store(ptr::null_mut(), Ordering::Release);
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Verifica se frame está livre
    #[inline]
    pub fn is_free(&self) -> bool {
        self.owner() == FrameOwner::Free
    }

    /// Verifica se frame pode ser evicted
    #[inline]
    pub fn is_evictable(&self) -> bool {
        !self.has_flags(FrameFlags::LOCKED)
            && !self.has_flags(FrameFlags::MOVING)
            && self.owner().is_reclaimable()
    }

    /// Reseta frame para estado inicial (livre)
    pub fn reset(&self) {
        self.set_owner(FrameOwner::Free);
        self.set_ref_count(0);
        self.set_flags(FrameFlags::NONE);
        self.rmap_clear();
    }
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: FrameInfo usa apenas operações atômicas
unsafe impl Send for FrameInfo {}
unsafe impl Sync for FrameInfo {}
