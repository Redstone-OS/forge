//! # Memory Dump
//!
//! Funções para dump de informações de memória.
//!
//! ## Funcionalidades
//!
//! - Dump de informações gerais de memória
//! - Dump de frame específico
//! - Dump de zona
//! - Hexdump de região de memória

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::phys::{free_count, get_owner, FrameOwner};
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;
use alloc::format;

// =============================================================================
// Memory Info Dump
// =============================================================================

/// Dump informações gerais de memória
pub fn dump_memory_info() {
    crate::kinfo!("=== Memory Information ===");

    // HHDM info
    let hhdm_offset = hhdm::offset();
    crate::kinfo!("HHDM Offset: 0x{:016x}", hhdm_offset);

    // Frame count
    let free = free_count();
    crate::kinfo!(
        "Free Frames: {} ({} MB)",
        free,
        (free * PAGE_SIZE) / (1024 * 1024)
    );

    // Page size
    crate::kinfo!("Page Size: {} bytes", PAGE_SIZE);

    // Kernel regions
    dump_kernel_regions();
}

/// Dump regiões do kernel
fn dump_kernel_regions() {
    crate::kinfo!("Kernel Regions:");
    crate::kinfo!("  HHDM Base:    0x{:016x}", crate::rmm::config::HHDM_BASE);
    crate::kinfo!("  Heap Base:    0x{:016x}", crate::rmm::config::HEAP_BASE);
    crate::kinfo!(
        "  Driver Zone:  0x{:016x}",
        crate::rmm::config::DRIVER_ZONE_BASE
    );
    crate::kinfo!(
        "  DMA Pool:     0x{:016x}",
        crate::rmm::config::DMA_POOL_BASE
    );
    crate::kinfo!("  Kernel Stack: 0x{:016x}", crate::rmm::config::KSTACK_BASE);
}

// =============================================================================
// Frame Dump
// =============================================================================

/// Dump de um frame específico
pub fn dump_frame(phys: u64) {
    let phys_addr = PhysAddr::new(phys);

    crate::kinfo!("=== Frame @ 0x{:016x} ===", phys);

    if let Some(owner) = get_owner(phys_addr) {
        let owner_str = format!("{:?}", owner);
        crate::kinfo!("  Owner: {}", owner_str.as_str());

        // Informações adicionais baseadas no owner
        match owner {
            FrameOwner::Free => {
                crate::kinfo!("  Status: Available for allocation");
            }
            FrameOwner::Kernel => {
                crate::kinfo!("  Status: Kernel-owned (page tables, stacks, etc)");
            }
            FrameOwner::Process { pid } => {
                crate::kinfo!("  Status: Process {} userspace memory", pid);
            }
            FrameOwner::Cache { inode } => {
                crate::kinfo!("  Status: Page cache for inode {}", inode);
            }
            FrameOwner::Pinned { owner } => {
                crate::kinfo!("  Status: Pinned by process {}", owner);
            }
            FrameOwner::Driver { id } => {
                crate::kinfo!("  Status: Driver {} DMA buffer", id);
            }
            FrameOwner::Shared => {
                crate::kinfo!("  Status: Shared (CoW or mmap)");
            }
            FrameOwner::Device => {
                crate::kinfo!("  Status: Device memory (MMIO)");
            }
            FrameOwner::Slab => {
                crate::kinfo!("  Status: Slab allocator");
            }
            FrameOwner::Buddy => {
                crate::kinfo!("  Status: Buddy allocator metadata");
            }
        }
    } else {
        crate::kinfo!("  Owner: Unknown (not tracked by FrameManager)");
    }

    // Zona
    let zone = Zone::for_address(phys);
    let zone_str = format!("{:?}", zone);
    crate::kinfo!("  Zone: {}", zone_str.as_str());

    // TODO: Adicionar refcount, flags, rmap quando APIs estiverem disponíveis
}

