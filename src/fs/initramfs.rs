//! Suporte a Initramfs (Formato USTAR).
//!
//! Carrega um arquivo TAR da memória e expõe como um sistema de arquivos read-only.
//! Fundamental para carregar o primeiro processo (init).

use super::vfs::{NodeType, VfsError, VfsHandle, VfsNode};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::str;

/// Representa o Initramfs montado na memória.
pub struct Initramfs {
    data: &'static [u8],
    files: Vec<Arc<TarFile>>,
}

impl Initramfs {
    /// Cria o FS a partir de um slice de memória contendo o TAR.
    pub fn new(data: &'static [u8]) -> Self {
        crate::kinfo!("Initramfs::new - criando struct...");
        let mut fs = Self {
            data,
            files: Vec::new(),
        };
        crate::kinfo!("Initramfs::new - struct OK, chamando parse...");
        fs.parse();
        crate::kinfo!("Initramfs::new - parse OK!");
        fs
    }

    fn parse(&mut self) {
        crate::kinfo!("parse: inicio, data.len={}", self.data.len());

        // Parsing simplificado de TAR (USTAR)
        let mut offset = 0;

        while offset + 512 <= self.data.len() {
            crate::kinfo!("parse: offset={}", offset);
            let header = &self.data[offset..offset + 512];

            // Verificar fim do arquivo (bloco de zeros)
            crate::kinfo!("parse: verificando zeros...");
            if header.iter().all(|&b| b == 0) {
                crate::kinfo!("parse: encontrou fim (zeros)");
                break;
            }

            crate::kinfo!("parse: lendo nome...");
            // Parse nome (offset 0, 100 bytes)
            let name = parse_null_term_str(&header[0..100]);
            crate::kinfo!("parse: nome={}", name);

            // Parse tamanho (offset 124, 12 bytes octal)
            crate::kinfo!("parse: lendo tamanho...");
            let size_str = parse_null_term_str(&header[124..136]);
            let size = u64::from_str_radix(size_str.trim(), 8).unwrap_or(0);
            crate::kinfo!("parse: tamanho={}", size);

            // Parse tipo (offset 156, 1 byte)
            let type_flag = header[156];
            let kind = match type_flag {
                b'5' => NodeType::Directory, // Directory
                _ => NodeType::File,         // Regular file (0 or \0)
            };

            // Calcular alinhamento para o próximo header
            let data_start = offset + 512;
            let data_end = data_start + size as usize;
            let next_header = (data_end + 511) & !511;

            if kind == NodeType::File {
                crate::kinfo!("parse: file_data slice...");
                let file_data = &self.data[data_start..data_end];
                crate::kinfo!("parse: slice OK len={}", file_data.len());

                // Teste: alocar Vec pequeno para ver se heap funciona
                crate::kinfo!("parse: teste Vec...");
                let test_vec: Vec<u8> = Vec::with_capacity(16);
                crate::kinfo!("parse: Vec OK");
                drop(test_vec);

                // Criar String manualmente (evita String::from que causa GPF)
                crate::kinfo!("parse: criando String manual...");
                let name_bytes = name.as_bytes();
                crate::kinfo!("parse: bytes len={}", name_bytes.len());
                let mut name_vec: Vec<u8> = Vec::with_capacity(name_bytes.len());
                crate::kinfo!("parse: Vec alocado");
                for &b in name_bytes {
                    name_vec.push(b);
                }
                crate::kinfo!("parse: bytes copiados");
                let name_str = unsafe { String::from_utf8_unchecked(name_vec) };
                crate::kinfo!("parse: String OK, len={}", name_str.len());

                crate::kinfo!("parse: Arc::new(TarFile)...");
                let tar_file = Arc::new(TarFile {
                    name: name_str,
                    size,
                    data: file_data,
                });
                crate::kinfo!("parse: Arc OK");

                crate::kinfo!("parse: push...");
                self.files.push(tar_file);
                crate::kinfo!("parse: TarFile OK");
            }

            offset = next_header;
        }

        crate::kinfo!(
            "initfs analisado: {} arquivos encontrados.",
            self.files.len()
        );
    }
}

// Implementação do VfsNode para a raiz do Initramfs
impl VfsNode for Initramfs {
    fn name(&self) -> &str {
        "/"
    }
    fn kind(&self) -> NodeType {
        NodeType::Directory
    }
    fn size(&self) -> u64 {
        0
    }

    fn list(&self) -> Result<Vec<Arc<dyn VfsNode>>, VfsError> {
        // Retorna todos os arquivos (flat structure por enquanto)
        // TODO: Implementar hierarquia real de diretórios.
        let mut nodes = Vec::new();
        for f in &self.files {
            nodes.push(f.clone() as Arc<dyn VfsNode>);
        }
        Ok(nodes)
    }
}

struct TarFile {
    name: String,
    size: u64,
    data: &'static [u8],
}

impl VfsNode for TarFile {
    fn name(&self) -> &str {
        &self.name
    }
    fn kind(&self) -> NodeType {
        NodeType::File
    }
    fn size(&self) -> u64 {
        self.size
    }

    fn open(&self) -> Result<Arc<dyn VfsHandle>, VfsError> {
        Ok(Arc::new(TarFileHandle { data: self.data }))
    }
}

struct TarFileHandle {
    data: &'static [u8],
}

impl VfsHandle for TarFileHandle {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, VfsError> {
        let offset = offset as usize;
        if offset >= self.data.len() {
            return Ok(0);
        }

        let available = self.data.len() - offset;
        let to_read = core::cmp::min(available, buf.len());

        // Cópia manual byte a byte (evitar copy_from_slice que usa memcpy e causa GPF)
        for i in 0..to_read {
            buf[i] = self.data[offset + i];
        }
        Ok(to_read)
    }

    fn write(&self, _buf: &[u8], _offset: u64) -> Result<usize, VfsError> {
        Err(VfsError::PermissionDenied) // Read-only
    }
}

fn parse_null_term_str(bytes: &[u8]) -> &str {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    str::from_utf8(&bytes[..len]).unwrap_or("")
}
