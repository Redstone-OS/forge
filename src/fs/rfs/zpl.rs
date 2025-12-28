//! ZFS POSIX Layer (ZPL)
//!
//! Interfaces RFS with the Kernel VFS.

use super::dmu::{ObjSet, Object};
use crate::fs::vfs::inode::{InodeOps, FsError, DirEntry};
use alloc::vec::Vec;
use alloc::sync::Arc;

pub struct Zpl {
    os: Arc<ObjSet>,
}

impl Zpl {
    pub fn new(os: Arc<ObjSet>) -> Self {
        Self { os }
    }
}

/// Adapter for VFS InodeOps
struct ZplInode {
    object: Object,
    zpl: Arc<Zpl>,
}

impl InodeOps for ZplInode {
    fn lookup(&self, _name: &str) -> Option<u64> {
        // TODO: Use DMU to lookup directory entry
        None
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, FsError> {
        // TODO: Use DMU to read object data
        Ok(0)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, FsError> {
        // TODO: Use DMU transaction to write data
        Err(FsError::ReadOnly)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        // TODO: Iterate DMU directory object
        Ok(Vec::new())
    }
}
