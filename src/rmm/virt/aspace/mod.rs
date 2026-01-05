//! # Address Space
//!
//! Gerenciamento de Address Space per-process (VMAs e page table root).
//!
//! ## Conceitos
//!
//! - **AddressSpace**: Representa o espaço de endereçamento de um processo
//! - **VMA (Virtual Memory Area)**: Região contígua com mesma proteção/propriedades
//! - **Page Table Root**: PML4 específica do processo
//!
//! ## Layout de Address Space
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────┐
//! │                     AddressSpace                          │
//! ├───────────────────────────────────────────────────────────┤
//! │  PML4 (CR3)                                               │
//! │  ┌─────────────────────────────────────────────────────┐  │
//! │  │ Entradas 0-255: Userspace (cópia por processo)      │  │
//! │  │ Entradas 256-511: Kernel (compartilhado)            │  │
//! │  └─────────────────────────────────────────────────────┘  │
//! │                                                           │
//! │  VMAs (regiões mapeadas)                                  │
//! │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
//! │  │ .text    │ │ .data    │ │  heap    │ │  stack   │      │
//! │  │ RX       │ │ RW       │ │ RW       │ │ RW       │      │
//! │  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │
//! └───────────────────────────────────────────────────────────┘
//! ```

pub mod vma;

pub use vma::{MemoryIntent, Protection, Vma, VmaFlags};

use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::error::{RmmError, RmmResult};
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::{hhdm, mapper, tlb};
use crate::rmm::zone::Zone;
use crate::sync::Spinlock;

use alloc::collections::BTreeMap;
// Todo: Revisar
#[allow(unused)]
use alloc::vec::Vec;

/// ID de processo para address space
pub type Pid = u32;

// =============================================================================
// AddressSpace
// =============================================================================

/// Address Space de um processo
///
/// Contém a PML4 (page table root) e as VMAs (regiões de memória).
pub struct AddressSpace {
    /// CR3: Endereço físico da PML4
    pml4: PhysAddr,
    /// VMAs ordenadas por endereço inicial
    vmas: BTreeMap<u64, Vma>,
    /// PID do processo dono
    owner: Pid,
    /// Lock para modificações
    lock: Spinlock<()>,
    /// Contador de páginas mapeadas
    mapped_pages: usize,
    /// Contador de páginas alocadas (fisicamente)
    allocated_pages: usize,
    /// Próximo endereço livre para mmap
    mmap_base: u64,
    /// Endereço do heap break
    heap_break: u64,
    /// Base do heap
    heap_base: u64,
    /// Topo da stack
    stack_top: u64,
}

impl AddressSpace {
    /// Layout padrão de userspace
    const USER_MMAP_BASE: u64 = 0x0000_7000_0000_0000; // Base para mmap
    const USER_HEAP_BASE: u64 = 0x0000_0001_0000_0000; // 4 GB
    const USER_STACK_TOP: u64 = 0x0000_7FFF_FFFF_F000; // Quase no topo do user space

    /// Cria novo address space vazio
    ///
    /// Aloca uma nova PML4 e copia os mappings do kernel (entradas 256-511).
    pub fn new(owner: Pid) -> RmmResult<Self> {
        // Aloca frame para nova PML4
        let pml4_phys = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
            .ok_or(RmmError::OutOfMemory)?;

        // Copia entradas do kernel (256-511) da PML4 atual
        let current_pml4 = mapper::read_cr3();
        unsafe {
            let current = hhdm::phys_to_virt_ptr::<[u64; 512]>(current_pml4.as_u64());
            let new = hhdm::phys_to_virt_ptr::<[u64; 512]>(pml4_phys.as_u64());

            // Copia apenas entradas do kernel
            for i in 256..512 {
                (*new)[i] = (*current)[i];
            }
        }

        Ok(Self {
            pml4: pml4_phys,
            vmas: BTreeMap::new(),
            owner,
            lock: Spinlock::new(()),
            mapped_pages: 0,
            allocated_pages: 0,
            mmap_base: Self::USER_MMAP_BASE,
            heap_break: Self::USER_HEAP_BASE,
            heap_base: Self::USER_HEAP_BASE,
            stack_top: Self::USER_STACK_TOP,
        })
    }

