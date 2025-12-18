//! Device Operations - Operações em dispositivos

use core::fmt;

/// Flags para abertura de dispositivos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenFlags {
    bits: u32,
}

impl OpenFlags {
    /// Apenas leitura
    pub const RDONLY: Self = Self { bits: 0o0 };
    /// Apenas escrita
    pub const WRONLY: Self = Self { bits: 0o1 };
    /// Leitura e escrita
    pub const RDWR: Self = Self { bits: 0o2 };
    /// Criar se não existir
    pub const CREAT: Self = Self { bits: 0o100 };
    /// Truncar ao abrir
    pub const TRUNC: Self = Self { bits: 0o1000 };
    /// Append
    pub const APPEND: Self = Self { bits: 0o2000 };
    /// Non-blocking
    pub const NONBLOCK: Self = Self { bits: 0o4000 };

    /// Cria flags a partir de bits
    pub const fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    /// Retorna os bits
    pub const fn bits(&self) -> u32 {
        self.bits
    }

    /// Verifica se tem flag de leitura
    pub const fn can_read(&self) -> bool {
        (self.bits & 0o3) == 0o0 || (self.bits & 0o3) == 0o2
    }

    /// Verifica se tem flag de escrita
    pub const fn can_write(&self) -> bool {
        (self.bits & 0o3) == 0o1 || (self.bits & 0o3) == 0o2
    }
}

/// Operações de dispositivo
pub trait DeviceOps {
    /// Abre o dispositivo
    fn open(&self, flags: OpenFlags) -> Result<(), &'static str>;

    /// Fecha o dispositivo
    fn close(&self) -> Result<(), &'static str>;

    /// Lê do dispositivo
    fn read(&self, buf: &mut [u8]) -> Result<usize, &'static str>;

    /// Escreve no dispositivo
    fn write(&self, buf: &[u8]) -> Result<usize, &'static str>;

    /// ioctl
    fn ioctl(&self, cmd: usize, arg: usize) -> Result<usize, &'static str>;

    /// mmap (mapear memória)
    fn mmap(&self, _offset: usize, _size: usize) -> Result<usize, &'static str> {
        Err("mmap not supported")
    }
}

/// Seek whence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekWhence {
    /// Do início
    Set,
    /// Da posição atual
    Cur,
    /// Do final
    End,
}

/// Resultado de stat
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceStat {
    /// Device number
    pub dev: u64,
    /// Inode number
    pub ino: u64,
    /// Mode (tipo + permissões)
    pub mode: u32,
    /// Número de hard links
    pub nlink: u32,
    /// UID do dono
    pub uid: u32,
    /// GID do grupo
    pub gid: u32,
    /// Device number (se for dispositivo)
    pub rdev: u64,
    /// Tamanho em bytes
    pub size: u64,
    /// Tamanho do bloco
    pub blksize: u32,
    /// Número de blocos
    pub blocks: u64,
}

impl DeviceStat {
    /// Cria um stat vazio
    pub const fn new() -> Self {
        Self {
            dev: 0,
            ino: 0,
            mode: 0,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            size: 0,
            blksize: 512,
            blocks: 0,
        }
    }
}

impl Default for DeviceStat {
    fn default() -> Self {
        Self::new()
    }
}
