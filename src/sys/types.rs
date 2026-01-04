//! # Tipos Fundamentais do Sistema
//!
//! Define **NewTypes** para identificadores do sistema.
//! Garante segurança de tipos em tempo de compilação.
//!
//! ## Por que NewTypes?
//!
//! ```text
//! // SEM NewType - pode trocar argumentos por engano
//! fn kill(pid: u32, signal: u32) { ... }
//!
//! // COM NewType - erro de compilação se trocar
//! fn kill(pid: Pid, signal: Signal) { ... }
//! ```

// =============================================================================
// PROCESS ID
// =============================================================================

/// Identificador único de processo.
///
/// ## Constantes
/// - `KERNEL` (0): Processo do kernel
/// - `INIT` (1): Primeiro processo de userspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Pid(pub u32);

impl Pid {
    /// PID do kernel (0).
    pub const KERNEL: Self = Self(0);

    /// PID do processo init (1).
    pub const INIT: Self = Self(1);

    /// Cria novo PID.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Retorna valor como u32.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Verifica se é o kernel.
    #[inline]
    pub const fn is_kernel(self) -> bool {
        self.0 == 0
    }

    /// Verifica se é init.
    #[inline]
    pub const fn is_init(self) -> bool {
        self.0 == 1
    }

    /// Próximo PID (saturating).
    #[inline]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl core::fmt::Display for Pid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PID({})", self.0)
    }
}

// =============================================================================
// THREAD ID
// =============================================================================

/// Identificador único de thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Tid(pub u32);

impl Tid {
    /// TID do kernel (0).
    pub const KERNEL: Self = Self(0);

    /// Cria novo TID.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Retorna valor como u32.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Próximo TID (saturating).
    #[inline]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl core::fmt::Display for Tid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TID({})", self.0)
    }
}

// =============================================================================
// USER ID
// =============================================================================

/// Identificador de usuário.
///
/// **Nota**: No modelo OCAP do RedstoneOS, UID é usado apenas
/// para auditoria/logging, NÃO para controle de acesso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Uid(pub u32);

impl Uid {
    /// UID do kernel (0).
    pub const KERNEL: Self = Self(0);

    /// Cria novo UID.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Retorna valor como u32.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Verifica se é o kernel.
    #[inline]
    pub const fn is_kernel(self) -> bool {
        self.0 == 0
    }
}

impl core::fmt::Display for Uid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UID({})", self.0)
    }
}

// =============================================================================
// GROUP ID
// =============================================================================

/// Identificador de grupo.
///
/// **Nota**: No modelo OCAP do RedstoneOS, GID é usado apenas
/// para auditoria/logging, NÃO para controle de acesso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Gid(pub u32);

impl Gid {
    /// GID do kernel (0).
    pub const KERNEL: Self = Self(0);

    /// Cria novo GID.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Retorna valor como u32.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for Gid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "GID({})", self.0)
    }
}

// =============================================================================
// TYPE ALIASES
// =============================================================================

/// Offset em arquivo (pode ser negativo para seek).
pub type FileOffset = i64;

/// Tamanho em bytes.
pub type Size = usize;

/// Timestamp (segundos desde epoch).
pub type Time = i64;

/// Resultado de operação que pode falhar.
pub type SysResult<T> = Result<T, super::error::KernelError>;
