//! Virtual File System (VFS) Minimalista.
//!
//! Focado em prover a estrutura básica para o Initramfs e DevFS.
//! Não tenta ser um VFS completo POSIX neste estágio (isso é tarefa da libstd/userspace).

use crate::sync::Mutex;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Tipo de Nó no VFS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
    Device,
}

/// Um nó no grafo do sistema de arquivos (Inode simplificado).
pub trait VfsNode: Send + Sync {
    /// Nome do arquivo.
    fn name(&self) -> &str;

    /// Tipo do nó.
    fn kind(&self) -> NodeType;

    /// Tamanho em bytes.
    fn size(&self) -> u64;

    /// Abre o arquivo. Retorna um handle.
    fn open(&self) -> Result<Arc<dyn VfsHandle>, VfsError> {
        Err(VfsError::NotSupported)
    }

    /// Lista diretório (apenas se for Directory).
    fn list(&self) -> Result<Vec<Arc<dyn VfsNode>>, VfsError> {
        Err(VfsError::NotDirectory)
    }
}

/// Handle de arquivo aberto (File Description).
pub trait VfsHandle: Send + Sync {
    /// Lê dados do arquivo.
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize, VfsError>;

    /// Escreve dados no arquivo.
    fn write(&self, buf: &[u8], offset: u64) -> Result<usize, VfsError>;
}

#[derive(Debug, Clone, Copy)]
pub enum VfsError {
    NotFound,
    NotDirectory,
    NotFile,
    PermissionDenied,
    NotSupported,
    IoError,
}

/// O VFS Global.
pub struct Vfs {
    root: Option<Arc<dyn VfsNode>>,
}

impl Vfs {
    pub const fn new() -> Self {
        Self { root: None }
    }

    pub fn mount_root(&mut self, root: Arc<dyn VfsNode>) {
        self.root = Some(root);
    }

    /// Resolve um caminho absoluto (ex: "/bin/init").
    pub fn lookup(&self, path: &str) -> Result<Arc<dyn VfsNode>, VfsError> {
        let root = self.root.as_ref().ok_or(VfsError::NotFound)?;

        if path == "/" {
            return Ok(root.clone());
        }

        let mut current = root.clone();

        // Caminhar pelos componentes (simples, sem .. ou .)
        for component in path.split('/').filter(|c| !c.is_empty()) {
            if current.kind() != NodeType::Directory {
                return Err(VfsError::NotDirectory);
            }

            let children = current.list()?;
            let found = children
                .into_iter()
                .find(|child| child.name() == component)
                .ok_or(VfsError::NotFound)?;

            current = found;
        }

        Ok(current)
    }
}

pub static ROOT_VFS: Mutex<Vfs> = Mutex::new(Vfs::new());
