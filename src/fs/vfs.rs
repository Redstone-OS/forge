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
        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: path={}", path);
        let root = self.root.as_ref().ok_or(VfsError::NotFound)?;
        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: root OK");

        if path == "/" {
            crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: returning root");
            return Ok(root.clone());
        }

        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: cloning root...");
        let mut current = root.clone();
        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: clone OK");

        // Caminhar pelos componentes (simples, sem .. ou .)
        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: starting path iteration...");

        // Simplificado: normalizar path removendo / inicial
        let path_trimmed = path.trim_start_matches('/');
        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: path_trimmed={}", path_trimmed);

        // Buscar arquivo na lista (comparando diretamente)
        if current.kind() != NodeType::Directory {
            return Err(VfsError::NotDirectory);
        }

        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: list()...");
        let children = current.list()?;
        crate::kinfo!(
            "\x1b[36m[VFS]\x1b[0m lookup: list OK, {} children",
            children.len()
        );

        for child in children.iter() {
            let child_name = child.name();
            crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: child_name={}", child_name);

            // Comparação manual byte a byte (evitar starts_with que pode causar GPF)
            let child_bytes = child_name.as_bytes();
            let has_dot_slash =
                child_bytes.len() >= 2 && child_bytes[0] == b'.' && child_bytes[1] == b'/';
            crate::kinfo!(
                "\x1b[36m[VFS]\x1b[0m lookup: has_dot_slash={}",
                has_dot_slash
            );

            let child_normalized = if has_dot_slash {
                unsafe { core::str::from_utf8_unchecked(&child_bytes[2..]) }
            } else {
                child_name
            };
            crate::kinfo!(
                "\x1b[36m[VFS]\x1b[0m lookup: child_normalized={}",
                child_normalized
            );

            // Comparar byte a byte manualmente
            let path_bytes = path_trimmed.as_bytes();
            let norm_bytes = child_normalized.as_bytes();

            if path_bytes.len() == norm_bytes.len() {
                let mut is_match = true;
                for i in 0..path_bytes.len() {
                    if path_bytes[i] != norm_bytes[i] {
                        is_match = false;
                        break;
                    }
                }
                if is_match {
                    crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: FOUND!");
                    return Ok(child.clone());
                }
            }
        }

        crate::kinfo!("\x1b[36m[VFS]\x1b[0m lookup: NOT FOUND");
        Err(VfsError::NotFound)
    }
}

pub static ROOT_VFS: Mutex<Vfs> = Mutex::new(Vfs::new());
