//! # Higher Half Direct Map (HHDM)
//!
//! Mapeamento direto de toda RAM física no higher half.
//! Permite acesso rápido à memória física: virt = HHDM_BASE + phys

use crate::core::boot::BootInfo;
use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::config::HHDM_BASE;

/// Offset do HHDM (configurado pelo bootloader)
static mut HHDM_OFFSET: u64 = 0;

/// Flag indicando se HHDM está inicializado
static mut HHDM_INITIALIZED: bool = false;

/// Inicializa HHDM com offset do bootloader
pub unsafe fn init(boot_info: &'static BootInfo) {
    // O bootloader (Limine) configura o HHDM offset
    HHDM_OFFSET = boot_info.hhdm_offset.unwrap_or(HHDM_BASE);
    HHDM_INITIALIZED = true;

    crate::kinfo!("(RMM/HHDM) Base: 0x{:016x}", HHDM_OFFSET);
}

/// Retorna o offset do HHDM
#[inline]
pub fn offset() -> u64 {
    unsafe { HHDM_OFFSET }
}

/// Verifica se HHDM está inicializado
#[inline]
pub fn is_initialized() -> bool {
    unsafe { HHDM_INITIALIZED }
}

/// Converte endereço físico para virtual
#[inline]
pub fn phys_to_virt(phys: u64) -> u64 {
    debug_assert!(unsafe { HHDM_INITIALIZED }, "HHDM not initialized");
    unsafe { HHDM_OFFSET + phys }
}

/// Converte endereço físico para ponteiro
#[inline]
pub fn phys_to_virt_ptr<T>(phys: u64) -> *mut T {
    phys_to_virt(phys) as *mut T
}

/// Converte PhysAddr para VirtAddr
#[inline]
pub fn phys_to_virt_addr(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys_to_virt(phys.as_u64()))
}

/// Converte endereço virtual HHDM para físico
#[inline]
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    let offset = unsafe { HHDM_OFFSET };
    if virt >= offset {
        Some(virt - offset)
    } else {
        None
    }
}

/// Converte VirtAddr para PhysAddr (se for HHDM)
#[inline]
pub fn virt_to_phys_addr(virt: VirtAddr) -> Option<PhysAddr> {
    virt_to_phys(virt.as_u64()).map(PhysAddr::new)
}

/// Verifica se endereço é HHDM
#[inline]
pub fn is_hhdm(virt: u64) -> bool {
    let offset = unsafe { HHDM_OFFSET };
    virt >= offset && virt < offset + crate::rmm::config::HHDM_SIZE
}

/// Zera uma página física via HHDM
pub unsafe fn zero_page(phys: PhysAddr) {
    let ptr = phys_to_virt_ptr::<u8>(phys.as_u64());
    core::ptr::write_bytes(ptr, 0, crate::rmm::config::PAGE_SIZE);
}

/// Copia uma página física para outra via HHDM
pub unsafe fn copy_page(src: PhysAddr, dst: PhysAddr) {
    let src_ptr = phys_to_virt_ptr::<u8>(src.as_u64());
    let dst_ptr = phys_to_virt_ptr::<u8>(dst.as_u64());
    core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, crate::rmm::config::PAGE_SIZE);
}
