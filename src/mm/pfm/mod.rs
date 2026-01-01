//! # Page Frame Manager (PFM)

pub mod cache;
pub mod frame;
pub mod iommu;
pub mod rmap;
pub mod zero;

use crate::mm::PhysAddr;
use crate::sync::Spinlock;
use core::sync::atomic::{AtomicBool, Ordering};
use frame::{FrameFlags, FrameInfo, FrameState};

pub type Pid = u64;
pub const PID_KERNEL: Pid = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PfmError {
    FrameNotFound,
    AlreadyFree,
    NotOwner,
    RefcountZero,
    OutOfMemory,
    Pinned,
    DeviceFrame,
    OutOfBounds,
}

pub type PfmResult<T> = Result<T, PfmError>;

#[derive(Debug, Default)]
pub struct PfmStats {
    pub total_frames: u64,
    pub free_frames: u64,
    pub kernel_frames: u64,
    pub user_frames: u64,
    pub shared_frames: u64,
    pub pinned_frames: u64,
    pub device_frames: u64,
    pub allocations: u64,
    pub frees: u64,
}

pub struct PageFrameManager {
    pub frames: Option<&'static mut [FrameInfo]>,
    pub base_phys: u64,
    frame_count: usize,
    stats: PfmStats,
    initialized: bool,
}

impl PageFrameManager {
    pub fn new() -> Self {
        Self {
            frames: None,
            base_phys: 0,
            frame_count: 0,
            stats: PfmStats::default(),
            initialized: false,
        }
    }

    pub unsafe fn init(&mut self, frames: &'static mut [FrameInfo], base_phys: u64) {
        self.frame_count = frames.len();
        self.base_phys = base_phys;
        self.stats.total_frames = frames.len() as u64;
        self.stats.free_frames = frames.len() as u64;
        self.frames = Some(frames);
        self.initialized = true;
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }
    pub fn stats(&self) -> &PfmStats {
        &self.stats
    }

    fn phys_to_index(&self, phys: PhysAddr) -> Option<usize> {
        let addr = phys.as_u64();
        if addr < self.base_phys {
            return None;
        }
        let index = ((addr - self.base_phys) / crate::mm::config::PAGE_SIZE as u64) as usize;
        if index < self.frame_count {
            Some(index)
        } else {
            None
        }
    }

    pub fn alloc_frame(&mut self, owner: Pid, flags: FrameFlags) -> PfmResult<PhysAddr> {
        let phys = crate::mm::pmm::FRAME_ALLOCATOR
            .lock()
            .allocate_frame()
            .ok_or(PfmError::OutOfMemory)?;

        if let Some(index) = self.phys_to_index(phys) {
            if let Some(frames) = &mut self.frames {
                let state = if owner == PID_KERNEL {
                    FrameState::Kernel
                } else {
                    FrameState::Owned { owner }
                };
                frames[index].set_state(state);
                frames[index].set_flags(flags);
                frames[index].set_ref_count(1);
                self.stats.free_frames = self.stats.free_frames.saturating_sub(1);
                self.stats.allocations += 1;
            }
        }
        Ok(phys)
    }

    pub fn alloc_contiguous(
        &mut self,
        owner: Pid,
        count: usize,
        flags: FrameFlags,
    ) -> PfmResult<PhysAddr> {
        if count == 0 {
            return Err(PfmError::OutOfBounds);
        }
        let first = self.alloc_frame(owner, flags)?;
        for _ in 1..count {
            let _ = self.alloc_frame(owner, flags);
        }
        Ok(first)
    }

