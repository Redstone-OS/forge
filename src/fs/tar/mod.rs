//! TAR Archive Parser
//!
//! Parser simples para arquivos TAR (POSIX ustar format)
//! Usado para ler initramfs durante boot

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// TAR Archive
pub struct TarArchive {
    data: &'static [u8],
}

/// TAR Entry (arquivo ou diretório)
#[derive(Debug, Clone)]
pub struct TarEntry {
    pub name: String,
    pub size: usize,
    pub offset: usize,
    pub is_dir: bool,
}

/// TAR Header (512 bytes)
#[repr(C, packed)]
struct TarHeader {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    checksum: [u8; 8],
    typeflag: u8,
    linkname: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    uname: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    prefix: [u8; 155],
    _padding: [u8; 12],
}

const TAR_BLOCK_SIZE: usize = 512;

impl TarArchive {
    /// Cria novo TAR archive a partir de dados
    pub fn new(data: &'static [u8]) -> Result<Self, &'static str> {
        if data.len() < TAR_BLOCK_SIZE {
            return Err("TAR muito pequeno");
        }

        Ok(Self { data })
    }

    /// Itera sobre todas as entradas do TAR
    pub fn entries(&self) -> TarIterator {
        TarIterator {
            data: self.data,
            offset: 0,
        }
    }

    /// Busca arquivo específico no TAR
    pub fn find(&self, path: &str) -> Option<TarEntry> {
        for entry in self.entries() {
            if entry.name == path {
                return Some(entry);
            }
        }
        None
    }

    /// Lê dados de uma entrada
    pub fn read_entry(&self, entry: &TarEntry) -> &[u8] {
        let start = entry.offset;
        let end = start + entry.size;
        &self.data[start..end]
    }

    /// Lista todos os arquivos em um diretório
    pub fn readdir(&self, path: &str) -> Vec<String> {
        let mut results = Vec::new();
        let prefix = if path == "/" {
            String::new()
        } else {
            let mut p = String::from(path);
            if !p.ends_with('/') {
                p.push('/');
            }
            p
        };

        for entry in self.entries() {
            if entry.name.starts_with(&prefix) {
                let relative = &entry.name[prefix.len()..];
                // Apenas arquivos diretos (não subdirs)
                if !relative.contains('/') && !relative.is_empty() {
                    results.push(entry.name.clone());
                }
            }
        }

        results
    }
}

/// Iterator sobre entradas TAR
pub struct TarIterator {
    data: &'static [u8],
    offset: usize,
}

impl Iterator for TarIterator {
    type Item = TarEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // Fim do arquivo (dois blocos vazios)
        if self.offset + TAR_BLOCK_SIZE * 2 > self.data.len() {
            return None;
        }

        // Ler header
        let header_bytes = &self.data[self.offset..self.offset + TAR_BLOCK_SIZE];

        // Verificar se é bloco vazio (fim do TAR)
        if header_bytes.iter().all(|&b| b == 0) {
            return None;
        }

        let header = unsafe { &*(header_bytes.as_ptr() as *const TarHeader) };

        // Parse nome
        let name = parse_string(&header.name);
        if name.is_empty() {
            return None;
        }

        // Parse tamanho (octal)
        let size = parse_octal(&header.size);

        // Tipo de arquivo
        let is_dir = header.typeflag == b'5' || name.ends_with('/');

        // Offset dos dados (após o header)
        let data_offset = self.offset + TAR_BLOCK_SIZE;

        // Próximo header (alinhado em 512 bytes)
        let blocks = (size + TAR_BLOCK_SIZE - 1) / TAR_BLOCK_SIZE;
        self.offset += TAR_BLOCK_SIZE + (blocks * TAR_BLOCK_SIZE);

        Some(TarEntry {
            name,
            size,
            offset: data_offset,
            is_dir,
        })
    }
}

/// Parse string C-style (null-terminated)
fn parse_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// Parse número octal
fn parse_octal(bytes: &[u8]) -> usize {
    let s = parse_string(bytes);
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }

    usize::from_str_radix(s, 8).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_octal() {
        assert_eq!(parse_octal(b"000644 \0"), 0o644);
        assert_eq!(parse_octal(b"0001750\0"), 0o1750);
    }
}
