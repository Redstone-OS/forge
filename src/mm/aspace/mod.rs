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
    vmas: Vec<VMA>,
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
            vmas: Vec::new(),
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
        if size == 0 {
            return Err(ASpaceError::InvalidSize);
        }

        // Alinhar tamanho para cima
        let aligned_size = crate::klib::align_up(size, crate::mm::config::PAGE_SIZE);

        // Se houver hint, alinhar endereço para baixo
        let (target_addr, target_size) = if let Some(h) = hint {
            let page_size = crate::mm::config::PAGE_SIZE as u64;
            let start = h.align_down(page_size);
            let raw_size = size as u64;
            let end = (h.as_u64() + raw_size + (page_size - 1)) & !(page_size - 1);
            let final_size = (end - start.as_u64()) as usize;
            (start, final_size)
        } else {
            // If no hint, find a free region first
            let found_addr = self.find_free_region(None, aligned_size)?;
            (found_addr, aligned_size)
        };

        crate::ktrace!("(ASpace) map_region: alvo=", target_addr.as_u64());
        crate::ktrace!("(ASpace) map_region: tamanho=", target_size as u64);

        // Verificar conflito com o endereço final calculado
        // This call will either confirm target_addr is free or return an error
        let addr = self.find_free_region(Some(target_addr), target_size)?;

        // Criar VMA
        let vma = VMA::new(addr, addr.offset(target_size as u64), prot, flags, intent);

        self.vmas.push(vma);
        self.vmas.sort_by_key(|v| v.start);
        self.stats.vma_count += 1;
        self.stats.mapped_pages += target_size as u64 / 4096;
        self.tlb_gen.fetch_add(1, Ordering::Release);

        Ok(addr)
    }

    pub fn unmap_region(&mut self, addr: VirtAddr, _size: usize) -> ASpaceResult<()> {
        let idx = self
            .vmas
            .iter()
            .position(|v| v.start == addr)
            .ok_or(ASpaceError::RegionNotFound)?;
        let vma = self.vmas.remove(idx);
        self.stats.vma_count = self.stats.vma_count.saturating_sub(1);
        self.tlb_gen.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn find_vma(&self, addr: VirtAddr) -> Option<VMA> {
        self.vmas
            .iter()
            .find(|v| addr >= v.start && addr < v.end)
            .cloned()
    }

    fn find_free_region(&self, hint: Option<VirtAddr>, size: usize) -> ASpaceResult<VirtAddr> {
        if let Some(addr) = hint {
            // Verificar se o endereço solicitado está livre
            let end = addr.as_u64() + size as u64;
            let mut conflict = false;
            for vma in self.vmas.iter() {
                if (addr.as_u64() < vma.end.as_u64()) && (end > vma.start.as_u64()) {
                    crate::kdebug!("(ASpace) Conflict detected with VMA:", vma.start.as_u64());
                    crate::kdebug!("(ASpace) VMA end:", vma.end.as_u64());
                    conflict = true;
                    break;
                }
            }
            if !conflict {
                return Ok(addr);
            }
            crate::kerror!("(ASpace) Sobreposicao de regiao para hint:", addr.as_u64());
            return Err(ASpaceError::RegionOverlap);
        }

        // Se não houver hint, procurar o primeiro gap disponível após 4GB
        // NOTA: Começamos em 4GB para evitar conflitos com a região do binário ELF (que costuma ficar em 4MB-1GB)
        let mut candidate = VirtAddr::new(0x0000_0001_0000_0000);

        // Como vmas está ordenado por start, podemos percorrer linearmente
        for vma in self.vmas.iter() {
            if vma.start >= candidate {
                let gap_size = vma.start.as_u64() - candidate.as_u64();
                if gap_size >= size as u64 {
                    return Ok(candidate);
                }
                candidate = vma.end;
            } else if vma.end > candidate {
                candidate = vma.end;
            }
        }

        // Verificar fim do espaço canônico de usuário (até 0x00007FFFFFFFFFFF)
        let end = candidate.as_u64() + size as u64;
        if end < 0x0000_7FFF_FFFF_0000 {
            Ok(candidate)
        } else {
            crate::kerror!(
                "(ASpace) find_free_region: MEMORIA INSUFICIENTE (Virtual). tam=",
                size as u64
            );
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
