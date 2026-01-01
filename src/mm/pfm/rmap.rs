//! # Reverse Mappings (RMAP)
//!
//! Rastreia PTEs que apontam para um frame físico.

use super::PfmResult;
use crate::mm::PhysAddr;
use crate::sync::Spinlock;

#[derive(Debug, Clone, Copy)]
pub struct RMapEntry {
    pub aspace_id: u64,
    pub virt_addr: u64,
}

impl RMapEntry {
    pub const fn new(aspace_id: u64, virt_addr: u64) -> Self {
        Self {
            aspace_id,
            virt_addr,
        }
    }
}

/// Adiciona entrada de rmap
pub fn add(phys: PhysAddr, aspace_id: u64, virt_addr: u64) -> PfmResult<()> {
    let pte_addr = (aspace_id << 48) | (virt_addr & 0x0000_FFFF_FFFF_F000);
    if let Some(frames) = super::get().lock().frames.as_mut() {
        let base = super::get().lock().base_phys;
        let index = ((phys.as_u64() - base) / crate::mm::config::PAGE_SIZE as u64) as usize;
        if index < frames.len() {
            frames[index].rmap_add(pte_addr);
        }
    }
    Ok(())
}

/// Remove entrada de rmap
pub fn remove(phys: PhysAddr, aspace_id: u64, virt_addr: u64) -> PfmResult<()> {
    let pte_addr = (aspace_id << 48) | (virt_addr & 0x0000_FFFF_FFFF_F000);
    if let Some(frames) = super::get().lock().frames.as_mut() {
        let base = super::get().lock().base_phys;
        let index = ((phys.as_u64() - base) / crate::mm::config::PAGE_SIZE as u64) as usize;
        if index < frames.len() {
            frames[index].rmap_remove(pte_addr);
        }
    }
    Ok(())
}

/// Conta mapeamentos de um frame
pub fn count(phys: PhysAddr) -> PfmResult<usize> {
    let pfm = super::get().lock();
    if let Some(frames) = &pfm.frames {
        let base = pfm.base_phys;
        let index = ((phys.as_u64() - base) / crate::mm::config::PAGE_SIZE as u64) as usize;
        if index < frames.len() {
            return Ok(frames[index].rmap_count());
        }
    }
    Ok(0)
}

/// Unmapeia de todos os address spaces
pub fn unmap_all(_phys: PhysAddr) -> PfmResult<usize> {
    // TODO: Iterar sobre entries e invalidar PTEs
    Ok(0)
}
