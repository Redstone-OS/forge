//! TarFS - Filesystem baseado em TAR
//!
//! Implementa filesystem read-only sobre arquivo TAR

#![allow(dead_code)]

extern crate alloc;

use super::tar::{TarArchive, TarEntry};
use alloc::string::String;
use alloc::vec::Vec;

/// TarFS - Filesystem TAR
pub struct TarFS {
    archive: TarArchive,
}

impl TarFS {
    /// Cria novo TarFS a partir de dados TAR
    pub fn new(data: &'static [u8]) -> Result<Self, &'static str> {
        let archive = TarArchive::new(data)?;
        Ok(Self { archive })
    }

    /// Lê arquivo completo
    pub fn read(&self, path: &str) -> Result<Vec<u8>, &'static str> {
        // Normalizar path (remover / inicial se existir)
        let path = path.strip_prefix('/').unwrap_or(path);

        // Buscar no TAR
        let entry = self.archive.find(path).ok_or("File not found")?;

        if entry.is_dir {
            return Err("Is a directory");
        }

        // Ler dados
        let data = self.archive.read_entry(&entry);
        Ok(data.to_vec())
    }

    /// Lista diretório
    pub fn readdir(&self, path: &str) -> Result<Vec<String>, &'static str> {
        // Normalizar path
        let path = if path == "/" {
            ""
        } else {
            path.strip_prefix('/').unwrap_or(path)
        };

        Ok(self.archive.readdir(path))
    }

    /// Verifica se path existe
    pub fn exists(&self, path: &str) -> bool {
        let path = path.strip_prefix('/').unwrap_or(path);
        self.archive.find(path).is_some()
    }

    /// Verifica se é diretório
    pub fn is_dir(&self, path: &str) -> bool {
        let path = path.strip_prefix('/').unwrap_or(path);
        if let Some(entry) = self.archive.find(path) {
            entry.is_dir
        } else {
            false
        }
    }

    /// Obtém tamanho do arquivo
    pub fn file_size(&self, path: &str) -> Option<usize> {
        let path = path.strip_prefix('/').unwrap_or(path);
        self.archive.find(path).map(|e| e.size)
    }
}
