//! # Parser de Entradas de Diretório FAT
//!
//! Lê e interpreta entradas de diretório do filesystem FAT.
//!
//! ## Formato de Entrada (32 bytes)
//!
//! | Offset | Tamanho | Descrição                    |
//! |--------|---------|------------------------------|
//! | 0x00   | 8       | Nome do arquivo (8 chars)    |
//! | 0x08   | 3       | Extensão (3 chars)           |
//! | 0x0B   | 1       | Atributos                    |
//! | 0x14   | 2       | Cluster alto (FAT32)         |
//! | 0x1A   | 2       | Cluster baixo                |
//! | 0x1C   | 4       | Tamanho do arquivo           |

use alloc::string::String;
use alloc::vec::Vec;

/// Entrada de diretório FAT (32 bytes)
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Nome do arquivo (formato 8.3)
    pub name: String,
    /// Atributos do arquivo
    pub attr: FileAttr,
    /// Primeiro cluster (16 bits baixos)
    pub first_cluster_lo: u16,
    /// Primeiro cluster (16 bits altos, FAT32 apenas)
    pub first_cluster_hi: u16,
    /// Tamanho do arquivo em bytes
    pub size: u32,
}

/// Atributos de arquivo
#[derive(Debug, Clone, Copy)]
pub struct FileAttr(pub u8);

impl FileAttr {
    pub const SOMENTE_LEITURA: u8 = 0x01;
    pub const OCULTO: u8 = 0x02;
    pub const SISTEMA: u8 = 0x04;
    pub const VOLUME_ID: u8 = 0x08;
    pub const DIRETORIO: u8 = 0x10;
    pub const ARQUIVO: u8 = 0x20;
    pub const LFN: u8 = 0x0F; // Entrada de nome longo

    /// Verifica se é um diretório
    pub fn is_directory(&self) -> bool {
        (self.0 & Self::DIRETORIO) != 0
    }

    /// Verifica se é um volume ID
    pub fn is_volume_id(&self) -> bool {
        (self.0 & Self::VOLUME_ID) != 0
    }

    /// Verifica se é uma entrada de nome longo (LFN)
    pub fn is_lfn(&self) -> bool {
        (self.0 & Self::LFN) == Self::LFN
    }

    /// Verifica se está oculto
    pub fn is_hidden(&self) -> bool {
        (self.0 & Self::OCULTO) != 0
    }
}

impl DirEntry {
    /// Faz parse de uma entrada de diretório a partir de 32 bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 32 {
            return None;
        }

        // Fim do diretório
        if data[0] == 0x00 {
            return None;
        }

        // Entrada deletada
        if data[0] == 0xE5 {
            return None;
        }

        let attr = FileAttr(data[11]);

        // Pular entradas LFN por enquanto
        if attr.is_lfn() {
            return None;
        }

        // Pular volume ID
        if attr.is_volume_id() {
            return None;
        }

        // Parse do nome 8.3
        let name = parse_short_name(&data[0..11]);

        let first_cluster_lo = u16::from_le_bytes([data[26], data[27]]);
        let first_cluster_hi = u16::from_le_bytes([data[20], data[21]]);
        let size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

        Some(Self {
            name,
            attr,
            first_cluster_lo,
            first_cluster_hi,
            size,
        })
    }

    /// Retorna o número do primeiro cluster
    pub fn first_cluster(&self) -> u32 {
        ((self.first_cluster_hi as u32) << 16) | (self.first_cluster_lo as u32)
    }

    /// Verifica se é um diretório
    pub fn is_directory(&self) -> bool {
        self.attr.is_directory()
    }
}

/// Faz parse de um nome curto 8.3
fn parse_short_name(data: &[u8]) -> String {
    let mut name = String::new();

    // Parse do nome (primeiros 8 bytes)
    for i in 0..8 {
        let c = data[i];
        if c == b' ' {
            break;
        }
        // Tratamento especial para primeiro byte
        let c = if i == 0 && c == 0x05 { 0xE5 } else { c };
        name.push(c as char);
    }

    // Parse da extensão (últimos 3 bytes)
    let ext_start = 8;
    let mut has_ext = false;
    for i in 0..3 {
        let c = data[ext_start + i];
        if c != b' ' {
            if !has_ext {
                name.push('.');
                has_ext = true;
            }
            name.push(c as char);
        }
    }

    name
}

/// Faz parse de todas as entradas de um buffer de setor
pub fn parse_directory(data: &[u8]) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    let mut offset = 0;

    while offset + 32 <= data.len() {
        // Fim do diretório
        if data[offset] == 0x00 {
            break;
        }

        if let Some(entry) = DirEntry::parse(&data[offset..offset + 32]) {
            entries.push(entry);
        }

        offset += 32;
    }

    entries
}
