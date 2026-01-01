//! # Heap Region Management

use super::vma::{MemoryIntent, Protection, VmaFlags, VMA};
use super::{ASpaceError, ASpaceResult};
use crate::mm::VirtAddr;

pub struct HeapManager {
    pub base: VirtAddr,
    pub brk: VirtAddr,
    pub max: VirtAddr,
}

impl HeapManager {
    pub fn new(base: VirtAddr, max: VirtAddr) -> Self {
        Self {
            base,
            brk: base,
            max,
        }
    }

    pub fn size(&self) -> u64 {
        self.brk.as_u64() - self.base.as_u64()
    }

    pub fn set_brk(&mut self, new_brk: VirtAddr) -> ASpaceResult<VirtAddr> {
        if new_brk < self.base {
            return Err(ASpaceError::InvalidAddress);
        }
        if new_brk > self.max {
            return Err(ASpaceError::OutOfMemory);
        }
        self.brk = new_brk;
        Ok(self.brk)
    }

    pub fn create_vma(&self) -> VMA {
        VMA::new(
            self.base,
            self.brk,
            Protection::RW,
            VmaFlags::GROWABLE,
            MemoryIntent::Heap,
        )
    }
}

impl Default for HeapManager {
    fn default() -> Self {
        Self::new(
            VirtAddr::new(0x0000_0001_0000_0000),
            VirtAddr::new(0x0000_7000_0000_0000),
        )
    }
}
