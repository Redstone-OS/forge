//! # Huge Pages Support
//!
//! Suporte a Huge Pages (2MB e 1GB).

use crate::mm::{MapFlags, PhysAddr, VirtAddr};

/// Tamanho de huge page 2MB
pub const HUGE_2MB: u64 = 2 * 1024 * 1024;
/// Tamanho de huge page 1GB
pub const HUGE_1GB: u64 = 1024 * 1024 * 1024;

/// Verifica se endereço está alinhado a 2MB
#[inline]
pub fn is_2mb_aligned(addr: u64) -> bool {
    addr & (HUGE_2MB - 1) == 0
}

/// Verifica se endereço está alinhado a 1GB
#[inline]
pub fn is_1gb_aligned(addr: u64) -> bool {
    addr & (HUGE_1GB - 1) == 0
}

/// Tipo de huge page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugePageSize {
    Page2MB,
    Page1GB,
}

impl HugePageSize {
    pub fn size(&self) -> u64 {
        match self {
            HugePageSize::Page2MB => HUGE_2MB,
            HugePageSize::Page1GB => HUGE_1GB,
        }
    }
}

/// Mapeia uma huge page 2MB
pub fn map_2mb(virt: VirtAddr, phys: PhysAddr, flags: MapFlags) -> crate::mm::error::MmResult<()> {
    use crate::mm::config::*;

    if !is_2mb_aligned(virt.as_u64()) || !is_2mb_aligned(phys.as_u64()) {
        return Err(crate::mm::MmError::InvalidAddress);
    }

    // Construir flags de PTE com PAGE_HUGE
    let mut pte_flags = PAGE_PRESENT | PAGE_HUGE;
    if flags.contains(MapFlags::WRITABLE) {
        pte_flags |= PAGE_WRITABLE;
    }
    if flags.contains(MapFlags::USER) {
        pte_flags |= PAGE_USER;
    }
    if !flags.contains(MapFlags::EXECUTABLE) {
        pte_flags |= PAGE_NO_EXEC;
    }
    if flags.contains(MapFlags::GLOBAL) {
        pte_flags |= PAGE_GLOBAL;
    }

    // TODO: Implementar mapeamento real quando VMM estiver atualizado
    let _ = (virt, phys, pte_flags);

    Ok(())
}

/// Divide huge page em 4KB pages
pub fn split_2mb_to_4kb(virt: VirtAddr) -> crate::mm::error::MmResult<()> {
    // TODO: Implementar split
    let _ = virt;
    Ok(())
}

/// Merge 512 páginas 4KB em uma 2MB
pub fn merge_4kb_to_2mb(virt: VirtAddr) -> crate::mm::error::MmResult<()> {
    // TODO: Implementar merge
    let _ = virt;
    Ok(())
}
