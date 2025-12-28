//! ProcFS implementation

use crate::fs::vfs::inode::{InodeOps, FsError, DirEntry};
use alloc::vec::Vec;

pub struct ProcFs;

impl ProcFs {
    pub fn new() -> Self {
        Self
    }
}

impl InodeOps for ProcFs {
    fn lookup(&self, _name: &str) -> Option<u64> {
        // TODO: lookup PID folders
        None
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, FsError> {
        Ok(0)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, FsError> {
        Err(FsError::ReadOnly)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        Ok(Vec::new())
    }
}
