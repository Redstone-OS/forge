//! Módulo de Segurança (Capabilities).
//!
//! Responsável por definir tipos e validações de acesso.
//! No futuro, conterá a lógica de CSpace (Capability Space) por processo.

pub mod capability;

pub use capability::{CapHandle, CapRights, CapType, Capability};

// TODO: Implementar CNode / CSpace (Tabela de Capabilities)
