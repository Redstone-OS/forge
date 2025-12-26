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
    /// Violação de política W^X (Write XOR Execute)
    WxViolation,
    /// Endereço inválido (não canônico ou fora de range)
    InvalidAddress,
    /// Endereço não alinhado a página
    NotAligned,
    /// Tamanho inválido (zero ou muito grande)
    InvalidSize,
    /// Operação não suportada para huge pages
    HugePageNotSupported,
    /// Índice fora dos limites
    OutOfBounds,
    /// Quota de memória excedida
    QuotaExceeded,
    /// Mapeamento falhou
    MappingFailed,
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
            Self::WxViolation => "Violação W^X: página RWX não permitida",
            Self::InvalidAddress => "Endereço inválido",
            Self::NotAligned => "Endereço não alinhado a página",
            Self::InvalidSize => "Tamanho inválido",
            Self::HugePageNotSupported => "Operação não suportada para huge pages",
            Self::OutOfBounds => "Índice fora dos limites",
            Self::QuotaExceeded => "Quota de memória excedida",
            Self::MappingFailed => "Mapeamento falhou",
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
