//! # Credentials (Legacy Compatibility)
//!
//! Este módulo existe apenas para compatibilidade com APIs que
//! esperam UID/GID. No modelo OCAP do RedstoneOS, identidade
//! não confere autorização - apenas capabilities fazem isso.
//!
//! ## Uso
//!
//! - Logging/auditoria (identificar "quem" fez algo)
//! - Compatibilidade POSIX em camada de emulação
//!
//! ## Não Use Para
//!
//! - Verificação de permissão (use capabilities)
//! - Controle de acesso (use capabilities)

/// UID numérico (apenas para identificação).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Uid(pub u32);

impl Uid {
    pub const KERNEL: Self = Self(0);
}

/// GID numérico (apenas para identificação).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Gid(pub u32);

impl Gid {
    pub const KERNEL: Self = Self(0);
}

/// Credenciais de identificação (NÃO de autorização).
///
/// Usado para auditoria e logging, não para controle de acesso.
#[derive(Debug, Clone)]
pub struct Credentials {
    /// UID para identificação.
    pub uid: Uid,
    /// GID para identificação.
    pub gid: Gid,
    /// Nome legível (para debug/logs).
    pub name: &'static str,
}

impl Credentials {
    /// Credenciais do kernel.
    pub const fn kernel() -> Self {
        Self {
            uid: Uid::KERNEL,
            gid: Gid::KERNEL,
            name: "kernel",
        }
    }

    /// Credenciais genéricas.
    pub fn new(uid: u32, gid: u32, name: &'static str) -> Self {
        Self {
            uid: Uid(uid),
            gid: Gid(gid),
            name,
        }
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self::kernel()
    }
}
