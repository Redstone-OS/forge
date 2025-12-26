//! Suporte a Initramfs (Formato USTAR).
//!
//! Carrega um arquivo TAR da memória e expõe como um sistema de arquivos read-only.
//! Fundamental para carregar o primeiro processo (init).

use super::vfs::{NodeType, VfsError, VfsHandle, VfsNode};
use alloc::string::String;
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
        crate::kdebug!("(Initramfs) Parsing arquivo TAR. bytes=", data.len() as u64);
        crate::ktrace!("(Initramfs) [1] Criando Vec...");

        // Criar Vec vazio primeiro, sem pré-alocação para evitar SSE
        let files: Vec<Arc<TarFile>> = Vec::new();

        crate::ktrace!("(Initramfs) [2] Vec criado, construindo struct...");

        let mut fs = Self { data, files };

        crate::ktrace!("(Initramfs) [3] Struct pronta, chamando parse...");
        fs.parse();
        crate::ktrace!("(Initramfs) [4] Parse concluído");
        fs
    }

    fn parse(&mut self) {
        crate::ktrace!("(Initramfs) parse: Iniciando...");

        // Parsing simplificado de TAR (USTAR)
        let mut offset = 0usize;

        while offset + 512 <= self.data.len() {
            let header = &self.data[offset..offset + 512];

            // Verificar fim do arquivo (bloco de zeros) usando while manual
            let mut all_zero = true;
            let mut check_idx = 0usize;
            while check_idx < 512 {
                if header[check_idx] != 0 {
                    all_zero = false;
                    break;
                }
                check_idx += 1;
            }

            if all_zero {
                crate::ktrace!("(Initramfs) parse: EOD (End of Data) encontrado");
                break;
            }

            // Parse nome (offset 0, 100 bytes)
            let name = parse_null_term_str(&header[0..100]);

            // Parse tamanho (offset 124, 12 bytes octal)
            let size_str = parse_null_term_str(&header[124..136]);
            let size = parse_octal(size_str);

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
                crate::ktrace!("(Initramfs) parse: [A] arquivo encontrado");
                let file_data = &self.data[data_start..data_end];

                crate::ktrace!("(Initramfs) parse: [B1] name.as_bytes()...");
                let name_bytes = name.as_bytes();

                crate::ktrace!("(Initramfs) parse: [B2] Vec::new()...");
                let mut name_vec: Vec<u8> = Vec::new();

                crate::ktrace!("(Initramfs) parse: [B3] loop push...");
                let mut i = 0usize;
                while i < name_bytes.len() {
                    name_vec.push(name_bytes[i]);
                    i += 1;
                }
                crate::ktrace!("(Initramfs) parse: [B4] loop concluído");

                crate::ktrace!("(Initramfs) parse: [C] convertendo para String...");
                let name_str = unsafe { String::from_utf8_unchecked(name_vec) };

                crate::ktrace!("(Initramfs) parse: [D] criando Arc<TarFile>...");
                let tar_file = Arc::new(TarFile {
                    name: name_str,
                    size,
                    data: file_data,
                });

                crate::ktrace!("(Initramfs) parse: [E] push para files...");
                self.files.push(tar_file);
                crate::ktrace!("(Initramfs) parse: [F] arquivo processado OK");
            }

            offset = next_header;
        }

        crate::kinfo!(
            "(Initramfs) Pronto. Objetos carregados=",
            self.files.len() as u64
        );
    }
}

/// Parse octal string manualmente para evitar from_str_radix que pode gerar SSE
fn parse_octal(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut result = 0u64;
    let mut i = 0usize;

    // Pular espaços iniciais
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'0') {
        i += 1;
    }

    while i < bytes.len() {
        let b = bytes[i];
        if b >= b'0' && b <= b'7' {
            result = result * 8 + (b - b'0') as u64;
        } else {
            break;
        }
        i += 1;
    }

    result
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
        // Pré-alocar para evitar realocação que causa crash
        let mut nodes = Vec::with_capacity(self.files.len());
        // Usar while ao invés de for para evitar SSE
        let mut i = 0usize;
        while i < self.files.len() {
            nodes.push(self.files[i].clone() as Arc<dyn VfsNode>);
            i += 1;
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

        // Cópia manual byte a byte usando while
        let mut i = 0usize;
        while i < to_read {
            buf[i] = self.data[offset + i];
            i += 1;
        }
        Ok(to_read)
    }

    fn write(&self, _buf: &[u8], _offset: u64) -> Result<usize, VfsError> {
        Err(VfsError::PermissionDenied) // Read-only
    }
}

/// Parse null-terminated string usando while manual
fn parse_null_term_str(bytes: &[u8]) -> &str {
    // Encontrar posição do null usando while
    let mut len = 0usize;
    while len < bytes.len() && bytes[len] != 0 {
        len += 1;
    }

    // Converter para str (assumindo UTF-8 válido)
    match str::from_utf8(&bytes[..len]) {
        Ok(s) => s,
        Err(_) => "",
    }
}
