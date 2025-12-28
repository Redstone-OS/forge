//! Log de auditoria de segurança

use crate::sys::types::{Pid, Uid};

/// Tipo de evento de auditoria
#[derive(Debug, Clone, Copy)]
pub enum AuditEvent {
    CapabilityGranted,
    CapabilityDenied,
    CapabilityRevoked,
    ProcessCreated,
    ProcessTerminated,
    AccessDenied,
    PrivilegeEscalation,
}

/// Entrada de log de auditoria
pub struct AuditEntry {
    pub timestamp: u64,
    pub event: AuditEvent,
    pub pid: Pid,
    pub uid: Uid,
    pub details: [u8; 64],
}

/// Loga evento de auditoria
pub fn log_event(event: AuditEvent, pid: Pid, uid: Uid) {
    // Evitar warning unused
    let _ = pid;
    let _ = uid;
    // TODO: adicionar ao buffer de log
    crate::kdebug!("Audit:", event as u64);
}
