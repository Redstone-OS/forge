//! # Tipos de Erro do RMM
//!
//! Define os tipos de erro que podem ocorrer nas operações de memória.
//! Todos os erros são recoverable (não causam panic), exceto em debug mode
//! para owner mismatch.
//!
//! ## Uso
//!
//! ```rust
//! use crate::rmm::{RmmError, RmmResult};
//!
//! fn allocate_something() -> RmmResult<PhysAddr> {
//!     let frame = phys::alloc(...).ok_or(RmmError::OutOfMemory)?;
//!     Ok(frame)
//! }
//! ```

use core::fmt;

/// Erros que podem ocorrer no RMM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RmmError {
    /// Sem memória disponível (OOM)
    OutOfMemory,

    /// Endereço físico inválido (fora do range gerenciado)
    InvalidPhysAddr,

    /// Endereço inválido (alias para InvalidPhysAddr)
    InvalidAddress,

    /// Endereço virtual inválido
    InvalidVirtAddr,

    /// Endereço não alinhado
    NotAligned,

    /// Página já está mapeada
    AlreadyMapped,

    /// Página não está mapeada
    NotMapped,

    /// Owner do frame não confere
    OwnerMismatch,

    /// Zona de memória inválida para a operação
    InvalidZone,

    /// Frames contíguos não disponíveis
    NoContiguousFrames,

    /// Reference count overflow
    RefCountOverflow,

    /// Reference count underflow (tentou decrementar 0)
    RefCountUnderflow,

    /// Frame está locked (não pode ser liberado)
    FrameLocked,

    /// VMA overlaps com região existente
    VmaOverlap,

    /// VMA não encontrada
    VmaNotFound,

    /// Operação não suportada
    NotSupported,

    /// Permissão negada
    PermissionDenied,

    /// Table allocation falhou
    TableAllocationFailed,

    /// IOMMU não disponível
    NoIommu,

    /// DMA buffer muito grande
    DmaBufferTooLarge,

    /// HHDM não inicializado
    HhdmNotInitialized,

    /// RMM não inicializado
    NotInitialized,

    /// Erro genérico interno
    Internal,
}

impl RmmError {
    /// Retorna uma string descritiva do erro
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OutOfMemory => "out of memory",
            Self::InvalidPhysAddr | Self::InvalidAddress => "invalid physical address",
            Self::InvalidVirtAddr => "invalid virtual address",
            Self::NotAligned => "address not aligned",
            Self::AlreadyMapped => "page already mapped",
            Self::NotMapped => "page not mapped",
            Self::OwnerMismatch => "frame owner mismatch",
            Self::InvalidZone => "invalid memory zone",
            Self::NoContiguousFrames => "no contiguous frames available",
            Self::RefCountOverflow => "reference count overflow",
            Self::RefCountUnderflow => "reference count underflow",
            Self::FrameLocked => "frame is locked",
            Self::VmaOverlap => "VMA overlap",
            Self::VmaNotFound => "VMA not found",
            Self::NotSupported => "operation not supported",
            Self::PermissionDenied => "permission denied",
            Self::TableAllocationFailed => "page table allocation failed",
            Self::NoIommu => "IOMMU not available",
            Self::DmaBufferTooLarge => "DMA buffer too large",
            Self::HhdmNotInitialized => "HHDM not initialized",
            Self::NotInitialized => "RMM not initialized",
            Self::Internal => "internal error",
        }
    }

    /// Retorna true se o erro é recuperável (pode tentar novamente)
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::OutOfMemory
            | Self::NoContiguousFrames
            | Self::AlreadyMapped
            | Self::FrameLocked => true,
            _ => false,
        }
    }

    /// Retorna true se o erro indica bug no caller
    pub const fn is_bug(&self) -> bool {
        match self {
            Self::OwnerMismatch
            | Self::RefCountUnderflow
            | Self::InvalidPhysAddr
            | Self::InvalidVirtAddr
            | Self::NotAligned => true,
            _ => false,
        }
    }
}

impl fmt::Display for RmmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RmmError: {}", self.as_str())
    }
}

/// Result type para operações do RMM
pub type RmmResult<T> = Result<T, RmmError>;
