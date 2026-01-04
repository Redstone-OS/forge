//! # Audit Subsystem
//!
//! Logging de eventos de segurança.
//!
//! ## Integração
//!
//! Atualmente os logs vão para `core::debug`.
//! TODO: Persistência em disco, análise em tempo real.

mod events;
mod policy;

pub use events::{AuditCategory, AuditEvent, AuditResult};
pub use policy::AuditPolicy;

use crate::sync::Spinlock;

/// Política de audit ativa.
static POLICY: Spinlock<AuditPolicy> = Spinlock::new(AuditPolicy::default_policy());

/// Inicializa o subsistema de audit.
pub fn init() {
    crate::kdebug!("(Audit) Logger inicializado");
}

/// Flush dos logs pendentes.
pub fn flush() {
    // TODO: Quando tiver buffer, fazer flush
}

/// Loga um evento de segurança.
pub fn log(event: AuditEvent, result: AuditResult) {
    let policy = POLICY.lock();

    // Verifica se este evento deve ser logado
    if !policy.should_log(&event, &result) {
        return;
    }

    // Formatar e enviar para debug
    match result {
        AuditResult::Success => {
            crate::kdebug!("(Audit) {} - OK", event.name());
        }
        AuditResult::Denied => {
            crate::kwarn!("(Audit) {} - DENIED", event.name());
        }
        AuditResult::Error(code) => {
            crate::kerror!("(Audit) {} - ERROR {}", event.name(), code);
        }
    }
}

/// Loga evento com detalhes.
pub fn log_detailed(event: AuditEvent, result: AuditResult, subject: u64, object: u64) {
    let policy = POLICY.lock();

    if !policy.should_log(&event, &result) {
        return;
    }

    match result {
        AuditResult::Success => {
            crate::kdebug!(
                "(Audit) {} - OK [subj={:#x}, obj={:#x}]",
                event.name(),
                subject,
                object
            );
        }
        AuditResult::Denied => {
            crate::kwarn!(
                "(Audit) {} - DENIED [subj={:#x}, obj={:#x}]",
                event.name(),
                subject,
                object
            );
        }
        AuditResult::Error(code) => {
            crate::kerror!(
                "(Audit) {} - ERROR {} [subj={:#x}, obj={:#x}]",
                event.name(),
                code,
                subject,
                object
            );
        }
    }
}

/// Atualiza política de audit.
pub fn set_policy(policy: AuditPolicy) {
    *POLICY.lock() = policy;
}

/// Retorna política atual.
pub fn get_policy() -> AuditPolicy {
    POLICY.lock().clone()
}