/// Dump múltiplos frames
pub fn dump_frames(start: u64, count: usize) {
    crate::kinfo!("=== Frames {} @ 0x{:016x} ===", count, start);

    for i in 0..count.min(16) {
        // Limita a 16 para não poluir log
        let phys = start + (i * PAGE_SIZE) as u64;
        let owner = get_owner(PhysAddr::new(phys));
        let owner_str = match owner {
            Some(FrameOwner::Free) => "Free",
            Some(FrameOwner::Kernel) => "Kernel",
            Some(FrameOwner::Process { .. }) => "Process",
            Some(FrameOwner::Cache { .. }) => "Cache",
            Some(FrameOwner::Pinned { .. }) => "Pinned",
            Some(FrameOwner::Driver { .. }) => "Driver",
            Some(FrameOwner::Shared) => "Shared",
            Some(FrameOwner::Device) => "Device",
            Some(FrameOwner::Slab) => "Slab",
            Some(FrameOwner::Buddy) => "Buddy",
            None => "???",
        };
        crate::kinfo!("  0x{:016x}: {}", phys, owner_str);
    }

    if count > 16 {
        crate::kinfo!("  ... and {} more", count - 16);
    }
}

// =============================================================================
// Zone Dump
// =============================================================================

pub fn dump_zone_info(zone: Zone) {
    let zone_str = format!("{:?}", zone);
    crate::kinfo!("=== Zone {} ===", zone_str.as_str());

    let (start, end) = zone.address_range();
    let size_mb = (end - start) / (1024 * 1024);

    crate::kinfo!("  Range: 0x{:016x} - 0x{:016x}", start, end);
    crate::kinfo!("  Size: {} MB", size_mb);

    // Total de frames na zona
    let total_frames = (end - start) as usize / PAGE_SIZE;
    crate::kinfo!("  Frames: {}", total_frames);

    // TODO: Adicionar estatísticas detalhadas quando disponíveis
}

/// Dump todas as zonas
pub fn dump_all_zones() {
    crate::kinfo!("=== Memory Zones ===");
    dump_zone_info(Zone::Dma);
    dump_zone_info(Zone::Dma32);
    dump_zone_info(Zone::Normal);
}

// =============================================================================
// Hexdump
// =============================================================================

/// Hexdump de região de memória física
///
/// # Safety
///
/// O endereço físico deve ser válido e mapeado via HHDM.
pub unsafe fn hexdump_phys(phys: u64, len: usize) {
    let virt = hhdm::phys_to_virt(phys) as *const u8;

    crate::kinfo!("=== Hexdump @ 0x{:016x} ({} bytes) ===", phys, len);

    let mut offset = 0;
    while offset < len {
        let line_len = (len - offset).min(16);
        let mut hex = alloc::string::String::with_capacity(48);
        let mut ascii = alloc::string::String::with_capacity(16);

        for i in 0..16 {
            if i < line_len {
                let byte = *virt.add(offset + i);
                use core::fmt::Write;
                let _ = write!(hex, "{:02x} ", byte);
                ascii.push(if byte.is_ascii_graphic() || byte == b' ' {
                    byte as char
                } else {
                    '.'
                });
            } else {
                hex.push_str("   ");
            }

            if i == 7 {
                hex.push(' ');
            }
        }

        crate::kinfo!("  {:08x}  {}|{}|", offset, hex.as_str(), ascii.as_str());
        offset += 16;
    }
}

/// Hexdump de região virtual
///
/// # Safety
///
/// O endereço virtual deve ser válido e acessível.
pub unsafe fn hexdump_virt(virt: u64, len: usize) {
    let ptr = virt as *const u8;

    crate::kinfo!("=== Hexdump @ 0x{:016x} ({} bytes) ===", virt, len);

    let mut offset = 0;
    while offset < len.min(256) {
        // Limita a 256 bytes
        let line_len = (len - offset).min(16);
        let mut hex = alloc::string::String::with_capacity(48);
        let mut ascii = alloc::string::String::with_capacity(16);

        for i in 0..16 {
            if i < line_len {
                let byte = *ptr.add(offset + i);
                use core::fmt::Write;
                let _ = write!(hex, "{:02x} ", byte);
                ascii.push(if byte.is_ascii_graphic() || byte == b' ' {
                    byte as char
                } else {
                    '.'
                });
            } else {
                hex.push_str("   ");
            }

            if i == 7 {
                hex.push(' ');
            }
        }

        crate::kinfo!("  {:08x}  {}|{}|", offset, hex.as_str(), ascii.as_str());
        offset += 16;
    }
}
