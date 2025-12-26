//! # VMM - Virtual Memory Manager
//!
//! Gerencia page tables x86_64 e mapeamentos.

// TODO: Refatorar para dissolver vmm.rs em módulos menores (paging, mapper, etc)
#[allow(clippy::module_inception)]
pub mod vmm;

pub use vmm::*;
