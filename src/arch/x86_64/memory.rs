use core::arch::asm;

/// Retorna o valor atual de CR3 (Endereço físico da PML4).
#[inline]
pub unsafe fn cr3() -> u64 {
    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    cr3
}
