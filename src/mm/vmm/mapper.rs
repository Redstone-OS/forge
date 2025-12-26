//! # Mapper - API de Alto Nível para Mapeamento de Memória
//!
//! ## Propósito
//! - API limpa para map/unmap/protect
//! - Guard Pages para stack overflow
//! - Políticas W^X
//!
//! ## Uso
//! ```rust
//! let region = mapper::map_region(addr, 16, MapFlags::KERNEL_STACK)?;
//! mapper::unmap_region(region)?;
//! ```

use crate::mm::addr::{PhysAddr, VirtAddr};
use crate::mm::config::{PAGE_NO_EXEC, PAGE_PRESENT, PAGE_SIZE, PAGE_USER, PAGE_WRITABLE};
use crate::mm::error::{MmError, MmResult};
use crate::mm::pmm::{PhysFrame, FRAME_ALLOCATOR};
use crate::mm::vmm::tlb;
use core::sync::atomic::{AtomicUsize, Ordering};

// =============================================================================
// FLAGS DE MAPEAMENTO
// =============================================================================

/// Flags de mapeamento de alto nível
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapFlags(u32);

impl MapFlags {
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXEC: Self = Self(1 << 2);
    pub const USER: Self = Self(1 << 3);
    pub const GLOBAL: Self = Self(1 << 4);
    pub const GUARD: Self = Self(1 << 5);
    pub const ZERO: Self = Self(1 << 6);

    // Presets
    pub const KERNEL_CODE: Self = Self(0x05); // READ | EXEC
    pub const KERNEL_DATA: Self = Self(0x03); // READ | WRITE
    pub const KERNEL_STACK: Self = Self(0x23); // READ | WRITE | GUARD
    pub const USER_CODE: Self = Self(0x0D); // READ | EXEC | USER
    pub const USER_STACK: Self = Self(0x2B); // READ | WRITE | USER | GUARD

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Converte para flags PTE x86_64
    pub fn to_pte_flags(self) -> u64 {
        let mut flags = PAGE_PRESENT;
        if self.contains(Self::WRITE) {
            flags |= PAGE_WRITABLE;
        }
        if self.contains(Self::USER) {
            flags |= PAGE_USER;
        }
        if !self.contains(Self::EXEC) {
            flags |= PAGE_NO_EXEC;
        }
        flags
    }
}

// =============================================================================
// TIPOS DE REGIÃO
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RegionType {
    Generic = 0,
    KernelCode = 1,
    KernelData = 2,
    KernelStack = 3,
    KernelHeap = 4,
    UserCode = 5,
    UserStack = 6,
    DeviceMemory = 7,
    Guard = 8,
}

// =============================================================================
// REGIÃO MAPEADA
// =============================================================================

/// Região de memória virtual mapeada
#[derive(Debug)]
pub struct MappedRegion {
    pub virt_start: VirtAddr,
    pub total_pages: usize,
    pub usable_pages: usize,
    pub usable_start: VirtAddr,
    pub flags: MapFlags,
    pub region_type: RegionType,
    frames: Option<alloc::vec::Vec<PhysFrame>>,
}

impl MappedRegion {
    pub fn start(&self) -> VirtAddr {
        self.usable_start
    }
    pub fn size(&self) -> usize {
        self.usable_pages * PAGE_SIZE
    }

    pub fn contains(&self, addr: VirtAddr) -> bool {
        let end = self.usable_start.as_u64() + (self.usable_pages as u64 * PAGE_SIZE as u64);
        addr.as_u64() >= self.usable_start.as_u64() && addr.as_u64() < end
    }
}

// =============================================================================
// ESTATÍSTICAS
// =============================================================================

pub struct MapperStats {
    pub regions_mapped: AtomicUsize,
    pub pages_mapped: AtomicUsize,
    pub guard_pages: AtomicUsize,
    pub wx_blocked: AtomicUsize,
}

impl MapperStats {
    pub const fn new() -> Self {
        Self {
            regions_mapped: AtomicUsize::new(0),
            pages_mapped: AtomicUsize::new(0),
            guard_pages: AtomicUsize::new(0),
            wx_blocked: AtomicUsize::new(0),
        }
    }
}

pub static MAPPER_STATS: MapperStats = MapperStats::new();

// =============================================================================
// VERIFICAÇÕES
// =============================================================================

