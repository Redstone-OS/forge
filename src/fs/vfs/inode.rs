//! Inodes - Representam arquivos e diretórios no VFS

/// Tipo de inode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
    Device,
    Symlink,
}

/// Inode - representa um arquivo ou diretório
pub struct Inode {
    /// Número do inode
    pub ino: u64,
    /// Tipo de inode
    pub inode_type: InodeType,
    /// Tamanho do arquivo
    pub size: u64,
    /// Permissões (modo Unix)
    pub mode: u16,
    /// UID do dono
    pub uid: u32,
    /// GID do grupo
    pub gid: u32,
}

impl Inode {
    /// Cria um novo inode
    pub const fn new(ino: u64, inode_type: InodeType) -> Self {
        Self {
            ino,
            inode_type,
            size: 0,
            mode: 0o644,
            uid: 0,
            gid: 0,
        }
    }

    /// Verifica se é um arquivo
    pub const fn is_file(&self) -> bool {
        matches!(self.inode_type, InodeType::File)
    }

    /// Verifica se é um diretório
    pub const fn is_dir(&self) -> bool {
        matches!(self.inode_type, InodeType::Directory)
    }
}

// TODO(prioridade=média, versão=v1.0): Implementar InodeOps trait
// - read() - Ler dados do inode
// - write() - Escrever dados
// - lookup() - Procurar entrada em diretório
// - create() - Criar novo arquivo
