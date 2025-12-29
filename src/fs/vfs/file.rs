#![allow(dead_code)]
//! Arquivo aberto

use super::inode::{FsError, Inode};
use crate::sync::Mutex;

/// Flags de abertura
#[derive(Debug, Clone, Copy)]
pub struct OpenFlags(pub u32);

impl OpenFlags {
    pub const READ: u32 = 1;
    pub const WRITE: u32 = 2;
    pub const APPEND: u32 = 4;
    pub const CREATE: u32 = 8;
    pub const TRUNCATE: u32 = 16;
}

/// Operações de arquivo
pub trait FileOps {
    fn read(&self, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write(&self, buf: &[u8]) -> Result<usize, FsError>;
    fn seek(&self, position: u64);
}

/// Arquivo aberto
pub struct File {
    /// Inode associado
    inode: *const Inode,
    /// Posição atual
    offset: Mutex<u64>,
    /// Flags de abertura
    flags: OpenFlags,
}

impl FileOps for File {
    fn read(&self, buf: &mut [u8]) -> Result<usize, FsError> {
        self.read_impl(buf)
    }

    fn write(&self, buf: &[u8]) -> Result<usize, FsError> {
        self.write_impl(buf)
    }

    fn seek(&self, position: u64) {
        self.seek_impl(position)
    }
}

impl File {
    /// Cria arquivo aberto
    pub fn new(inode: *const Inode, flags: OpenFlags) -> Self {
        Self {
            inode,
            offset: Mutex::new(0),
            flags,
        }
    }

    /// Lê dados
    pub fn read_impl(&self, buf: &mut [u8]) -> Result<usize, FsError> {
        let inode = unsafe { &*self.inode };
        let mut offset = self.offset.lock();
        let bytes = inode.ops.read(*offset, buf)?;
        *offset += bytes as u64;
        Ok(bytes)
    }

    /// Escreve dados
    pub fn write_impl(&self, buf: &[u8]) -> Result<usize, FsError> {
        let inode = unsafe { &*self.inode };
        let mut offset = self.offset.lock();
        let bytes = inode.ops.write(*offset, buf)?;
        *offset += bytes as u64;
        Ok(bytes)
    }

    /// Seek
    pub fn seek_impl(&self, position: u64) {
        *self.offset.lock() = position;
    }
}
