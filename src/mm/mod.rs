//! # Memory Management (MM)
//!
//! Gerenciamento completo de memória física e virtual.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                      HEAP                           │
//! │  GlobalAlloc → Slab/Buddy → Vec, Box, Arc           │
//! └─────────────────────────────────────────────────────┘
//!                        ↑
//! ┌─────────────────────────────────────────────────────┐
//! │                      VMM                            │
//! │  Page Tables → Mapper → VirtAddr ↔ PhysAddr         │
//! └─────────────────────────────────────────────────────┘
//!                        ↑
//! ┌─────────────────────────────────────────────────────┐
//! │                      PMM                            │
//! │  Bitmap → PhysFrame → Free/Used tracking            │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Ordem de Inicialização
//!
//! 1. VMM: Registra CR3 do bootloader
//! 2. PMM: Escaneia mapa de memória
//! 3. Heap: Conecta ao GlobalAlloc

// =============================================================================
// ADDRESS TYPES
// =============================================================================

/// Wrappers type-safe para endereços
/// Wrappers type-safe para endereços
pub mod addr;
pub use addr::{PhysAddr, VirtAddr};

// =============================================================================
// PHYSICAL MEMORY MANAGER
// =============================================================================

/// Gerenciador de memória física
pub mod pmm;
pub use pmm::{FrameAllocator, PhysFrame, FRAME_ALLOCATOR};

// =============================================================================
// VIRTUAL MEMORY MANAGER
// =============================================================================

/// Gerenciador de memória virtual
pub mod vmm;
pub use vmm::{map_page, translate_addr, unmap_page, MapFlags, PageTable};

// =============================================================================
// ALLOCATORS
// =============================================================================

/// Alocadores (buddy, slab, bump)
pub mod alloc;

/// Heap do kernel (GlobalAlloc)
pub mod heap;

// =============================================================================
// SUPPORT
// =============================================================================

/// Configurações e constantes
pub mod config;

/// Tipos de erro
pub mod error;

/// Operações de memória (memset, memcpy)
pub mod ops;

/// Handler de OOM
pub mod oom;

/// Tipos seguros (VMO, Pinned)
pub mod types;

/// Page cache
pub mod cache;

/// Accounting por subsistema
#[cfg(feature = "memory_accounting")]
pub mod accounting;

// =============================================================================
// NEW MEMORY SUBSYSTEM MODULES
// =============================================================================

/// Higher Half Direct Map
pub mod hhdm;

/// Early Boot Allocator
pub mod early;

/// Page Fault Handler
pub mod fault;

/// Page Frame Manager (ownership-based)
pub mod pfm;

/// Address Space Manager
pub mod aspace;

/// Page Reclaim subsystem
pub mod reclaim;

/// Swap subsystem
pub mod swap;

/// Memory tracepoints
pub mod trace;

/// Memory statistics
pub mod stats;

/// Debug utilities
pub mod debug;

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa o subsistema de memória
///
/// # Safety
///
/// Deve ser chamado uma única vez durante early-boot.
pub unsafe fn init(boot_info: &'static crate::core::BootInfo) {
    crate::kinfo!("(MM) Inicializando VMM...");
    vmm::init(boot_info);

    crate::kinfo!("(MM) Inicializando PMM...");
    pmm::init(boot_info);

    crate::kinfo!("(MM) Inicializando Heap...");
    heap::init(&mut *pmm::FRAME_ALLOCATOR.lock());

    crate::kinfo!("(MM) Memória inicializada");
}

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use error::MmError;
pub type Result<T> = core::result::Result<T, MmError>;

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
