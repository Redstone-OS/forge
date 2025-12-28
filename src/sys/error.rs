//! Códigos de erro do kernel

/// Erro genérico do kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum KernelError {
    /// Sucesso (não é erro)
    Success = 0,
    /// Permissão negada
    PermissionDenied = -1,
    /// Não encontrado
    NotFound = -2,
    /// Já existe
    AlreadyExists = -3,
    /// Sem memória
    OutOfMemory = -4,
    /// Argumento inválido
    InvalidArgument = -5,
    /// Operação não suportada
    NotSupported = -6,
    /// Recurso ocupado
    Busy = -7,
    /// Timeout
    Timeout = -8,
    /// Handle inválido
    InvalidHandle = -9,
    /// Buffer muito pequeno
    BufferTooSmall = -10,
    /// Fim de arquivo
    EndOfFile = -11,
    /// IO Error
    IoError = -12,
    /// Interrompido
    Interrupted = -13,
    /// Novamente (tente de novo)
    Again = -14,
    /// Operação cancelada
    Cancelled = -15,
    /// Erro interno
    Internal = -99,
}

/// Result type do kernel
pub type KernelResult<T> = Result<T, KernelError>;

impl KernelError {
    /// Converte para código numérico
    pub const fn as_code(self) -> i32 {
        self as i32
    }
    
    /// Cria a partir de código
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
            _ => Self::Internal,
        }
    }
}