    pub fn free_frame(&mut self, phys: PhysAddr, owner: Pid) -> PfmResult<()> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &mut self.frames {
            let frame = &mut frames[index];
            match frame.state() {
                FrameState::Free => return Err(PfmError::AlreadyFree),
                FrameState::Owned { owner: o } if o != owner => return Err(PfmError::NotOwner),
                FrameState::Kernel if owner != PID_KERNEL => return Err(PfmError::NotOwner),
                FrameState::Pinned { .. } => return Err(PfmError::Pinned),
                FrameState::Device => return Err(PfmError::DeviceFrame),
                _ => {}
            }
            let new_count = frame.dec_ref_count();
            if new_count == 0 {
                frame.rmap_clear();
                frame.set_state(FrameState::Free);
                crate::mm::pmm::FRAME_ALLOCATOR
                    .lock()
                    .deallocate_frame(phys);
                self.stats.free_frames += 1;
                self.stats.frees += 1;
            }
        }
        Ok(())
    }

    pub fn inc_ref(&mut self, phys: PhysAddr) -> PfmResult<u32> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &mut self.frames {
            return Ok(frames[index].inc_ref_count());
        }
        Err(PfmError::FrameNotFound)
    }

    pub fn dec_ref(&mut self, phys: PhysAddr) -> PfmResult<u32> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &mut self.frames {
            return Ok(frames[index].dec_ref_count());
        }
        Err(PfmError::FrameNotFound)
    }

    pub fn get_state(&self, phys: PhysAddr) -> PfmResult<FrameState> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &self.frames {
            return Ok(frames[index].state());
        }
        Err(PfmError::FrameNotFound)
    }

    pub fn mark_kernel(&mut self, phys: PhysAddr) -> PfmResult<()> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &mut self.frames {
            frames[index].set_state(FrameState::Kernel);
            frames[index].set_ref_count(1);
        }
        Ok(())
    }

    pub fn mark_device(&mut self, phys: PhysAddr) -> PfmResult<()> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &mut self.frames {
            frames[index].set_state(FrameState::Device);
        }
        Ok(())
    }

    pub fn pin_frame(&mut self, phys: PhysAddr, owner: Pid) -> PfmResult<()> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &mut self.frames {
            frames[index].set_state(FrameState::Pinned { owner });
        }
        Ok(())
    }

    pub fn unpin_frame(&mut self, phys: PhysAddr, owner: Pid) -> PfmResult<()> {
        let index = self.phys_to_index(phys).ok_or(PfmError::FrameNotFound)?;
        if let Some(frames) = &mut self.frames {
            if let FrameState::Pinned { owner: o } = frames[index].state() {
                if o != owner {
                    return Err(PfmError::NotOwner);
                }
                frames[index].set_state(FrameState::Owned { owner });
            }
        }
        Ok(())
    }
}

impl Default for PageFrameManager {
    fn default() -> Self {
        Self::new()
    }
}

static PFM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Use OnceCell pattern or lazy init - for now use Option
static mut PFM_INNER: Option<PageFrameManager> = None;
static PFM_LOCK: Spinlock<()> = Spinlock::new(());

pub fn get() -> &'static Spinlock<PageFrameManager> {
    // Safety: This is a workaround - proper initialization needed
    static PFM: Spinlock<PageFrameManager> = Spinlock::new(PageFrameManager {
        frames: None,
        base_phys: 0,
        frame_count: 0,
        stats: PfmStats {
            total_frames: 0,
            free_frames: 0,
            kernel_frames: 0,
            user_frames: 0,
            shared_frames: 0,
            pinned_frames: 0,
            device_frames: 0,
            allocations: 0,
            frees: 0,
        },
        initialized: false,
    });
    &PFM
}

pub unsafe fn init(frames: &'static mut [FrameInfo], base_phys: u64) {
    get().lock().init(frames, base_phys);
    PFM_INITIALIZED.store(true, Ordering::Release);
}

pub fn is_initialized() -> bool {
    PFM_INITIALIZED.load(Ordering::Acquire)
}

pub fn alloc_kernel_frame() -> PfmResult<PhysAddr> {
    get().lock().alloc_frame(PID_KERNEL, FrameFlags::empty())
}

pub fn alloc_user_frame(owner: Pid) -> PfmResult<PhysAddr> {
    get().lock().alloc_frame(owner, FrameFlags::USER)
}

pub fn free_frame(phys: PhysAddr, owner: Pid) -> PfmResult<()> {
    get().lock().free_frame(phys, owner)
}

pub fn inc_ref(phys: PhysAddr) -> PfmResult<u32> {
    get().lock().inc_ref(phys)
}
pub fn dec_ref(phys: PhysAddr) -> PfmResult<u32> {
    get().lock().dec_ref(phys)
}
