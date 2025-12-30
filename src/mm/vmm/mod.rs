//! Virtual Memory Manager (VMM)
//!
//! Gerencia tabelas de páginas e endereçamento virtual.

pub mod mapper;
pub mod tlb;
pub mod vmm;

pub use mapper::{map_page, map_page_in_target_p4, map_page_with_pmm, translate_addr, unmap_page};
pub use vmm::{init, MapFlags, PageTable};
