//! Files - Arquivos abertos no VFS

use super::FilesystemType;

/// VFSFile - Arquivo aberto no VFS
pub struct VFSFile {
    /// Path completo do arquivo
    pub path: &'static str,
    /// Tipo de filesystem
    pub fs_type: FilesystemType,
    /// Offset de leitura/escrita
    pub offset: u64,
}

impl VFSFile {
    /// Cria um novo arquivo
    pub const fn new(path: &'static str, fs_type: FilesystemType) -> Self {
        Self {
            path,
            fs_type,
            offset: 0,
        }
    }

    /// Seek para posição
    pub fn seek(&mut self, offset: u64) {
        self.offset = offset;
    }

    /// Retorna offset atual
    pub const fn tell(&self) -> u64 {
        self.offset
    }
}
