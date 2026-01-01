//! # Memory Debug Utilities

pub mod kasan;

use crate::mm::PhysAddr;

pub fn dump_memory_info() {
    let stats = super::stats::snapshot();
    crate::kinfo!("=== Memory Info ===");
    crate::kinfo!("Total/Free/Kernel/User KB in stats", stats.total_physical);
}

pub fn dump_frame(phys: u64) {
    let pfm = super::pfm::get().lock();
    match pfm.get_state(PhysAddr::new(phys)) {
        Ok(state) => {
            crate::kinfo!("Frame state:", phys);
        }
        Err(_) => {
            crate::kwarn!("Frame error:", phys);
        }
    }
}

pub fn verify_pfm_integrity() -> bool {
    let pfm = super::pfm::get().lock();
    let stats = pfm.stats();

    let total = stats.free_frames
        + stats.kernel_frames
        + stats.user_frames
        + stats.shared_frames
        + stats.pinned_frames
        + stats.device_frames;

    if total != stats.total_frames {
        crate::kerror!("PFM integrity check failed");
        return false;
    }
    true
}
