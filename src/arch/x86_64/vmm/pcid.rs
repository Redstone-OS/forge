//! # PCID Management (x86_64)

use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

pub const MAX_PCID: u16 = 4096;
pub const KERNEL_PCID: u16 = 0;

static PCID_ENABLED: AtomicBool = AtomicBool::new(false);
static NEXT_PCID: AtomicU16 = AtomicU16::new(1);

pub fn init() {
    if is_supported() {
        enable();
        crate::kinfo!("(PCID) Enabled");
    } else {
        crate::kinfo!("(PCID) Not supported");
    }
}

pub fn is_supported() -> bool {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let ecx: u32;
        core::arch::asm!(
            "mov eax, 1",
            "cpuid",
            out("ecx") ecx,
            out("eax") _,
            out("edx") _,
            options(nostack, nomem)
        );
        ecx & (1 << 17) != 0
    }
    #[cfg(not(target_arch = "x86_64"))]
    false
}

fn enable() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | (1 << 17));
    }
    PCID_ENABLED.store(true, Ordering::Release);
}

pub fn alloc() -> u16 {
    if !PCID_ENABLED.load(Ordering::Acquire) {
        return 0;
    }
    let pcid = NEXT_PCID.fetch_add(1, Ordering::SeqCst);
    if pcid >= MAX_PCID {
        NEXT_PCID.store(1, Ordering::SeqCst);
        1
    } else {
        pcid
    }
}

pub fn free(_pcid: u16) {}

pub fn make_cr3(pml4_phys: u64, pcid: u16) -> u64 {
    if PCID_ENABLED.load(Ordering::Acquire) {
        (pml4_phys & !0xFFF) | (pcid as u64 & 0xFFF)
    } else {
        pml4_phys
    }
}

pub fn invalidate(_pcid: u16, _addr: u64) {
    // TODO: INVPCID instruction
}
