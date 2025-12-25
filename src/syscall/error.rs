//! Códigos de Erro do Redstone OS
//!
//! Sistema de erros unificado para todas as syscalls.
//! Erros são retornados como valores negativos em RAX.

/// Enum de erros do sistema.
///
/// Valores são i32 para permitir representação negativa em isize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SysError {
    // === Erros Gerais (1-15) ===
    /// Operação não permitida
    PermissionDenied = 1,
    /// Objeto não encontrado
    NotFound = 2,
    /// Objeto já existe
    AlreadyExists = 3,
    /// Argumento inválido
    InvalidArgument = 4,
    /// Operação bloquearia (non-blocking mode)
    WouldBlock = 5,
    /// Operação interrompida
    Interrupted = 6,
    /// Timeout expirado
    TimedOut = 7,
    /// Recurso ocupado
    Busy = 8,

    // === Erros de Handle (16-31) ===
    /// Handle inválido ou fechado
    BadHandle = 16,
    /// Tipo de objeto incompatível com operação
    HandleTypeMismatch = 17,
    /// Direitos insuficientes no handle
    InsufficientRights = 18,
    /// Tabela de handles cheia
    HandleTableFull = 19,

    // === Erros de Memória (32-47) ===
    /// Sem memória disponível
    OutOfMemory = 32,
    /// Endereço inválido ou não mapeado
    BadAddress = 33,
    /// Região de memória em uso
    AddressInUse = 34,
    /// Alinhamento incorreto
    BadAlignment = 35,

    // === Erros de IO (48-63) ===
    /// Erro genérico de IO
    IoError = 48,
    /// Fim de arquivo/stream
    EndOfFile = 49,
    /// Conexão/Pipe quebrado
    BrokenPipe = 50,

    // === Erros de IPC (64-79) ===
    /// Fila da porta cheia
    PortFull = 64,
    /// Porta fechada
    PortClosed = 65,
    /// Mensagem excede tamanho máximo
    MessageTooLarge = 66,
    /// Nenhuma mensagem disponível
    NoMessage = 67,

    // === Erros de Processo (80-95) ===
    /// Processo não encontrado
    ProcessNotFound = 80,
    /// Limite de processos atingido
    TooManyProcesses = 81,

    // === Erros de Sistema (240-255) ===
    /// Syscall não implementada
    NotImplemented = 254,
    /// Erro desconhecido
    Unknown = 255,
}

impl SysError {
    /// Converte para isize negativo (formato de retorno da syscall)
    #[inline]
    pub fn as_isize(self) -> isize {
        -(self as i32 as isize)
    }

    /// Cria erro a partir de código negativo
    pub fn from_code(code: isize) -> Option<Self> {
        if code >= 0 {
            return None;
        }
        let abs = (-code) as i32;
        match abs {
            1 => Some(Self::PermissionDenied),
            2 => Some(Self::NotFound),
            3 => Some(Self::AlreadyExists),
            4 => Some(Self::InvalidArgument),
            5 => Some(Self::WouldBlock),
            6 => Some(Self::Interrupted),
            7 => Some(Self::TimedOut),
            8 => Some(Self::Busy),
            16 => Some(Self::BadHandle),
            17 => Some(Self::HandleTypeMismatch),
            18 => Some(Self::InsufficientRights),
            19 => Some(Self::HandleTableFull),
            32 => Some(Self::OutOfMemory),
            33 => Some(Self::BadAddress),
            34 => Some(Self::AddressInUse),
            35 => Some(Self::BadAlignment),
            48 => Some(Self::IoError),
            49 => Some(Self::EndOfFile),
            50 => Some(Self::BrokenPipe),
            64 => Some(Self::PortFull),
            65 => Some(Self::PortClosed),
            66 => Some(Self::MessageTooLarge),
            67 => Some(Self::NoMessage),
            80 => Some(Self::ProcessNotFound),
            81 => Some(Self::TooManyProcesses),
            254 => Some(Self::NotImplemented),
            255 => Some(Self::Unknown),
            _ => Some(Self::Unknown),
        }
    }
}

/// Resultado de syscall: Ok(valor) ou Err(SysError)
pub type SysResult<T> = Result<T, SysError>;

/// Helper para converter SysResult<usize> em isize para retorno
pub fn result_to_isize(result: SysResult<usize>) -> isize {
    match result {
        Ok(val) => val as isize,
        Err(e) => e.as_isize(),
    }
}
