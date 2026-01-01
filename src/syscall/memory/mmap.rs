//! # Memory Mapping Syscalls

use crate::mm::aspace::vma::{MemoryIntent, Protection, VmaFlags};
use crate::syscall::SysResult;

pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1;
pub const PROT_WRITE: u32 = 2;
pub const PROT_EXEC: u32 = 4;

pub const MAP_PRIVATE: u32 = 0x02;
pub const MAP_SHARED: u32 = 0x01;
pub const MAP_ANONYMOUS: u32 = 0x20;
pub const MAP_FIXED: u32 = 0x10;

pub fn sys_mmap(
    _hint: usize,
    size: usize,
    _prot: u32,
    flags: u32,
    fd: i32,
    _offset: u64,
) -> SysResult<usize> {
    if size == 0 || size > 0x0000_7FFF_FFFF_0000 {
        return Err(crate::syscall::SysError::InvalidArgument);
    }

    if flags & MAP_ANONYMOUS == 0 && fd >= 0 {
        return Err(crate::syscall::SysError::NotSupported);
    }

    Err(crate::syscall::SysError::NotSupported)
}

pub fn sys_munmap(_addr: usize, size: usize) -> SysResult<usize> {
    if size == 0 {
        return Err(crate::syscall::SysError::InvalidArgument);
    }
    Err(crate::syscall::SysError::NotSupported)
}

pub fn sys_mprotect(_addr: usize, size: usize, _prot: u32) -> SysResult<usize> {
    if size == 0 {
        return Err(crate::syscall::SysError::InvalidArgument);
    }
    Err(crate::syscall::SysError::NotSupported)
}

fn convert_prot(prot: u32) -> Protection {
    if prot & (PROT_WRITE | PROT_EXEC) == (PROT_WRITE | PROT_EXEC) {
        Protection::RWX
    } else if prot & PROT_WRITE != 0 {
        Protection::RW
    } else if prot & PROT_EXEC != 0 {
        Protection::RX
    } else if prot & PROT_READ != 0 {
        Protection::READ
    } else {
        Protection::NONE
    }
}

fn convert_flags(flags: u32) -> VmaFlags {
    let mut f = VmaFlags::empty();
    if flags & MAP_SHARED != 0 {
        f = f | VmaFlags::SHARED;
    }
    if flags & MAP_PRIVATE != 0 {
        f = f | VmaFlags::COW;
    }
    f
}

fn infer_intent(prot: u32, flags: u32) -> MemoryIntent {
    if flags & MAP_ANONYMOUS != 0 {
        if prot & PROT_EXEC != 0 {
            MemoryIntent::Code
        } else {
            MemoryIntent::Heap
        }
    } else {
        if flags & MAP_PRIVATE != 0 {
            MemoryIntent::FilePrivate
        } else {
            MemoryIntent::FileReadOnly
        }
    }
}
