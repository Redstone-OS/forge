//! # Redstone Memory Manager (RMM)
//!
//! O RMM é o subsistema central de gerenciamento de memória do kernel Forge.
//! Unifica alocação física, mapeamento virtual, heap e memória para drivers
//! em uma arquitetura coesa e bem definida.
//!
//! ## Filosofia
//!
//! > *"Cada byte de memória tem dono, endereço e propósito. Nenhuma alocação é anônima."*
//!
//! ## Arquitetura
//!
//! O RMM é dividido em camadas:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                         RMM                                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
//! │  │  phys   │  │  virt   │  │  heap   │  │ driver  │         │
//! │  │ Physical│  │ Virtual │  │ Kernel  │  │  DMA    │         │
//! │  │ Frames  │  │ Mapping │  │  Heap   │  │ Buffers │         │
//! │  └─────────┘  └─────────┘  └─────────┘  └─────────┘         │
//! │       │            │            │            │              │
//! │       └────────────┴────────────┴────────────┘              │
//! │                         │                                   │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │                    zone                             │    │
//! │  │           DMA | DMA32 | Normal                      │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Componentes
//!
//! | Módulo | Descrição |
//! |--------|-----------|
//! | `config` | Constantes e configuração |
//! | `error` | Tipos de erro (`RmmError`) |
//! | `addr` | Tipos de endereço (`PhysAddr`, `VirtAddr`) |
//! | `early` | Alocador de boot (bump allocator) |
//! | `zone` | Zonas de memória física (DMA, DMA32, Normal) |
//! | `phys` | Gerenciador de frames físicos (`FrameManager`) |
//! | `virt` | Mapeamento virtual (page tables, HHDM, TLB) |
//! | `heap` | Heap do kernel (Buddy + Slab) |
//! | `driver` | API de memória para drivers (DMA, IOMMU) |
//! | `numa` | Suporte a NUMA (preparado, stub) |
//! | `reclaim` | Page reclaim (stub com contrato) |
//! | `swap` | Swap (stub com contrato) |
//! | `cache` | Page cache (stub com contrato) |
//! | `debug` | Debug e observabilidade |
//!
//! ## Inicialização
//!
//! A inicialização deve ser feita na seguinte ordem:
//!
//! 1. `early::init()` - Prepara alocador de boot
//! 2. `virt::hhdm::init()` - Configura HHDM
//! 3. `phys::init()` - Inicializa FrameManager
//! 4. `heap::init()` - Inicializa heap do kernel
//!
//! ## Uso
//!
//! ```rust
//! use crate::rmm::{PhysAddr, VirtAddr, Zone, AllocFlags};
//! use crate::rmm::phys;
//!
//! // Alocar frame físico
//! let frame = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)?;
//!
//! // Mapear para virtual
//! virt::map_page(virt_addr, frame, MapFlags::PRESENT | MapFlags::WRITABLE)?;
//!
//! // Usar heap
//! let boxed = Box::new(42);
//! ```

// Submódulos
pub mod addr;
pub mod config;
pub mod early;
pub mod error;
pub mod zone;

pub mod driver;
pub mod heap;
pub mod phys;
pub mod virt;

pub mod cache;
pub mod fault;
pub mod numa;
pub mod reclaim;
pub mod swap;

pub mod debug;

// Re-exports principais
pub use addr::{PhysAddr, VirtAddr};
pub use config::*;
pub use error::{RmmError, RmmResult};
pub use phys::{AllocFlags, FrameOwner};
pub use zone::{MigrateType, Zone};

// Re-export de phys_to_virt para compatibilidade, remover assim que possivel
pub mod addr_compat {
    //! Compatibilidade com API antiga de crate::mm::addr

    /// Converte endereço físico para virtual via HHDM
    ///
    /// # Safety
    ///
    /// O endereço físico deve ser válido.
    #[inline]
    pub unsafe fn phys_to_virt<T>(phys: u64) -> *mut T {
        crate::rmm::virt::hhdm::phys_to_virt(phys) as *mut T
    }
}

use crate::core::boot::BootInfo;

/// Flag global indicando se o RMM foi inicializado
static mut RMM_INITIALIZED: bool = false;

/// Verifica se o RMM está inicializado
#[inline]
pub fn is_initialized() -> bool {
    unsafe { RMM_INITIALIZED }
}

/// Inicializa o RMM completo.
///
/// Esta função deve ser chamada uma única vez durante o boot,
/// após o bootloader ter passado as informações de memória.
///
/// # Ordem de Inicialização
///
/// 1. Early allocator (para alocar metadados do FrameManager)
/// 2. HHDM (Higher Half Direct Map)
/// 3. FrameManager (gerenciador de frames físicos)
/// 4. Heap do kernel (Buddy + Slab)
///
/// # Safety
///
/// - Deve ser chamada apenas uma vez
/// - Deve ser chamada antes de qualquer alocação de memória
/// - O BootInfo deve ser válido
///
/// # Panics
///
/// Panic se:
/// - Chamada mais de uma vez
/// - BootInfo inválido
/// - Falha ao inicializar qualquer subsistema
pub unsafe fn init(boot_info: &'static BootInfo) {
    if RMM_INITIALIZED {
        panic!("RMM: init() chamado mais de uma vez!");
    }

    crate::kinfo!("(RMM) Inicializando Redstone Memory Manager...");

    // Fase 1: Early Allocator
    crate::kinfo!("(RMM) [1/4] Early Allocator...");
    early::init(boot_info);

    // Fase 2: HHDM
    crate::kinfo!("(RMM) [2/4] HHDM...");
    virt::hhdm::init(boot_info);

    // Fase 3: FrameManager
    crate::kinfo!("(RMM) [3/4] FrameManager...");
    phys::init(boot_info);

    // Fase 4: Heap
    crate::kinfo!("(RMM) [4/4] Heap...");
    heap::init();

    RMM_INITIALIZED = true;

    crate::kinfo!("(RMM) Inicialização completa!");

    // Dump estatísticas iniciais
    #[cfg(debug_assertions)]
    debug::dump_stats();
}
