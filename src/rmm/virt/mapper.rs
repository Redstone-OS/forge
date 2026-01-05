//! # Page Table Mapper
//!
//! Manipulação de page tables x86_64 (PML4, PDPT, PD, PT).

use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::config::*;
use crate::rmm::error::{RmmError, RmmResult};

/// Flags de mapeamento
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MapFlags(u64);

impl MapFlags {
    pub const NONE: Self = Self(0);
    pub const PRESENT: Self = Self(PTE_PRESENT);
    pub const WRITABLE: Self = Self(PTE_WRITABLE);
    pub const USER: Self = Self(PTE_USER);
    pub const NO_CACHE: Self = Self(PTE_NO_CACHE);
    pub const HUGE: Self = Self(PTE_HUGE);
    pub const GLOBAL: Self = Self(PTE_GLOBAL);
    pub const NO_EXEC: Self = Self(PTE_NO_EXEC);

    pub const KERNEL_RO: Self = Self(PTE_PRESENT | PTE_GLOBAL);
    pub const KERNEL_RW: Self = Self(PTE_PRESENT | PTE_WRITABLE | PTE_GLOBAL);
    pub const KERNEL_RX: Self = Self(PTE_PRESENT | PTE_GLOBAL);
    pub const USER_RW: Self = Self(PTE_PRESENT | PTE_WRITABLE | PTE_USER);
    pub const USER_RX: Self = Self(PTE_PRESENT | PTE_USER);

    #[inline]
    pub const fn bits(&self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Mapeia uma página virtual para física
pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: MapFlags) -> RmmResult<()> {
    if !virt.is_page_aligned() || !phys.is_page_aligned() {
        return Err(RmmError::NotAligned);
    }

    // TODO: Implementar mapeamento completo
    // 1. Obter PML4 atual (CR3)
    // 2. Criar tabelas intermediárias se necessário
    // 3. Definir PTE final
    // 4. Flush TLB

    Ok(())
}

/// Remove mapeamento de página
pub fn unmap_page(virt: VirtAddr) -> RmmResult<()> {
    if !virt.is_page_aligned() {
        return Err(RmmError::NotAligned);
    }

    // TODO: Implementar unmap
    // 1. Encontrar PTE
    // 2. Limpar entrada
    // 3. Flush TLB

    Ok(())
}

/// Traduz endereço virtual para físico
pub fn translate(virt: VirtAddr) -> Option<PhysAddr> {
    // TODO: Implementar page walk
    // 1. Ler PML4[virt.pml4_index()]
    // 2. Ler PDPT[virt.pdpt_index()]
    // 3. Ler PD[virt.pd_index()]
    // 4. Ler PT[virt.pt_index()]
    // 5. Extrair endereço físico

    None
}

/// Lê CR3 (endereço físico da PML4)
#[inline]
pub fn read_cr3() -> PhysAddr {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    }
    PhysAddr::new(cr3 & PTE_ADDR_MASK)
}

/// Escreve CR3 (troca page table root)
#[inline]
pub unsafe fn write_cr3(pml4: PhysAddr) {
    core::arch::asm!("mov cr3, {}", in(reg) pml4.as_u64(), options(nostack));
}
