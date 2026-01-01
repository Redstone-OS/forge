//! # Virtual Memory Area (VMA)
//!
//! Cada região de memória virtual com intenção semântica.

use crate::mm::VirtAddr;

/// Intenção de uso da memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryIntent {
    Code,
    Data,
    Bss,
    Heap,
    Stack,
    FileReadOnly,
    FilePrivate,
    SharedMemory,
    DeviceBuffer,
    Guard,
}

/// Proteção de página
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Protection(u8);

impl Protection {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
    pub const EXEC: Self = Self(4);
    pub const RW: Self = Self(3);
    pub const RX: Self = Self(5);
    pub const RWX: Self = Self(7);

    pub fn can_read(&self) -> bool {
        self.0 & 1 != 0
    }
    pub fn can_write(&self) -> bool {
        self.0 & 2 != 0
    }
    pub fn can_exec(&self) -> bool {
        self.0 & 4 != 0
    }

    pub fn permits(&self, access: crate::mm::fault::AccessType) -> bool {
        match access {
            crate::mm::fault::AccessType::Read => self.can_read(),
            crate::mm::fault::AccessType::Write => self.can_write(),
            crate::mm::fault::AccessType::Execute => self.can_exec(),
        }
    }
}

/// Flags de VMA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmaFlags(u32);

impl VmaFlags {
    pub const GROWABLE: Self = Self(1 << 0);
    pub const GROWS_DOWN: Self = Self(1 << 1);
    pub const COW: Self = Self(1 << 2);
    pub const SHARED: Self = Self(1 << 3);
    pub const LOCKED: Self = Self(1 << 4);

    pub const fn empty() -> Self {
        Self(0)
    }
    pub const fn bits(&self) -> u32 {
        self.0
    }
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl core::ops::BitOr for VmaFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for VmaFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Backing de uma VMA
#[derive(Debug, Clone)]
pub enum VmaBacking {
    Anonymous,
}

/// Virtual Memory Area
#[derive(Debug, Clone)]
pub struct VMA {
    pub start: VirtAddr,
    pub end: VirtAddr,
    pub protection: Protection,
    pub flags: VmaFlags,
    pub intent: MemoryIntent,
    pub backing: VmaBacking,
}

impl VMA {
    pub fn new(
        start: VirtAddr,
        end: VirtAddr,
        protection: Protection,
        flags: VmaFlags,
        intent: MemoryIntent,
    ) -> Self {
        Self {
            start,
            end,
            protection,
            flags,
            intent,
            backing: VmaBacking::Anonymous,
        }
    }

    pub fn size(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }
    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr >= self.start && addr < self.end
    }
}
