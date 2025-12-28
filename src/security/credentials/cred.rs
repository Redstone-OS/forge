//! Credenciais de processo

use crate::sys::types::{Uid, Gid};

/// Credenciais do processo
#[derive(Debug, Clone)]
pub struct Credentials {
    /// UID real
    pub uid: Uid,
    /// GID real
    pub gid: Gid,
    /// UID efetivo
    pub euid: Uid,
    /// GID efetivo
    pub egid: Gid,
    /// GIDs suplementares
    pub groups: [Gid; 16],
    pub num_groups: usize,
}

impl Credentials {
    /// Credenciais root
    pub const fn root() -> Self {
        Self {
            uid: Uid::ROOT,
            gid: Gid::ROOT,
            euid: Uid::ROOT,
            egid: Gid::ROOT,
            groups: [Gid::ROOT; 16],
            num_groups: 0,
        }
    }
    
    /// Credenciais padrão de usuário
    pub const fn user(uid: Uid, gid: Gid) -> Self {
        Self {
            uid,
            gid,
            euid: uid,
            egid: gid,
            groups: [Gid::ROOT; 16],
            num_groups: 0,
        }
    }
    
    /// Verifica se é privilegiado
    pub fn is_privileged(&self) -> bool {
        self.euid.0 == 0
    }
}
