//! # Capability Module
//!
//! Sistema de Object-Capabilities.
//!
//! ## Componentes:
//! - `Capability`: Token de acesso
//! - `CapType`: Tipo de objeto referenciado
//! - `CapRights`: Direitos concedidos
//! - `CSpace`: Tabela de capabilities por processo
//! - `CapHandle`: Handle opaco para userspace

mod cap;
mod cspace;
mod revocation;
mod rights;

pub use cap::{CapHandle, CapType, Capability};
pub use cspace::{CSpace, CapError};
pub use revocation::RevocationTree;
pub use rights::CapRights;