    /// Cria clone do address space (para fork)
    ///
    /// Implementa Copy-on-Write: as páginas são compartilhadas inicialmente
    /// e copiadas quando uma delas é escrita.
    pub fn fork(&self, new_owner: Pid) -> RmmResult<Self> {
        let _guard = self.lock.lock();

        // Aloca nova PML4
        let new_pml4_phys = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
            .ok_or(RmmError::OutOfMemory)?;

        // Copia entradas do kernel
        unsafe {
            let current = hhdm::phys_to_virt_ptr::<[u64; 512]>(self.pml4.as_u64());
            let new = hhdm::phys_to_virt_ptr::<[u64; 512]>(new_pml4_phys.as_u64());

            for i in 256..512 {
                (*new)[i] = (*current)[i];
            }
        }

        // Clona VMAs e configura CoW
        let mut new_vmas = BTreeMap::new();
        for (addr, vma) in &self.vmas {
            let mut new_vma = vma.clone();

            // Se writável, marca como CoW
            if vma.protection.can_write() {
                new_vma.flags = new_vma.flags.union(VmaFlags::COW);
                // TODO: Marcar páginas como read-only e incrementar refcount
            }

            new_vmas.insert(*addr, new_vma);
        }

        Ok(Self {
            pml4: new_pml4_phys,
            vmas: new_vmas,
            owner: new_owner,
            lock: Spinlock::new(()),
            mapped_pages: self.mapped_pages,
            allocated_pages: 0, // CoW, ainda não alocou
            mmap_base: self.mmap_base,
            heap_break: self.heap_break,
            heap_base: self.heap_base,
            stack_top: self.stack_top,
        })
    }

    /// Retorna endereço físico da PML4
    #[inline]
    pub fn pml4(&self) -> PhysAddr {
        self.pml4
    }

    /// Retorna valor para CR3
    #[inline]
    pub fn cr3(&self) -> u64 {
        self.pml4.as_u64()
    }

    /// Retorna PID do owner
    #[inline]
    pub fn owner(&self) -> Pid {
        self.owner
    }

    /// Ativa este address space (switch CR3)
    ///
    /// # Safety
    ///
    /// O caller deve garantir que este é o momento adequado para switch.
    pub unsafe fn activate(&self) {
        mapper::write_cr3(self.pml4);
    }

    // -------------------------------------------------------------------------
    // VMA Management
    // -------------------------------------------------------------------------

    /// Mapeia uma região com VMA
    pub fn map_region(
        &mut self,
        start: VirtAddr,
        size: usize,
        prot: Protection,
        intent: MemoryIntent,
    ) -> RmmResult<()> {
        let _guard = self.lock.lock();

        // Verifica alinhamento
        if !start.is_page_aligned() || size % PAGE_SIZE != 0 {
            return Err(RmmError::NotAligned);
        }

        // Verifica overlap com VMAs existentes
        let end = start.as_u64() + size as u64;
        for (vma_start, vma) in &self.vmas {
            let vma_end = *vma_start + vma.size() as u64;
            if start.as_u64() < vma_end && end > *vma_start {
                return Err(RmmError::AlreadyMapped);
            }
        }

        // Cria VMA
        let vma = Vma::new(start, size, prot, intent);
        self.vmas.insert(start.as_u64(), vma);

        Ok(())
    }

    /// Mapeia região com backing físico imediato
    pub fn map_region_backed(
        &mut self,
        start: VirtAddr,
        size: usize,
        prot: Protection,
        intent: MemoryIntent,
    ) -> RmmResult<()> {
        self.map_region(start, size, prot, intent)?;

        // Aloca e mapeia páginas
        let page_count = size / PAGE_SIZE;
        for i in 0..page_count {
            let virt = start + (i * PAGE_SIZE) as u64;

            // Aloca frame físico
            let phys = phys::alloc(
                FrameOwner::Process { pid: self.owner },
                Zone::Normal,
                AllocFlags::USER,
            )
            .ok_or(RmmError::OutOfMemory)?;

            // Mapeia
            let flags = prot.to_map_flags();
            self.map_page_internal(virt, phys, flags)?;

            self.allocated_pages += 1;
        }

        self.mapped_pages += page_count;

        Ok(())
    }

