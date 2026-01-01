//! # Address Space Manager

pub mod heap;
pub mod rbtree;
pub mod shared;
pub mod vma;

extern crate alloc;

use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Spinlock;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use vma::{MemoryIntent, Protection, VmaFlags, VMA};

pub type Pid = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASpaceError {
    OutOfMemory,
    InvalidAddress,
    InvalidSize,
    RegionOverlap,
    RegionNotFound,
    ProtectionViolation,
    AlreadyMapped,
    NotMapped,
}

pub type ASpaceResult<T> = Result<T, ASpaceError>;

#[derive(Debug, Default, Clone)]
pub struct AddressSpaceStats {
    pub vma_count: u64,
    pub mapped_pages: u64,
    pub resident_pages: u64,
    pub shared_pages: u64,
}

pub struct AddressSpace {
    pml4: PhysAddr,
    vmas: Spinlock<Vec<VMA>>,
    owner: Pid,
    stats: AddressSpaceStats,
    pcid: u16,
    tlb_gen: AtomicU64,
}

impl AddressSpace {
    pub fn new(owner: Pid) -> ASpaceResult<Self> {
        let pml4_phys = {
            let mut pmm = crate::mm::pmm::FRAME_ALLOCATOR.lock();
            crate::mm::vmm::mapper::create_new_p4(&mut *pmm)
                .map_err(|_| ASpaceError::OutOfMemory)?
        };

        Ok(Self {
            pml4: PhysAddr::new(pml4_phys),
            vmas: Spinlock::new(Vec::new()),
            owner,
            stats: AddressSpaceStats::default(),
            pcid: 0,
            tlb_gen: AtomicU64::new(0),
        })
    }

    pub fn cr3(&self) -> u64 {
        self.pml4.as_u64()
    }
    pub fn owner(&self) -> Pid {
        self.owner
    }

    pub fn map_region(
        &mut self,
        hint: Option<VirtAddr>,
        size: usize,
        prot: Protection,
        flags: VmaFlags,
        intent: MemoryIntent,
    ) -> ASpaceResult<VirtAddr> {
        let size = crate::klib::align_up(size, crate::mm::config::PAGE_SIZE);
        let addr = self.find_free_region(hint, size)?;
        let vma = VMA::new(
            addr,
            VirtAddr::new(addr.as_u64() + size as u64),
            prot,
            flags,
            intent,
        );
        self.vmas.lock().push(vma);
        self.stats.vma_count += 1;
        Ok(addr)
    }

    pub fn unmap_region(&mut self, addr: VirtAddr, _size: usize) -> ASpaceResult<()> {
        let mut vmas = self.vmas.lock();
        let idx = vmas
            .iter()
            .position(|v| v.start == addr)
            .ok_or(ASpaceError::RegionNotFound)?;
        vmas.remove(idx);
        self.stats.vma_count = self.stats.vma_count.saturating_sub(1);
        self.tlb_gen.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn find_vma(&self, addr: VirtAddr) -> Option<VMA> {
        self.vmas
            .lock()
            .iter()
            .find(|v| addr >= v.start && addr < v.end)
            .cloned()
    }

    fn find_free_region(&self, hint: Option<VirtAddr>, size: usize) -> ASpaceResult<VirtAddr> {
        let base = hint.unwrap_or(VirtAddr::new(0x0000_0001_0000_0000));
        let vmas = self.vmas.lock();
        let mut candidate = base;
        for vma in vmas.iter() {
            let end = candidate.as_u64() + size as u64;
            if end <= vma.start.as_u64() {
                return Ok(candidate);
            }
            candidate = vma.end;
        }
        let end = candidate.as_u64() + size as u64;
        if end < 0x0000_7FFF_FFFF_0000 {
            Ok(candidate)
        } else {
            Err(ASpaceError::OutOfMemory)
        }
    }

    pub unsafe fn activate(&self) {
        crate::arch::Cpu::write_cr3(self.pml4.as_u64());
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        crate::mm::pmm::FRAME_ALLOCATOR
            .lock()
            .deallocate_frame(self.pml4);
    }
}
