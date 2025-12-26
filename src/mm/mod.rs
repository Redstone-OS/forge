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
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa todo o subsistema de memória na ordem correta.
///
/// # Ordem de Inicialização (CRÍTICO)
///
/// 1. **VMM primeiro**: Registra CR3 do bootloader, valida scratch slot
/// 2. **PMM segundo**: Pode acessar memória física via identity map validado
/// 3. **Heap terceiro**: Precisa do PMM para alocar frames
///
/// # Safety
///
/// - Deve ser chamado uma única vez no early-boot
/// - O boot_info deve ser válido e estático
/// - O CR3 deve conter page tables válidas do bootloader
///
/// # Panics
///
/// Faz panic se a inicialização do Heap falhar.
pub unsafe fn init(boot_info: &'static crate::core::handoff::BootInfo) {
    // 1. VMM primeiro: registra CR3, valida scratch slot
    crate::kdebug!("(MM) Inicializando VMM...");
    vmm::init(boot_info);

    // 2. PMM segundo: pode acessar memória física via identity map
    crate::kdebug!("(MM) Inicializando PMM...");
    pmm::init(boot_info);

    // 3. Heap terceiro: precisa do PMM para alocar frames
    crate::kdebug!("(MM) Inicializando Heap...");
    if !heap::init_heap(&mut *pmm::FRAME_ALLOCATOR.lock()) {
        panic!("(MM) Falha crítica ao inicializar Heap!");
    }

    crate::kinfo!("(MM) Subsistema de memória inicializado!");
}

// =============================================================================
// RE-EXPORTS
// =============================================================================

// PMM
pub use pmm::{
    BitmapFrameAllocator, MemoryRegion, MemoryRegionType, PhysFrame, PmmStats, FRAME_ALLOCATOR,
};

// Addr
pub use addr::{PhysAddr, VirtAddr};

// Error
pub use error::MmError;
pub type Result<T> = core::result::Result<T, MmError>;

// VMM (APIs principais)
pub use vmm::{map_page, map_page_with_pmm, translate_addr, unmap_page};

// Mapper (API de alto nível)
pub use vmm::mapper::{MapFlags, MappedRegion, RegionType};
