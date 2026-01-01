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
    pub ip: VirtAddr,
    pub error_code: u64,
    pub access: AccessType,
    pub user_mode: bool,
}

impl PageFaultInfo {
    pub fn from_error_code(addr: u64, ip: u64, error_code: u64) -> Self {
        let access = if error_code & 0x10 != 0 {
            AccessType::Execute
        } else if error_code & 0x02 != 0 {
            AccessType::Write
        } else {
            AccessType::Read
        };
        Self {
            addr: VirtAddr::new(addr),
            ip: VirtAddr::new(ip),
            error_code,
            access,
            user_mode: error_code & 0x04 != 0,
        }
    }
}

pub fn handle_page_fault(info: PageFaultInfo) -> FaultResult {
    // 1. Verificar se é um endereço de Kernel Space
    if info.addr.as_u64() >= 0xFFFF_8000_0000_0000 {
        crate::kerror!("(Fault) Kernel Page Fault at:", info.addr.as_u64());
        crate::kerror!("(Fault) Faulting RIP:", info.ip.as_u64());
        return FaultResult::FatalError;
    }

    // 2. Obter AddressSpace da tarefa atual
    let current_guard = crate::sched::core::CURRENT.lock();
    let aspace_arc = match current_guard.as_ref() {
        Some(task) => match &task.aspace {
            Some(as_arc) => as_arc.clone(),
            None => return FaultResult::FatalError,
        },
        None => return FaultResult::FatalError,
    };
    drop(current_guard);

    let as_lock = aspace_arc.lock();

    // 3. Procurar VMA correspondente
    let vma = match as_lock.find_vma(info.addr) {
        Some(v) => v,
        None => {
            crate::kerror!(
                "(Fault) Falha de Segmentacao (Sem VMA) em:",
                info.addr.as_u64()
            );
            crate::kerror!("(Fault) RIP da Falha:", info.ip.as_u64());
            return FaultResult::InvalidAddress;
        }
    };

    // 4. Validar permissões
    if !vma.protection.permits(info.access) {
        crate::kerror!("(Fault) Protection Violation at:", info.addr.as_u64());
        return FaultResult::ProtectionViolation;
    }

    // 5. Resolver Fault (Lazy Allocation para Anonymous)
    crate::kdebug!("(Fault) Lazy allocation for:", info.addr.as_u64());

    // Converter Protection/VmaFlags para MapFlags (Simplificado)
    let mut flags = MapFlags::PRESENT | MapFlags::USER;
    if vma.protection.can_write() {
        flags |= MapFlags::WRITABLE;
    }
    if vma.protection.can_exec() {
        flags |= MapFlags::EXECUTABLE;
    }

    match lazy_alloc(info.addr.align_down(4096), flags) {
        Ok(_) => FaultResult::Success,
        Err(e) => e,
    }
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
        let src: *const u8 = crate::mm::hhdm::phys_to_virt::<u8>(old_phys.as_u64());
        let dst: *mut u8 = crate::mm::hhdm::phys_to_virt::<u8>(new_phys.as_u64());
        core::ptr::copy_nonoverlapping(src, dst, crate::mm::config::PAGE_SIZE);
    }

    let _ = crate::mm::unmap_page(addr.as_u64());
    crate::mm::map_page(addr.as_u64(), new_phys.as_u64(), flags)
        .map_err(|_| FaultResult::OutOfMemory)?;

    Ok(new_phys)
}
