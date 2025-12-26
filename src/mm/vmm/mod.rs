//! # VMM - Virtual Memory Manager
//!
//! Gerencia page tables x86_64 e mapeamentos.

// Módulo principal de paging
#[allow(clippy::module_inception)]
pub mod vmm;
pub use vmm::*;

// TLB Management e Shootdown
pub mod tlb;

// API de alto nível para mapeamento
pub mod mapper;
