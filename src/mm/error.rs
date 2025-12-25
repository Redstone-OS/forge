//! Tipos de Erro do Subsistema de Memória
//!
//! Define erros estruturados para diagnóstico preciso de falhas em MM.

/// Erros do subsistema de memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmError {
    /// Sem memória física disponível (OOM)
    OutOfMemory,
    /// Scratch slot não inicializado ou indisponível
    ScratchNotReady,
    /// Falha ao dividir huge page em páginas 4K
    HugeSplitFailed,
    /// Endereço não alinhado corretamente
    InvalidAlignment,
    /// Região já mapeada
    AlreadyMapped,
    /// Região não mapeada
    NotMapped,
    /// Parâmetro inválido
    InvalidParameter,
    /// Falha na inicialização
    InitFailed,
    /// Double free detectado
    DoubleFree,
}

impl MmError {
    /// Retorna descrição legível do erro
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OutOfMemory => "OOM: sem frames físicos disponíveis",
            Self::ScratchNotReady => "Scratch slot não inicializado",
            Self::HugeSplitFailed => "Falha ao dividir huge page",
            Self::InvalidAlignment => "Endereço não alinhado",
            Self::AlreadyMapped => "Região já mapeada",
            Self::NotMapped => "Região não mapeada",
            Self::InvalidParameter => "Parâmetro inválido",
            Self::InitFailed => "Falha na inicialização",
            Self::DoubleFree => "Double free detectado",
        }
    }
}

impl core::fmt::Display for MmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Tipo Result específico para operações de memória
pub type MmResult<T> = Result<T, MmError>;
