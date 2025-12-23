//! Syscalls de Memória (mmap, brk).
//! Placeholder para futuro.

use super::numbers::*;

pub fn sys_mmap(
    _addr: usize,
    _len: usize,
    _prot: usize,
    _flags: usize,
    _fd: usize,
    _offset: usize,
) -> isize {
    ENOSYS
}

pub fn sys_munmap(_addr: usize, _len: usize) -> isize {
    ENOSYS
}
