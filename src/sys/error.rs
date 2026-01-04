//! # Códigos de Erro do Kernel
//!
//! Define erros internos usados entre subsistemas do kernel.
//!
//! ## Diferença de SysError
//!
//! - `KernelError`: Usado **dentro** do kernel
//! - `SysError`: Usado na **borda** da syscall (retorno para userspace)
//!
//! Geralmente `KernelError` é convertido para `SysError` antes de retornar.

/// Erro genérico do kernel.
///
/// Representa falhas internas entre subsistemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum KernelError {
    /// Sucesso (não é erro).
    Success = 0,

    /// Permissão negada (sem capability necessária).
    PermissionDenied = -1,

    /// Recurso não encontrado.
    NotFound = -2,

    /// Recurso já existe.
    AlreadyExists = -3,

    /// Sem memória disponível.
    OutOfMemory = -4,

    /// Argumento inválido.
    InvalidArgument = -5,

    /// Operação não suportada.
    NotSupported = -6,

    /// Recurso ocupado.
    Busy = -7,

    /// Operação expirou.
    Timeout = -8,

    /// Handle inválido ou expirado.
    InvalidHandle = -9,

    /// Buffer muito pequeno.
    BufferTooSmall = -10,

    /// Fim de arquivo/stream.
    EndOfFile = -11,

    /// Erro de I/O (hardware/driver).
    IoError = -12,

    /// Operação interrompida.
    Interrupted = -13,

    /// Tente novamente (EAGAIN).
    Again = -14,

    /// Operação cancelada.
    Cancelled = -15,

    /// Conexão recusada.
    ConnectionRefused = -16,

    /// Connexão resetada.
    ConnectionReset = -17,

    /// Deadlock detectado.
    Deadlock = -18,

    /// Erro interno (bug ou estado inconsistente).
    Internal = -99,
}

/// Result type do kernel.
pub type KernelResult<T> = Result<T, KernelError>;

impl KernelError {
    /// Converte para código numérico.
    #[inline]
    pub const fn as_code(self) -> i32 {
        self as i32
    }

    /// Cria a partir de código numérico.
    #[inline]
    pub const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Success,
            -1 => Self::PermissionDenied,
            -2 => Self::NotFound,
            -3 => Self::AlreadyExists,
            -4 => Self::OutOfMemory,
            -5 => Self::InvalidArgument,
            -6 => Self::NotSupported,
            -7 => Self::Busy,
            -8 => Self::Timeout,
            -9 => Self::InvalidHandle,
            -10 => Self::BufferTooSmall,
            -11 => Self::EndOfFile,
            -12 => Self::IoError,
            -13 => Self::Interrupted,
            -14 => Self::Again,
            -15 => Self::Cancelled,
            -16 => Self::ConnectionRefused,
            -17 => Self::ConnectionReset,
            -18 => Self::Deadlock,
            _ => Self::Internal,
        }
    }

    /// Retorna nome do erro.
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::PermissionDenied => "PermissionDenied",
            Self::NotFound => "NotFound",
            Self::AlreadyExists => "AlreadyExists",
            Self::OutOfMemory => "OutOfMemory",
            Self::InvalidArgument => "InvalidArgument",
            Self::NotSupported => "NotSupported",
            Self::Busy => "Busy",
            Self::Timeout => "Timeout",
            Self::InvalidHandle => "InvalidHandle",
            Self::BufferTooSmall => "BufferTooSmall",
            Self::EndOfFile => "EndOfFile",
            Self::IoError => "IoError",
            Self::Interrupted => "Interrupted",
            Self::Again => "Again",
            Self::Cancelled => "Cancelled",
            Self::ConnectionRefused => "ConnectionRefused",
            Self::ConnectionReset => "ConnectionReset",
            Self::Deadlock => "Deadlock",
            Self::Internal => "Internal",
        }
    }

    /// Verifica se é sucesso.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Verifica se é erro.
    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Verifica se é erro recuperável (pode tentar novamente).
    #[inline]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Again | Self::Busy | Self::Interrupted | Self::Timeout
        )
    }
}

impl core::fmt::Display for KernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({})", self.name(), self.as_code())
    }
}

impl Default for KernelError {
    fn default() -> Self {
        Self::Success
    }
}
