//! TmpFS - Temporary Filesystem
//!
//! Sistema de arquivos temporário em memória RAM.
//!
//! # Implementação Básica
//! Armazenamento simples em array estático para boot inicial.

#![allow(dead_code)]

/// Tamanho máximo de um arquivo
const MAX_FILE_SIZE: usize = 4096;

/// Número máximo de arquivos
const MAX_FILES: usize = 16;

/// Arquivo no TmpFS
pub struct TmpFile {
    /// Nome do arquivo
    pub name: [u8; 64],
    /// Tamanho do nome
    pub name_len: usize,
    /// Dados do arquivo
    pub data: [u8; MAX_FILE_SIZE],
    /// Tamanho dos dados
    pub size: usize,
    /// Se é diretório
    pub is_dir: bool,
    /// Se está em uso
    pub in_use: bool,
}

impl TmpFile {
    /// Cria um arquivo vazio
    pub const fn new() -> Self {
        Self {
            name: [0; 64],
            name_len: 0,
            data: [0; MAX_FILE_SIZE],
            size: 0,
            is_dir: false,
            in_use: false,
        }
    }

    /// Verifica se o nome corresponde
    pub fn name_matches(&self, name: &str) -> bool {
        if !self.in_use || name.len() != self.name_len {
            return false;
        }
        &self.name[..self.name_len] == name.as_bytes()
    }
}

/// TmpFS - Temporary Filesystem
pub struct TmpFS {
    /// Array de arquivos
    files: [TmpFile; MAX_FILES],
    /// Tamanho máximo total
    pub max_size: usize,
    /// Tamanho usado
    pub used_size: usize,
}

impl TmpFS {
    /// Cria uma nova instância de TmpFS
    pub fn new(max_size: usize) -> Self {
        const EMPTY_FILE: TmpFile = TmpFile::new();
        Self {
            files: [EMPTY_FILE; MAX_FILES],
            max_size,
            used_size: 0,
        }
    }

    /// Cria um novo arquivo
    pub fn create_file(&mut self, name: &str, data: &[u8]) -> Result<(), &'static str> {
        if name.len() > 64 {
            return Err("Name too long");
        }
        if data.len() > MAX_FILE_SIZE {
            return Err("File too large");
        }
        if self.used_size + data.len() > self.max_size {
            return Err("Not enough space");
        }

        // Procurar slot vazio
        let slot = self
            .files
            .iter_mut()
            .find(|f| !f.in_use)
            .ok_or("Too many files")?;

        // Copiar nome
        slot.name[..name.len()].copy_from_slice(name.as_bytes());
        slot.name_len = name.len();

        // Copiar dados
        slot.data[..data.len()].copy_from_slice(data);
        slot.size = data.len();

        slot.is_dir = false;
        slot.in_use = true;

        self.used_size += data.len();
        Ok(())
    }

    /// Cria um diretório
    pub fn create_dir(&mut self, name: &str) -> Result<(), &'static str> {
        if name.len() > 64 {
            return Err("Name too long");
        }

        // Procurar slot vazio
        let slot = self
            .files
            .iter_mut()
            .find(|f| !f.in_use)
            .ok_or("Too many files")?;

        // Copiar nome
        slot.name[..name.len()].copy_from_slice(name.as_bytes());
        slot.name_len = name.len();
        slot.size = 0;
        slot.is_dir = true;
        slot.in_use = true;

        Ok(())
    }

    /// Lê um arquivo
    pub fn read_file(&self, name: &str) -> Result<&[u8], &'static str> {
        let file = self
            .files
            .iter()
            .find(|f| f.name_matches(name))
            .ok_or("File not found")?;

        if file.is_dir {
            return Err("Is a directory");
        }

        Ok(&file.data[..file.size])
    }

    /// Remove um arquivo
    pub fn remove(&mut self, name: &str) -> Result<(), &'static str> {
        let file = self
            .files
            .iter_mut()
            .find(|f| f.name_matches(name))
            .ok_or("File not found")?;

        self.used_size -= file.size;
        file.in_use = false;
        file.size = 0;
        file.name_len = 0;

        Ok(())
    }

    /// Lista arquivos
    pub fn list(&self) -> impl Iterator<Item = &TmpFile> {
        self.files.iter().filter(|f| f.in_use)
    }

    /// Retorna espaço disponível
    pub fn available_space(&self) -> usize {
        self.max_size - self.used_size
    }
}

impl Default for TmpFS {
    fn default() -> Self {
        // 1 MB por padrão
        Self::new(1024 * 1024)
    }
}

// TODO(prioridade=média, versão=v1.0): Implementar com alloc
// - Usar Vec para arquivos dinâmicos
// - Remover limite de MAX_FILES
// - Suporte a arquivos maiores

// TODO(prioridade=baixa, versão=v2.0): Implementar diretórios aninhados
// - Árvore de diretórios
// - Paths completos (/tmp/foo/bar)
