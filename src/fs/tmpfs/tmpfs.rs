//! RAM disk implementation

use crate::fs::vfs::inode::{Inode, InodeOps, FsError, DirEntry};
use alloc::vec::Vec;
use alloc::string::String;

pub struct TmpFs;

impl TmpFs {
    pub fn new() -> Self {
        Self
    }
}
