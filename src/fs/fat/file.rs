//! # Operações de Arquivo FAT
//!
//! Leitura de arquivos do filesystem FAT.
//!
//! ## Exemplo de Uso
//!
//! ```ignore
//! let fs = FatFs::mount(device)?;
//! let file = FatFile::new(&fs, entry);
//! let data = file.read_all()?;
//! ```

use super::dir::DirEntry;
use super::fs::FatFs;
use crate::fs::vfs::inode::FsError;
use alloc::vec::Vec;

/// Handle de arquivo para leitura de FAT
pub struct FatFile<'a> {
    /// Referência ao filesystem
    fs: &'a FatFs,
    /// Entrada de diretório
    entry: DirEntry,
    /// Posição atual no arquivo
    _position: u64,
}

impl<'a> FatFile<'a> {
    /// Cria um novo handle de arquivo
    pub fn new(fs: &'a FatFs, entry: DirEntry) -> Self {
        Self {
            fs,
            entry,
            _position: 0,
        }
    }

    /// Retorna o tamanho do arquivo
    pub fn size(&self) -> u64 {
        self.entry.size as u64
    }

    /// Lê o arquivo inteiro para um buffer
    pub fn read_all(&self) -> Result<Vec<u8>, FsError> {
        let size = self.entry.size as usize;
        if size == 0 {
            return Ok(Vec::new());
        }

        let mut data = Vec::with_capacity(size);
        let cluster_size = self.fs.cluster_size();
        let mut cluster_buf = alloc::vec![0u8; cluster_size];

        let mut cluster = self.entry.first_cluster();
        let mut remaining = size;

        while remaining > 0 && cluster >= 2 {
            self.fs.read_cluster(cluster, &mut cluster_buf)?;

            let to_copy = remaining.min(cluster_size);
            data.extend_from_slice(&cluster_buf[..to_copy]);
            remaining -= to_copy;

            // Obter próximo cluster
            match self.fs.next_cluster(cluster) {
                Some(next) => cluster = next,
                None => break,
            }
        }

        Ok(data)
    }

    /// Lê bytes a partir de um offset específico
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let size = self.entry.size as u64;
        if offset >= size {
            return Ok(0);
        }

        let cluster_size = self.fs.cluster_size() as u64;
        let mut cluster_buf = alloc::vec![0u8; cluster_size as usize];

        // Encontrar cluster inicial
        let start_cluster_idx = offset / cluster_size;
        let offset_in_cluster = (offset % cluster_size) as usize;

        let mut cluster = self.entry.first_cluster();
        for _ in 0..start_cluster_idx {
            match self.fs.next_cluster(cluster) {
                Some(next) => cluster = next,
                None => return Ok(0),
            }
        }

        // Ler o cluster
        self.fs.read_cluster(cluster, &mut cluster_buf)?;

        // Copiar dados para o buffer
        let available = (cluster_size as usize) - offset_in_cluster;
        let remaining_in_file = (size - offset) as usize;
        let to_read = buf.len().min(available).min(remaining_in_file);

        buf[..to_read]
            .copy_from_slice(&cluster_buf[offset_in_cluster..offset_in_cluster + to_read]);

        Ok(to_read)
    }
}
