//! # MM - Módulo de Memória do Kernel Redstone OS
//!
//! Este módulo implementa todo o gerenciamento de memória do kernel:
//! - PMM: Physical Memory Manager (frames físicos)
//! - VMM: Virtual Memory Manager (page tables)
//! - Heap: Alocação dinâmica (Box, Vec, String)
//! - Accounting: Rastreamento de uso por subsistema
//! - Types: Tipos seguros (VMO, Pinned)

// =============================================================================
// MÓDULOS PRINCIPAIS
// =============================================================================

/// Wrappers type-safe para endereços
pub mod addr;

/// Alocadores (Buddy, Slab, Bump, Per-CPU)
pub mod alloc;

/// Heap do kernel (GlobalAlloc)
pub mod heap;

/// Handler de OOM
pub mod oom;

/// Operações de memória (memset, memcpy)
pub mod ops;

/// Physical Memory Manager
pub mod pmm;

/// Testes do subsistema
pub mod test;

/// Virtual Memory Manager
pub mod vmm;

// =============================================================================
// NOVOS MÓDULOS
// =============================================================================

/// Memory Accounting por subsistema
#[cfg(feature = "memory_accounting")]
pub mod accounting;

/// Tipos seguros de memória (VMO, Pinned)
pub mod types;

// =============================================================================
// CONFIGURAÇÃO
// =============================================================================

/// Configurações e constantes
pub mod config;

/// Tipos de erro
pub mod error;

// =============================================================================
// RE-EXPORTS
// =============================================================================

// PMM
pub use pmm::init;
pub use pmm::{
    BitmapFrameAllocator, MemoryRegion, MemoryRegionType, PhysFrame, PmmStats, FRAME_ALLOCATOR,
};

// Addr
pub use addr::{PhysAddr, VirtAddr};

// Error
pub use error::MmError;
pub type Result<T> = core::result::Result<T, MmError>;

// VMM (APIs principais)
pub use vmm::{map_page, map_page_with_pmm, translate_addr};

// Mapper (API de alto nível)
pub use vmm::mapper::{MapFlags, MappedRegion, RegionType};
