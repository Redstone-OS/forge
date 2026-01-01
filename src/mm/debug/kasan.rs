//! # KASAN - Kernel Address Sanitizer

static ENABLED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

pub fn enable() {
    ENABLED.store(true, core::sync::atomic::Ordering::Release);
    crate::kinfo!("(KASAN) Enabled");
}

pub fn disable() {
    ENABLED.store(false, core::sync::atomic::Ordering::Release);
}

pub fn is_enabled() -> bool {
    ENABLED.load(core::sync::atomic::Ordering::Acquire)
}

pub fn report_invalid_access(addr: u64, _size: usize, is_write: bool) {
    if !is_enabled() {
        return;
    }
    if is_write {
        crate::kerror!("(KASAN) Invalid write at", addr);
    } else {
        crate::kerror!("(KASAN) Invalid read at", addr);
    }
}

pub fn check_memory_access(_addr: u64, _size: usize, _is_write: bool) -> bool {
    true
}
