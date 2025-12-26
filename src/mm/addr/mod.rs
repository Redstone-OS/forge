//! # Addr - Wrappers Type-Safe para Endereços
//!
//! Tipos distintos para PhysAddr e VirtAddr evitando confusão.

mod phys;
mod translate;
mod virt;

pub use phys::PhysAddr;
pub use translate::{is_phys_accessible, phys_to_virt, virt_to_phys};
pub use virt::VirtAddr;