    /// Remove mapeamento de região
    pub fn unmap_region(&mut self, start: VirtAddr, size: usize) -> RmmResult<()> {
        let _guard = self.lock.lock();

        // Remove VMA
        self.vmas.remove(&start.as_u64());

        // Unmapa páginas
        let page_count = size / PAGE_SIZE;
        for i in 0..page_count {
            let virt = start + (i * PAGE_SIZE) as u64;
            if let Some(phys) = self.translate(virt) {
                // Libera frame físico
                let _ = phys::free(phys, FrameOwner::Process { pid: self.owner });
                self.allocated_pages = self.allocated_pages.saturating_sub(1);
            }
        }

        self.mapped_pages = self.mapped_pages.saturating_sub(page_count);

        // Flush TLB
        tlb::flush_tlb_range(start, start + size as u64);

        Ok(())
    }

    /// Encontra VMA contendo endereço
    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        for (vma_start, vma) in &self.vmas {
            if addr.as_u64() >= *vma_start && addr.as_u64() < *vma_start + vma.size() as u64 {
                return Some(vma);
            }
        }
        None
    }

    /// Encontra VMA contendo endereço (mutável)
    pub fn find_vma_mut(&mut self, addr: VirtAddr) -> Option<&mut Vma> {
        for (vma_start, vma) in &mut self.vmas {
            if addr.as_u64() >= *vma_start && addr.as_u64() < *vma_start + vma.size() as u64 {
                return Some(vma);
            }
        }
        None
    }

    // -------------------------------------------------------------------------
    // Heap Management (brk)
    // -------------------------------------------------------------------------

    /// Expande ou contrai o heap (brk syscall)
    pub fn brk(&mut self, new_break: u64) -> RmmResult<u64> {
        // Nota: Não trava lock aqui porque &mut self já garante exclusividade

        if new_break < self.heap_base {
            return Ok(self.heap_break);
        }

        if new_break > self.heap_break {
            // Expandir heap
            let old_break = self.heap_break;
            let old_page = (old_break + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;
            let new_page = (new_break + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;

            // Mapeia novas páginas
            for page in old_page..new_page {
                let virt = VirtAddr::new(page * PAGE_SIZE as u64);
                let phys = phys::alloc(
                    FrameOwner::Process { pid: self.owner },
                    Zone::Normal,
                    AllocFlags::USER,
                )
                .ok_or(RmmError::OutOfMemory)?;

                let flags = Protection::RW.to_map_flags();
                self.map_page_internal(virt, phys, flags)?;
                self.allocated_pages += 1;
            }
        } else if new_break < self.heap_break {
            // Contrair heap
            let old_page = (self.heap_break + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;
            let new_page = (new_break + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;

            for page in new_page..old_page {
                let virt = VirtAddr::new(page * PAGE_SIZE as u64);
                if let Some(phys) = self.translate(virt) {
                    let _ = phys::free(phys, FrameOwner::Process { pid: self.owner });
                    self.allocated_pages = self.allocated_pages.saturating_sub(1);
                }
            }
        }

        self.heap_break = new_break;
        Ok(self.heap_break)
    }

    /// Retorna atual heap break
    pub fn get_brk(&self) -> u64 {
        self.heap_break
    }

    // -------------------------------------------------------------------------
    // mmap
    // -------------------------------------------------------------------------

    /// Encontra espaço livre para mmap
    pub fn find_free_region(&self, size: usize) -> Option<VirtAddr> {
        let size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1); // Alinha

        let mut candidate = self.mmap_base;

        for (vma_start, vma) in &self.vmas {
            let vma_end = *vma_start + vma.size() as u64;

            if candidate + size as u64 <= *vma_start {
                return Some(VirtAddr::new(candidate));
            }

            if vma_end > candidate {
                candidate = vma_end;
            }
        }

        // Verifica se cabe após última VMA
        if candidate + size as u64 <= Self::USER_STACK_TOP {
            return Some(VirtAddr::new(candidate));
        }

        None
    }

    // -------------------------------------------------------------------------
    // Page Table Operations
    // -------------------------------------------------------------------------

    /// Traduz endereço virtual para físico neste address space
    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        // Usa a PML4 deste address space
        unsafe {
            let pml4 = hhdm::phys_to_virt_ptr::<[u64; 512]>(self.pml4.as_u64());

            // PML4
            let pml4_idx = ((virt.as_u64() >> 39) & 0x1FF) as usize;
            let pml4_entry = (*pml4)[pml4_idx];
            if (pml4_entry & 1) == 0 {
                return None;
            }

            // PDPT
            let pdpt_phys = pml4_entry & 0x000F_FFFF_FFFF_F000;
            let pdpt = hhdm::phys_to_virt_ptr::<[u64; 512]>(pdpt_phys);
            let pdpt_idx = ((virt.as_u64() >> 30) & 0x1FF) as usize;
            let pdpt_entry = (*pdpt)[pdpt_idx];
            if (pdpt_entry & 1) == 0 {
                return None;
            }

            // PD
            let pd_phys = pdpt_entry & 0x000F_FFFF_FFFF_F000;
            let pd = hhdm::phys_to_virt_ptr::<[u64; 512]>(pd_phys);
            let pd_idx = ((virt.as_u64() >> 21) & 0x1FF) as usize;
            let pd_entry = (*pd)[pd_idx];
            if (pd_entry & 1) == 0 {
                return None;
            }

            // PT
            let pt_phys = pd_entry & 0x000F_FFFF_FFFF_F000;
            let pt = hhdm::phys_to_virt_ptr::<[u64; 512]>(pt_phys);
            let pt_idx = ((virt.as_u64() >> 12) & 0x1FF) as usize;
            let pt_entry = (*pt)[pt_idx];
            if (pt_entry & 1) == 0 {
                return None;
            }

            let phys = (pt_entry & 0x000F_FFFF_FFFF_F000) + (virt.as_u64() & 0xFFF);
            Some(PhysAddr::new(phys))
        }
    }

    /// Mapeia página neste address space
    fn map_page_internal(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: mapper::MapFlags,
    ) -> RmmResult<()> {
        // Implementação similar ao mapper global, mas usando self.pml4
        unsafe {
            let pml4 = hhdm::phys_to_virt_ptr::<[u64; 512]>(self.pml4.as_u64());

            let pml4_idx = ((virt.as_u64() >> 39) & 0x1FF) as usize;
            let pdpt = self.get_or_create_table(&mut (*pml4)[pml4_idx], flags)?;

            let pdpt_idx = ((virt.as_u64() >> 30) & 0x1FF) as usize;
            let pd = self.get_or_create_table(&mut (*pdpt)[pdpt_idx], flags)?;

            let pd_idx = ((virt.as_u64() >> 21) & 0x1FF) as usize;
            let pt = self.get_or_create_table(&mut (*pd)[pd_idx], flags)?;

            let pt_idx = ((virt.as_u64() >> 12) & 0x1FF) as usize;
            (*pt)[pt_idx] = phys.as_u64() | flags.bits();
        }

        Ok(())
    }

    /// Obtém ou cria tabela de página
    unsafe fn get_or_create_table(
        &mut self,
        entry: &mut u64,
        flags: mapper::MapFlags,
    ) -> RmmResult<*mut [u64; 512]> {
        if (*entry & 1) != 0 {
            // Já existe
            let phys = *entry & 0x000F_FFFF_FFFF_F000;
            Ok(hhdm::phys_to_virt_ptr(phys))
        } else {
            // Cria nova
            let new_table = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(RmmError::OutOfMemory)?;

            let intermediate_flags = if flags.is_user() {
                0x07 // Present + Writable + User
            } else {
                0x03 // Present + Writable
            };

            *entry = new_table.as_u64() | intermediate_flags;

            Ok(hhdm::phys_to_virt_ptr(new_table.as_u64()))
        }
    }

    // -------------------------------------------------------------------------
    // Estatísticas
    // -------------------------------------------------------------------------

    /// Número de páginas mapeadas
    pub fn mapped_pages(&self) -> usize {
        self.mapped_pages
    }

    /// Número de páginas alocadas fisicamente
    pub fn allocated_pages(&self) -> usize {
        self.allocated_pages
    }

    /// Memória virtual mapeada (em bytes)
    pub fn mapped_bytes(&self) -> usize {
        self.mapped_pages * PAGE_SIZE
    }

    /// Memória física alocada (em bytes)
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_pages * PAGE_SIZE
    }

    /// Número de VMAs
    pub fn vma_count(&self) -> usize {
        self.vmas.len()
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        // Libera todos os frames alocados
        for (_addr, vma) in &self.vmas {
            let page_count = vma.size() / PAGE_SIZE;
            for i in 0..page_count {
                let virt = vma.start + (i * PAGE_SIZE) as u64;
                if let Some(phys) = self.translate(virt) {
                    let _ = phys::free(phys, FrameOwner::Process { pid: self.owner });
                }
            }
        }

        // Libera page tables (exceto kernel)
        // TODO: Implementar liberação recursiva de page tables userspace

        // Libera PML4
        let _ = phys::free(self.pml4, FrameOwner::Kernel);
    }
}
