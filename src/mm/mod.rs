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
/// 4. **Guard Page**: Desmapeia página de guarda da stack
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
    crate::kinfo!("(MM) Inicializando VMM...");
    vmm::init(boot_info);

    // 2. PMM segundo: pode acessar memória física via identity map
    // Nota: pt_scanner é chamado DENTRO de pmm::init, ANTES de liberar frames
    crate::kinfo!("(MM) Inicializando PMM...");
    pmm::init(boot_info);

    // 3. Heap terceiro: precisa do PMM para alocar frames
    crate::kinfo!("(MM) Inicializando Heap...");
    if !heap::init_heap(&mut *pmm::FRAME_ALLOCATOR.lock()) {
        panic!("(MM) Falha crítica ao inicializar Heap!");
    }

    // 4. Guard Page: desmapeia página de guarda da stack para detectar stack overflow
    crate::kinfo!("(MM) Configurando guard page da stack...");
    setup_guard_page();

    crate::kok!("(MM) Subsistema de memória inicializado com sucesso!");
}

/// Configura a guard page da stack do kernel.
///
/// Desmapeia a página imediatamente antes da stack do kernel,
/// causando Page Fault se ocorrer stack overflow.
unsafe fn setup_guard_page() {
    // Obter endereço da guard page do main.rs
    // A guard page está ANTES da stack (endereço menor)
    extern "C" {
        static KERNEL_STACK: u8;
    }

    let stack_base = &KERNEL_STACK as *const u8 as u64;
    let guard_page_addr = stack_base.saturating_sub(4096);

    // Desmapear a guard page (marcar como NOT PRESENT)
    // Se já não está mapeada, unmap_page é um no-op seguro
    if let Err(_e) = vmm::unmap_page(guard_page_addr) {
        crate::kwarn!("(MM) Guard page não pôde ser desmapeada (pode já estar desmapeada)");
    } else {
        crate::kinfo!("(MM) Guard page configurada em=", guard_page_addr);
    }

    // Invalidar TLB para a guard page
    vmm::tlb::invlpg(addr::VirtAddr::new(guard_page_addr));
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

// TLB (flush_all para casos especiais)
pub use vmm::tlb::{flush_tlb_local, invalidate_page, invlpg};

// Mapper (API de alto nível)
pub use vmm::mapper::{MapFlags, MappedRegion, RegionType};
