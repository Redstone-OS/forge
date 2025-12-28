//! Inode - metadados de arquivo

/// Tipo de arquivo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
}

/// Permissões
#[derive(Debug, Clone, Copy)]
pub struct FileMode(pub u32);

impl FileMode {
    pub const OWNER_READ: u32 = 0o400;
    pub const OWNER_WRITE: u32 = 0o200;
    pub const OWNER_EXEC: u32 = 0o100;
    pub const GROUP_READ: u32 = 0o040;
    pub const GROUP_WRITE: u32 = 0o020;
    pub const GROUP_EXEC: u32 = 0o010;
    pub const OTHER_READ: u32 = 0o004;
    pub const OTHER_WRITE: u32 = 0o002;
    pub const OTHER_EXEC: u32 = 0o001;
    
    pub fn can_read(&self, is_owner: bool) -> bool {
        if is_owner {
            (self.0 & Self::OWNER_READ) != 0
        } else {
            (self.0 & Self::OTHER_READ) != 0
        }
    }
}

/// Número de inode
pub type InodeNum = u64;

/// Inode
pub struct Inode {
    /// Número único
    pub ino: InodeNum,
    /// Tipo de arquivo
    pub file_type: FileType,
    /// Permissões
    pub mode: FileMode,
    /// Tamanho em bytes
    pub size: u64,
    /// Links count
    pub nlink: u32,
    /// UID do dono
    pub uid: u32,
    /// GID do grupo
    pub gid: u32,
    /// Timestamps
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    /// Operações específicas
    pub ops: &'static dyn InodeOps,
}

/// Operações de inode
pub trait InodeOps: Send + Sync {
    /// Lookup em diretório
    fn lookup(&self, name: &str) -> Option<InodeNum>;
    
    /// Ler dados
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, FsError>;
    
    /// Escrever dados
    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize, FsError>;
    
    /// Listar diretório
    fn readdir(&self) -> Result<alloc::vec::Vec<DirEntry>, FsError>;
}

/// Entrada de diretório
pub struct DirEntry {
    pub name: alloc::string::String,
    pub ino: InodeNum,
    pub file_type: FileType,
}

/// Erro de filesystem
#[derive(Debug)]
pub enum FsError {
    NotFound,
    NotDirectory,
    IsDirectory,
    PermissionDenied,
    IoError,
    ReadOnly,
    NoSpace,
}
