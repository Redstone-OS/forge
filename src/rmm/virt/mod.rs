//! # Gerenciamento de Memória Virtual
//!
//! Este módulo implementa o Virtual Memory Manager (VMM) do RMM.
//!
//! ## Componentes
//!
//! - `mapper`: Manipulação de page tables (map/unmap/translate)
//! - `hhdm`: Higher Half Direct Map
//! - `tlb`: TLB management e IPI shootdown
//! - `aspace`: Address Space per-process (VMAs)
//!
//! ## Arquitetura x86_64
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                    Virtual Address (48 bits)               │
//! ├─────────┬─────────┬─────────┬─────────┬────────────────────┤
//! │  PML4   │  PDPT   │   PD    │   PT    │      Offset        │
//! │ 9 bits  │ 9 bits  │ 9 bits  │ 9 bits  │     12 bits        │
//! │ [47:39] │ [38:30] │ [29:21] │ [20:12] │     [11:0]         │
//! └─────────┴─────────┴─────────┴─────────┴────────────────────┘
//! ```
//!
//! ## Layout de Memória Virtual
//!
//! ```text
//! 0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : Userspace (128 TB)
//! 0xFFFF_8000_0000_0000 - 0xFFFF_8FFF_FFFF_FFFF : HHDM (16 TB)
//! 0xFFFF_9000_0000_0000 - 0xFFFF_9FFF_FFFF_FFFF : Kernel Heap (1 TB)
//! 0xFFFF_C000_0000_0000 - 0xFFFF_CFFF_FFFF_FFFF : Driver Zone (1 TB)
//! 0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel Code
//! ```

pub mod aspace;
pub mod hhdm;
pub mod mapper;
pub mod tlb;

pub use aspace::AddressSpace;
pub use hhdm::{phys_to_virt, virt_to_phys};
pub use mapper::{map_page, translate, unmap_page, MapFlags};
pub use tlb::{flush_tlb, flush_tlb_all, flush_tlb_range};

use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::error::RmmResult;

/// Inicializa o subsistema de memória virtual
pub unsafe fn init() {
    crate::kinfo!("(RMM/Virt) Subsistema virtual inicializado");
}

/// Mapeia página no address space do kernel
pub fn kernel_map(virt: VirtAddr, phys: PhysAddr, flags: MapFlags) -> RmmResult<()> {
    mapper::map_page(virt, phys, flags)
}

/// Remove mapeamento do kernel
pub fn kernel_unmap(virt: VirtAddr) -> RmmResult<Option<PhysAddr>> {
    mapper::unmap_page(virt)
}

/// Traduz endereço virtual do kernel para físico
pub fn kernel_translate(virt: VirtAddr) -> Option<PhysAddr> {
    mapper::translate(virt)
}

/// Verifica se endereço está no espaço do kernel
#[inline]
pub fn is_kernel_address(virt: VirtAddr) -> bool {
    virt.as_u64() >= 0xFFFF_8000_0000_0000
}

/// Verifica se endereço está no espaço do usuário
#[inline]
pub fn is_user_address(virt: VirtAddr) -> bool {
    virt.as_u64() < 0x0000_8000_0000_0000
}

/// Verifica se endereço é canônico (válido em x86_64)
#[inline]
pub fn is_canonical(virt: VirtAddr) -> bool {
    let addr = virt.as_u64();
    // Bits 48-63 devem ser iguais ao bit 47
    let high_bits = addr >> 47;
    high_bits == 0 || high_bits == 0x1FFFF
}
