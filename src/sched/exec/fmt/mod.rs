//! # Formatos de Executáveis
//!
//! Detecção e parsing de formatos binários suportados.
//!
//! ## Formatos Suportados
//!
//! - **ELF64**: Executáveis estáticos x86_64
//! - **Scripts**: Detecção de shebang (stub)

pub mod elf;
mod script;

pub use script::detect_shebang;

use super::error::ExecError;

/// Tipos de formato binário reconhecidos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// ELF64 executável estático
    Elf64,
    /// Script com shebang
    Script,
    /// Formato desconhecido
    Unknown,
}

/// Detecta o formato de um binário a partir dos seus bytes iniciais
///
/// # Argumentos
///
/// * `data` - Primeiros bytes do arquivo
///
/// # Retorna
///
/// O formato detectado ou `Unknown` se não reconhecido.
pub fn detect_format(data: &[u8]) -> BinaryFormat {
    if data.len() < 4 {
        return BinaryFormat::Unknown;
    }

    // ELF magic: 0x7F 'E' 'L' 'F'
    if &data[0..4] == b"\x7fELF" {
        return BinaryFormat::Elf64;
    }

    // Shebang: '#' '!'
    if data.len() >= 2 && &data[0..2] == b"#!" {
        return BinaryFormat::Script;
    }

    BinaryFormat::Unknown
}

/// Valida se o formato é executável
pub fn validate_format(format: BinaryFormat) -> Result<(), ExecError> {
    match format {
        BinaryFormat::Elf64 => Ok(()),
        BinaryFormat::Script => Err(ExecError::UnsupportedType), // TODO: Implementar
        BinaryFormat::Unknown => Err(ExecError::InvalidFormat),
    }
}
