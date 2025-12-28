//! Mount points

use super::inode::InodeNum;
use alloc::string::String;

pub struct Mount {
    pub device: String,
    pub path: String,
    pub root_ino: InodeNum,
    // TODO: fs instance ref
}
