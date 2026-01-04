//! # Security Subsystem
//!
//! Sistema de segurança baseado em Object-Capabilities (OCAP).
//!
//! ## Filosofia
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │  CAPABILITY-BASED SECURITY                                   │
//! │                                                              │
//! │  • Acesso via TOKEN, não identidade                          │
//! │  • Sem "root" ou superusuário global                         │
//! │  • Least privilege por design                                │
//! │  • Delegação explícita via transfer                          │
//! │  • Revogação a qualquer momento                              │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Módulos:
//!
//! | Módulo       | Descrição                              | Status |
//! |--------------|----------------------------------------|--------|
//! | `capability` | CSpace, Rights, CapHandle              | Func   |
//! | `audit`      | Logging de eventos de segurança        | WIP    |
//! | `sandbox`    | Isolamento via namespaces              | WIP    |
//!
//! ## Modelo de Acesso:
//!
//! ```text
//! Process → CSpace → Capability → Object
//!                        ↓
//!                    Rights Check
//!                        ↓
//!                 Allow / Deny + Audit
//! ```

pub mod audit;
pub mod capability;
pub mod sandbox;
pub mod traits;

// Re-exports principais
pub use audit::{AuditEvent, AuditResult};
pub use capability::{CSpace, CapError, CapHandle, CapRights, CapType, Capability};
pub use sandbox::{Container, Namespace, NamespaceType};

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema de segurança.
pub fn init() {
    crate::kinfo!("(Security) Inicializando subsistema de segurança...");

    // Inicializar audit logger
    audit::init();

    crate::kinfo!("(Security) Modelo OCAP ativo");
}

/// Desliga o subsistema (flush de logs, etc).
pub fn shutdown() {
    crate::kinfo!("(Security) Shutdown do subsistema...");
    audit::flush();
}

// =============================================================================
// CONVENIÊNCIA
// =============================================================================

/// Loga evento de auditoria.
#[inline]
pub fn audit_log(event: AuditEvent, result: AuditResult) {
    audit::log(event, result);
}

// =============================================================================
// TESTES
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
