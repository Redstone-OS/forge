//! Memory Management Errors
//!
//! Define os erros possíveis durante operações de memória.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmError {
    /// Erro genérico
    General,
    /// Out of Memory (OOM)
    OutOfMemory,
    /// Endereço inválido ou não alinhado
    InvalidAddress,
    /// Tamanho inválido
    InvalidSize,
    /// Frame já liberado ou não alocado
    FrameNotAllocated,
    /// Página já mapeada
    AlreadyMapped,
    /// Página não mapeada
    NotMapped,
    /// Permissões inválidas
    InvalidFlags,
    /// Acesso fora dos limites
    OutOfBounds,
    /// Erro de IO (swap)
    IoError,
}

pub type MmResult<T> = Result<T, MmError>;
