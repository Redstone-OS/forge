//! Device Filesystem (/dev).
//!
//! Mapeia dispositivos de hardware (drivers) para arquivos virtuais.
//! Essencial para `stdin`, `stdout`, `stderr`.

use super::vfs::{NodeType, VfsError, VfsHandle, VfsNode};
use alloc::sync::Arc;
use alloc::vec::Vec;

pub struct DevFs;

impl VfsNode for DevFs {
    fn name(&self) -> &str {
        "dev"
    }
    fn kind(&self) -> NodeType {
        NodeType::Directory
    }
    fn size(&self) -> u64 {
        0
    }

    fn list(&self) -> Result<Vec<Arc<dyn VfsNode>>, VfsError> {
        let mut devices = Vec::new();
        devices.push(Arc::new(ConsoleDevice) as Arc<dyn VfsNode>);
        devices.push(Arc::new(NullDevice) as Arc<dyn VfsNode>);
        Ok(devices)
    }
}

// --- /dev/console ---
struct ConsoleDevice;
impl VfsNode for ConsoleDevice {
    fn name(&self) -> &str {
        "console"
    }
    fn kind(&self) -> NodeType {
        NodeType::Device
    }
    fn size(&self) -> u64 {
        0
    }
    fn open(&self) -> Result<Arc<dyn VfsHandle>, VfsError> {
        Ok(Arc::new(ConsoleHandle))
    }
}

struct ConsoleHandle;
impl VfsHandle for ConsoleHandle {
    fn read(&self, _buf: &mut [u8], _offset: u64) -> Result<usize, VfsError> {
        // TODO: Ler do driver de teclado (buffer)
        Ok(0)
    }

    fn write(&self, buf: &[u8], _offset: u64) -> Result<usize, VfsError> {
        // Escrever no Logger do Kernel (kprint!)
        if let Ok(s) = core::str::from_utf8(buf) {
            crate::kprint!("{}", s);
        } else {
            for &b in buf {
                crate::kprint!("{}", b as char);
            }
        }
        Ok(buf.len())
    }
}

// --- /dev/null ---
struct NullDevice;
impl VfsNode for NullDevice {
    fn name(&self) -> &str {
        "null"
    }
    fn kind(&self) -> NodeType {
        NodeType::Device
    }
    fn size(&self) -> u64 {
        0
    }
    fn open(&self) -> Result<Arc<dyn VfsHandle>, VfsError> {
        Ok(Arc::new(NullHandle))
    }
}

struct NullHandle;
impl VfsHandle for NullHandle {
    fn read(&self, _buf: &mut [u8], _offset: u64) -> Result<usize, VfsError> {
        Ok(0) // EOF imediato
    }
    fn write(&self, buf: &[u8], _offset: u64) -> Result<usize, VfsError> {
        Ok(buf.len()) // Aceita tudo, joga fora
    }
}