/// Verifica política W^X
pub fn check_wx_policy(flags: MapFlags) -> MmResult<()> {
    if flags.contains(MapFlags::WRITE) && flags.contains(MapFlags::EXEC) {
        #[cfg(feature = "wx_enforcement")]
        {
            MAPPER_STATS.wx_blocked.fetch_add(1, Ordering::Relaxed);
            return Err(MmError::WxViolation);
        }
        #[cfg(not(feature = "wx_enforcement"))]
        crate::kwarn!("(Mapper) W^X: RWX detectado");
    }
    Ok(())
}

fn validate_vaddr(vaddr: VirtAddr) -> MmResult<()> {
    if vaddr.as_u64() % PAGE_SIZE as u64 != 0 {
        return Err(MmError::NotAligned);
    }
    Ok(())
}

// =============================================================================
// MAPEAMENTO
// =============================================================================

/// Mapeia região de memória virtual
pub fn map_region(
    virt_start: VirtAddr,
    num_pages: usize,
    flags: MapFlags,
    region_type: RegionType,
) -> MmResult<MappedRegion> {
    validate_vaddr(virt_start)?;
    check_wx_policy(flags)?;

    let has_guards = flags.contains(MapFlags::GUARD);
    let guard_pages = if has_guards { 2 } else { 0 };
    let total_pages = num_pages + guard_pages;

    let usable_start = if has_guards {
        VirtAddr::new(virt_start.as_u64() + PAGE_SIZE as u64)
    } else {
        virt_start
    };

    let pte_flags = flags.to_pte_flags();
    let mut frames = alloc::vec::Vec::with_capacity(num_pages);
    let mut pmm = FRAME_ALLOCATOR.lock();

    for i in 0..total_pages {
        let vaddr = virt_start.as_u64() + (i as u64 * PAGE_SIZE as u64);

        // Guard pages não são mapeadas
        if has_guards && (i == 0 || i == total_pages - 1) {
            MAPPER_STATS.guard_pages.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        let frame = pmm.allocate_frame().ok_or(MmError::OutOfMemory)?;
        unsafe {
            crate::mm::vmm::map_page_with_pmm(vaddr, frame.addr(), pte_flags, &mut *pmm)?;
        }
        frames.push(frame);
        MAPPER_STATS.pages_mapped.fetch_add(1, Ordering::Relaxed);
    }

    MAPPER_STATS.regions_mapped.fetch_add(1, Ordering::Relaxed);

    Ok(MappedRegion {
        virt_start,
        total_pages,
        usable_pages: num_pages,
        usable_start,
        flags,
        region_type,
        frames: Some(frames),
    })
}

/// Desmapeia região
pub fn unmap_region(region: MappedRegion) -> MmResult<()> {
    let has_guards = region.flags.contains(MapFlags::GUARD);

    for i in 0..region.total_pages {
        if has_guards && (i == 0 || i == region.total_pages - 1) {
            continue;
        }
        let vaddr = region.virt_start.as_u64() + (i as u64 * PAGE_SIZE as u64);
        unsafe {
            let _ = crate::mm::vmm::unmap_page(vaddr);
        }
    }

    if let Some(frames) = region.frames {
        let mut pmm = FRAME_ALLOCATOR.lock();
        for frame in frames {
            pmm.deallocate_frame(frame);
        }
    }

    Ok(())
}

/// Desmapeia página individual
pub unsafe fn unmap_page(vaddr: VirtAddr) -> MmResult<PhysAddr> {
    let phys = crate::mm::vmm::unmap_page(vaddr.as_u64())?;
    tlb::invalidate_page(vaddr);
    Ok(phys)
}

/// Verifica se é guard page fault
pub fn is_guard_page_fault(fault_addr: VirtAddr) -> bool {
    crate::mm::vmm::translate_addr(fault_addr.as_u64()).is_none()
}

/// Imprime estatísticas
pub fn print_stats() {
    crate::kinfo!(
        "(Mapper) Regiões=",
        MAPPER_STATS.regions_mapped.load(Ordering::Relaxed) as u64
    );
    crate::kinfo!(
        "(Mapper) Páginas=",
        MAPPER_STATS.pages_mapped.load(Ordering::Relaxed) as u64
    );
    crate::kinfo!(
        "(Mapper) Guards =",
        MAPPER_STATS.guard_pages.load(Ordering::Relaxed) as u64
    );
}
