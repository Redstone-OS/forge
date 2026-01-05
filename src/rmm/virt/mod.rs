//! # Gerenciamento de Memória Virtual
//!
//! Este módulo implementa o gerenciador de memória virtual (VMM).
//!
//! ## Componentes
//!
//! - `mapper`: Manipulação de page tables (map/unmap/translate)
//! - `hhdm`: Higher Half Direct Map
//! - `tlb`: TLB management e IPI shootdown
//! - `aspace`: Address Space per-process

pub mod aspace;
pub mod hhdm;
pub mod mapper;
pub mod tlb;

pub use aspace::AddressSpace;
pub use hhdm::{phys_to_virt, virt_to_phys};
pub use mapper::{map_page, translate, unmap_page, MapFlags};
pub use tlb::{flush_tlb, flush_tlb_all};
