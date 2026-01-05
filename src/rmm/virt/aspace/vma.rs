//! # Virtual Memory Area (VMA)
//!
//! Define regiões de memória virtual e suas propriedades.

use crate::rmm::addr::VirtAddr;

/// Intenção semântica da memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryIntent {
    Code,         // .text - RX
    Data,         // .data/.rodata - RW/RO
    Bss,          // .bss - RW, lazy zero
    Heap,         // brk/sbrk - RW, growable
    Stack,        // Stack - RW, grows down
    Mmap,         // mmap anonymous - RW
    SharedMemory, // IPC shared - varies
    DeviceBuffer, // Framebuffer, etc
    Guard,        // Guard page - no access
}

/// Proteção de memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Protection(u8);

impl Protection {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
    pub const EXEC: Self = Self(4);

    pub const RO: Self = Self(1);
    pub const RW: Self = Self(3);
    pub const RX: Self = Self(5);
    pub const RWX: Self = Self(7);

    #[inline]
    pub const fn can_read(&self) -> bool {
        (self.0 & 1) != 0
    }
    #[inline]
    pub const fn can_write(&self) -> bool {
        (self.0 & 2) != 0
    }
    #[inline]
    pub const fn can_exec(&self) -> bool {
        (self.0 & 4) != 0
    }
}

/// Flags de VMA
#[derive(Debug, Clone, Copy)]
pub struct VmaFlags(u32);

impl VmaFlags {
    pub const NONE: Self = Self(0);
    pub const GROWABLE: Self = Self(1 << 0);
    pub const LOCKED: Self = Self(1 << 1);
    pub const SHARED: Self = Self(1 << 2);
    pub const COW: Self = Self(1 << 3);
}

/// Virtual Memory Area
pub struct VMA {
    /// Início da região
    pub start: VirtAddr,
    /// Fim da região (exclusivo)
    pub end: VirtAddr,
    /// Proteção
    pub protection: Protection,
    /// Flags
    pub flags: VmaFlags,
    /// Intenção
    pub intent: MemoryIntent,
}

impl VMA {
    pub fn new(start: VirtAddr, size: usize, prot: Protection, intent: MemoryIntent) -> Self {
        Self {
            start,
            end: start + size as u64,
            protection: prot,
            flags: VmaFlags::NONE,
            intent,
        }
    }

    pub fn size(&self) -> usize {
        (self.end - self.start) as usize
    }

    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr >= self.start && addr < self.end
    }
}
