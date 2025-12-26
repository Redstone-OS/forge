use super::{PhysAddr, VirtAddr};
use crate::mm::config::HHDM_BASE;

/// Converte endereço físico para virtual (Direct Map)
///
/// Assume que a memória física está mapeada linearmente em `HHDM_BASE`.
#[inline]
pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys.as_u64() + HHDM_BASE as u64)
}

/// Converte endereço virtual para físico (Direct Map)
///
/// # Safety
/// O chamador deve garantir que o endereço virtual pertence ao HHDM.
#[inline]
pub fn virt_to_phys(virt: VirtAddr) -> Option<PhysAddr> {
    let v = virt.as_u64();
    let base = HHDM_BASE as u64;

    if v >= base {
        Some(PhysAddr::new(v - base))
    } else {
        None
    }
}

/// Helper para converter ponteiro raw para PhysAddr
/// (Útil para interagir com APIs legadas que usam pointers)
pub fn ptr_to_phys<T>(ptr: *const T) -> Option<PhysAddr> {
    virt_to_phys(VirtAddr::new(ptr as u64))
}

/// Verifica se um endereço físico é acessível via HHDM
pub fn is_phys_accessible(phys: PhysAddr) -> bool {
    // Por enquanto, assumimos que tudo abaixo do limite inicial mapeado é acessível
    use crate::mm::config::IDENTITY_MAP_LIMIT;
    phys.as_usize() < IDENTITY_MAP_LIMIT
}
