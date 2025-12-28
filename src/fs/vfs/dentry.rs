//! Directory Entry Cache
//!
//! Cache para acelerar lookups de path para inode.

use super::inode::InodeNum;
use alloc::string::String;
use alloc::sync::Arc;
use crate::sync::Spinlock;

pub struct Dentry {
    pub name: String,
    pub ino: InodeNum,
    pub parent: Option<Arc<Spinlock<Dentry>>>,
    // TODO: children cache
}
