//! # Sandbox Subsystem
//!
//! Isolamento de processos via namespaces e containers.
//!
//! ## Aplicação Principal
//!
//! - Módulos de kernel rodam em containers com capabilities limitadas
//! - Processos podem ser isolados do resto do sistema
//! - Cada processo "vê" apenas o que foi dado a ele

mod container;
mod namespace;

pub use container::{Container, ResourceLimits};
pub use namespace::{Namespace, NamespaceSet, NamespaceType};
