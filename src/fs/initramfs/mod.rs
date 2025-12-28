//! InitramFS - filesystem em memória do boot

use crate::mm::VirtAddr;
use crate::fs::vfs::inode::{InodeOps, FsError, DirEntry};
use alloc::vec::Vec;

/// Inode do initramfs
struct InitramfsInode {
    data: *const u8,
    size: usize,
}

impl InodeOps for InitramfsInode {
    fn lookup(&self, _name: &str) -> Option<u64> {
        // TODO: implementar lookup em CPIO/TAR
        None
    }
    
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let offset = offset as usize;
        if offset >= self.size {
            return Ok(0);
        }
        
        let to_read = buf.len().min(self.size - offset);
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.data.add(offset),
                buf.as_mut_ptr(),
                to_read
            );
        }
        Ok(to_read)
    }
    
    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, FsError> {
        Err(FsError::ReadOnly)
    }
    
    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        // TODO: parse CPIO entries
        Ok(Vec::new())
    }
}

/// Carrega initramfs da memória
pub fn init(addr: VirtAddr, size: usize) {
    crate::kinfo!("(InitramFS) Carregando de addr=", addr.as_u64());
    crate::kinfo!("(InitramFS) Tamanho:", size as u64);
    // TODO: parse e montar
}
