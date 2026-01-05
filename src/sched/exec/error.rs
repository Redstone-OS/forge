//! # Erros de Execução
//!
//! Tipos de erro para operações de spawn e carregamento de binários.

use crate::sys::KernelError;
use core::fmt;

/// Erro durante execução/spawn de processo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError {
    /// Arquivo não encontrado no VFS
    NotFound,

    /// Formato de binário inválido ou não reconhecido
    InvalidFormat,

    /// Binário não é ELF64 válido
    InvalidElf,

    /// Arquitetura não suportada (não é x86_64)
    UnsupportedArch,

    /// Tipo ELF não suportado (ex: PIE/DYN)
    UnsupportedType,

    /// Sem memória disponível para alocar
    OutOfMemory,

    /// Permissão negada para executar
    PermissionDenied,

    /// Falha ao criar Address Space
    AddressSpaceError,

    /// Falha ao mapear segmento
    MappingFailed,

    /// Header ELF fora dos limites do arquivo
    HeaderOutOfBounds,

    /// Segmento ELF fora dos limites do arquivo
    SegmentOutOfBounds,
}

impl ExecError {
    /// Retorna string descritiva do erro
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecError::NotFound => "file not found",
            ExecError::InvalidFormat => "invalid binary format",
            ExecError::InvalidElf => "invalid ELF binary",
            ExecError::UnsupportedArch => "unsupported architecture",
            ExecError::UnsupportedType => "unsupported ELF type (PIE not supported)",
            ExecError::OutOfMemory => "out of memory",
            ExecError::PermissionDenied => "permission denied",
            ExecError::AddressSpaceError => "failed to create address space",
            ExecError::MappingFailed => "failed to map segment",
            ExecError::HeaderOutOfBounds => "ELF header out of bounds",
            ExecError::SegmentOutOfBounds => "ELF segment out of bounds",
        }
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<ExecError> for KernelError {
    fn from(e: ExecError) -> Self {
        match e {
            ExecError::NotFound => KernelError::NotFound,
            ExecError::InvalidFormat
            | ExecError::InvalidElf
            | ExecError::UnsupportedArch
            | ExecError::UnsupportedType
            | ExecError::HeaderOutOfBounds
            | ExecError::SegmentOutOfBounds => KernelError::InvalidArgument,
            ExecError::OutOfMemory | ExecError::AddressSpaceError | ExecError::MappingFailed => {
                KernelError::OutOfMemory
            }
            ExecError::PermissionDenied => KernelError::PermissionDenied,
        }
    }
}
