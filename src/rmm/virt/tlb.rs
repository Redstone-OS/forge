//! # TLB Management
//!
//! Gerenciamento de TLB e IPI shootdown para SMP.

use crate::rmm::addr::VirtAddr;

/// Flush TLB para um endereço
#[inline]
pub fn flush_tlb(virt: VirtAddr) {
    unsafe {
        core::arch::asm!(
            "invlpg [{}]",
            in(reg) virt.as_u64(),
            options(nostack, preserves_flags)
        );
    }
}

/// Flush TLB completo (reload CR3)
#[inline]
pub fn flush_tlb_all() {
    unsafe {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
    }
}

/// Flush range de páginas
pub fn flush_tlb_range(start: VirtAddr, end: VirtAddr) {
    let mut addr = start;
    while addr < end {
        flush_tlb(addr);
        addr = addr + crate::rmm::config::PAGE_SIZE;
    }
}

/// Envia IPI para TLB shootdown em todas as CPUs
///
/// TODO: Implementar quando SMP estiver ativo
pub fn shootdown_all() {
    // 1. Flush local
    flush_tlb_all();

    // 2. Enviar IPI para outras CPUs
    // TODO: Implementar IPI

    // 3. Aguardar ACK
    // TODO: Implementar wait
}

/// Envia IPI para TLB shootdown em uma CPU específica
pub fn shootdown_cpu(_cpu: usize, _virt: VirtAddr) {
    // TODO: Implementar IPI unicast
}
