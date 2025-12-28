//! # Security Subsystem
//!
//! Segurança baseada em Capabilities (não ACLs).
//!
//! ## Filosofia
//!
//! ```text
//! ╔═══════════════════════════════════════════════════════════╗
//! ║  CAPABILITY-BASED SECURITY                                ║
//! ║                                                           ║
//! ║  • Acesso via TOKEN, não identidade                       ║
//! ║  • Sem "root" ou superusuário global                      ║
//! ║  • Least privilege por design                             ║
//! ║  • Delegação explícita via transfer                       ║
//! ╚═══════════════════════════════════════════════════════════╝
//! ```
//!
//! ## Modelo
//!
//! ```text
//! Process A                    Process B
//! ┌─────────┐                  ┌─────────┐
//! │ CSpace  │                  │ CSpace  │
//! │ ┌─────┐ │    transfer      │ ┌─────┐ │
//! │ │Cap 1│ │ ───────────────► │ │Cap 1│ │
//! │ └─────┘ │                  │ └─────┘ │
//! │ ┌─────┐ │                  │         │
//! │ │Cap 2│ │                  │         │
//! │ └─────┘ │                  │         │
//! └─────────┘                  └─────────┘
//! ```
//!
//! ## Rights
//!
//! | Right     | Descrição                          |
//! |-----------|------------------------------------|
//! | READ      | Ler conteúdo                       |
//! | WRITE     | Modificar conteúdo                 |
//! | EXECUTE   | Executar código                    |
//! | DUPLICATE | Criar cópia da capability          |
//! | TRANSFER  | Enviar via IPC                     |
//! | GRANT     | Criar capability derivada          |

// =============================================================================
// CAPABILITIES
// =============================================================================

/// Sistema de capabilities
pub mod capability;

pub use capability::{CSpace, CapHandle, CapRights, CapType, Capability};

// =============================================================================
// CREDENTIALS
// =============================================================================

/// Credenciais de processo
pub mod credentials;

pub use credentials::Credentials;

// =============================================================================
// SANDBOX
// =============================================================================

/// Namespaces e isolamento
pub mod sandbox;

pub use sandbox::Sandbox;

// =============================================================================
// AUDIT
// =============================================================================

/// Logging de segurança
pub mod audit;

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa subsistema de segurança
pub fn init() {
    crate::kinfo!("(Security) Inicializando capabilities...");
    // Inicializar CSpace global do kernel
    crate::kinfo!("(Security) Segurança inicializada");
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
