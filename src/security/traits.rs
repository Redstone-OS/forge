//! # Security Traits
//!
//! Interfaces comuns para o subsistema de segurança.

use super::capability::CSpace;
use super::sandbox::NamespaceSet;

/// Contexto de segurança de uma entidade.
///
/// Toda entidade que precisa de verificação de segurança
/// (processos, threads, módulos) implementa esta trait.
pub trait SecurityContext {
    /// Retorna o CSpace da entidade.
    fn cspace(&self) -> &CSpace;

    /// Retorna CSpace mutável.
    fn cspace_mut(&mut self) -> &mut CSpace;

    /// Retorna namespaces da entidade.
    fn namespaces(&self) -> &NamespaceSet;

    /// ID único para auditoria.
    fn security_id(&self) -> u64;
}

/// Entidade que pode ser auditada.
pub trait Auditable {
    /// ID de auditoria.
    fn audit_id(&self) -> u64;

    /// Nome para logs.
    fn audit_name(&self) -> &str;
}

/// Objeto que pode ser acessado via capability.
pub trait CapabilityObject {
    /// Tipo de capability necessário.
    fn cap_type(&self) -> super::capability::CapType;

    /// ID do objeto para referência.
    fn object_id(&self) -> u64;
}
