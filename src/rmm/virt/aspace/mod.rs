//! # Address Space
//!
//! Gerenciamento de address space per-process (VMAs e page table root).

pub mod vma;

pub use vma::{MemoryIntent, Protection, VmaFlags, VMA};

use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::error::{RmmError, RmmResult};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Address Space de um processo
pub struct AddressSpace {
    /// CR3 (endereço físico da PML4)
    pml4: PhysAddr,
    /// VMAs ordenadas por endereço
    vmas: BTreeMap<u64, VMA>,
    /// PID do processo dono
    owner: u32,
}

impl AddressSpace {
    /// Cria novo address space
    pub fn new(owner: u32) -> RmmResult<Self> {
        // TODO: Alocar PML4 e copiar kernel mappings
        Ok(Self {
            pml4: PhysAddr::new(0),
            vmas: BTreeMap::new(),
            owner,
        })
    }

    /// Retorna endereço físico da PML4
    pub fn pml4(&self) -> PhysAddr {
        self.pml4
    }

    /// Retorna valor para CR3
    pub fn cr3(&self) -> u64 {
        self.pml4.as_u64()
    }

    /// Mapeia uma região
    pub fn map_region(
        &mut self,
        start: VirtAddr,
        size: usize,
        prot: Protection,
        intent: MemoryIntent,
    ) -> RmmResult<()> {
        // TODO: Verificar overlap e criar VMA
        let vma = VMA::new(start, size, prot, intent);
        self.vmas.insert(start.as_u64(), vma);
        Ok(())
    }

    /// Desmapeia uma região
    pub fn unmap_region(&mut self, start: VirtAddr, size: usize) -> RmmResult<()> {
        // TODO: Remover VMA e liberar frames
        self.vmas.remove(&start.as_u64());
        Ok(())
    }

    /// Encontra VMA contendo endereço
    pub fn find_vma(&self, addr: VirtAddr) -> Option<&VMA> {
        for (_, vma) in &self.vmas {
            if vma.contains(addr) {
                return Some(vma);
            }
        }
        None
    }

    /// Ativa este address space (switch CR3)
    pub unsafe fn activate(&self) {
        super::mapper::write_cr3(self.pml4);
    }
}
