//! # Page Fault Handler

use crate::mm::{MapFlags, PhysAddr, VirtAddr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultResult {
    Success,
    OutOfMemory,
    ProtectionViolation,
    InvalidAddress,
    BeyondLimit,
    FatalError,
}

#[derive(Debug, Clone, Copy)]
pub struct PageFaultInfo {
    pub addr: VirtAddr,
    pub error_code: u64,
    pub access: AccessType,
    pub user_mode: bool,
}

impl PageFaultInfo {
    pub fn from_error_code(addr: u64, error_code: u64) -> Self {
        let access = if error_code & 0x10 != 0 {
            AccessType::Execute
        } else if error_code & 0x02 != 0 {
            AccessType::Write
        } else {
            AccessType::Read
        };
        Self {
            addr: VirtAddr::new(addr),
            error_code,
            access,
            user_mode: error_code & 0x04 != 0,
        }
    }
}

pub fn handle_page_fault(info: PageFaultInfo) -> FaultResult {
    if info.addr.as_u64() >= 0xFFFF_8000_0000_0000 {
        return FaultResult::ProtectionViolation;
    }
    FaultResult::InvalidAddress
}

pub fn lazy_alloc(addr: VirtAddr, flags: MapFlags) -> Result<PhysAddr, FaultResult> {
    let phys = crate::mm::pmm::FRAME_ALLOCATOR
        .lock()
        .allocate_frame()
        .ok_or(FaultResult::OutOfMemory)?;

    unsafe {
        crate::mm::hhdm::zero_page(phys.as_u64());
    }

    crate::mm::map_page(addr.as_u64(), phys.as_u64(), flags)
        .map_err(|_| FaultResult::OutOfMemory)?;

    Ok(phys)
}

pub fn resolve_cow(
    addr: VirtAddr,
    old_phys: PhysAddr,
    flags: MapFlags,
) -> Result<PhysAddr, FaultResult> {
    let new_phys = crate::mm::pmm::FRAME_ALLOCATOR
        .lock()
        .allocate_frame()
        .ok_or(FaultResult::OutOfMemory)?;

    unsafe {
        let src: *const u8 = crate::mm::hhdm::phys_to_virt(old_phys.as_u64());
        let dst: *mut u8 = crate::mm::hhdm::phys_to_virt(new_phys.as_u64());
        core::ptr::copy_nonoverlapping(src, dst, crate::mm::config::PAGE_SIZE);
    }

    let _ = crate::mm::unmap_page(addr.as_u64());
    crate::mm::map_page(addr.as_u64(), new_phys.as_u64(), flags)
        .map_err(|_| FaultResult::OutOfMemory)?;

    Ok(new_phys)
}
