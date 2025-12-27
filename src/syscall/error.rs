//! # Syscall Error Codes
//!
//! Códigos de erro retornados por syscalls.

/// Resultado de syscall
pub type SysResult<T> = Result<T, SysError>;

/// Erros de syscall
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SysError {
    /// Operação não implementada
    NotImplemented = -1,
    /// Syscall inválida
    InvalidSyscall = -2,
    /// Argumento inválido
    InvalidArgument = -3,
    /// Handle inválido
    InvalidHandle = -4,
    /// Permissão negada
    PermissionDenied = -5,
    /// Recurso não encontrado
    NotFound = -6,
    /// Recurso já existe
    AlreadyExists = -7,
    /// Recurso ocupado
    Busy = -8,
    /// Timeout
    Timeout = -9,
    /// Sem memória
    OutOfMemory = -10,
    /// Buffer muito pequeno
    BufferTooSmall = -11,
    /// Operação interrompida
    Interrupted = -12,
    /// Fim de arquivo
    EndOfFile = -13,
    /// Pipe quebrado
    BrokenPipe = -14,
    /// É um diretório
    IsDirectory = -15,
    /// Não é um diretório
    NotDirectory = -16,
    /// Diretório não vazio
    NotEmpty = -17,
    /// Erro de I/O
    IoError = -18,
    /// Limite atingido
    LimitReached = -19,
    /// Operação não suportada
    NotSupported = -20,
    /// Ponteiro inválido (bad address)
    BadAddress = -21,
}

impl SysError {
    /// Converte para isize (para retorno em RAX)
    pub fn as_isize(self) -> isize {
        self as i32 as isize
    }

    /// Converte de isize
    pub fn from_isize(val: isize) -> Option<Self> {
        match val as i32 {
            -1 => Some(Self::NotImplemented),
            -2 => Some(Self::InvalidSyscall),
            -3 => Some(Self::InvalidArgument),
            -4 => Some(Self::InvalidHandle),
            -5 => Some(Self::PermissionDenied),
            -6 => Some(Self::NotFound),
            -7 => Some(Self::AlreadyExists),
            -8 => Some(Self::Busy),
            -9 => Some(Self::Timeout),
            -10 => Some(Self::OutOfMemory),
            -11 => Some(Self::BufferTooSmall),
            -12 => Some(Self::Interrupted),
            -13 => Some(Self::EndOfFile),
            -14 => Some(Self::BrokenPipe),
            -15 => Some(Self::IsDirectory),
            -16 => Some(Self::NotDirectory),
            -17 => Some(Self::NotEmpty),
            -18 => Some(Self::IoError),
            -19 => Some(Self::LimitReached),
            -20 => Some(Self::NotSupported),
            -21 => Some(Self::BadAddress),
            _ => None,
        }
    }

    /// Nome do erro para debug
    pub fn name(&self) -> &'static str {
        match self {
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::InvalidSyscall => "INVALID_SYSCALL",
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::InvalidHandle => "INVALID_HANDLE",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::NotFound => "NOT_FOUND",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::Busy => "BUSY",
            Self::Timeout => "TIMEOUT",
            Self::OutOfMemory => "OUT_OF_MEMORY",
            Self::BufferTooSmall => "BUFFER_TOO_SMALL",
            Self::Interrupted => "INTERRUPTED",
            Self::EndOfFile => "END_OF_FILE",
            Self::BrokenPipe => "BROKEN_PIPE",
            Self::IsDirectory => "IS_DIRECTORY",
            Self::NotDirectory => "NOT_DIRECTORY",
            Self::NotEmpty => "NOT_EMPTY",
            Self::IoError => "IO_ERROR",
            Self::LimitReached => "LIMIT_REACHED",
            Self::NotSupported => "NOT_SUPPORTED",
            Self::BadAddress => "BAD_ADDRESS",
        }
    }
}
