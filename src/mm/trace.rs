//! # Memory Tracepoints

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static TRACING_ENABLED: AtomicBool = AtomicBool::new(false);

pub static ALLOC_EVENTS: AtomicU64 = AtomicU64::new(0);
pub static FREE_EVENTS: AtomicU64 = AtomicU64::new(0);
pub static FAULT_EVENTS: AtomicU64 = AtomicU64::new(0);
pub static MAP_EVENTS: AtomicU64 = AtomicU64::new(0);
pub static UNMAP_EVENTS: AtomicU64 = AtomicU64::new(0);

pub fn set_enabled(enabled: bool) {
    TRACING_ENABLED.store(enabled, Ordering::Release);
}

pub fn is_enabled() -> bool {
    TRACING_ENABLED.load(Ordering::Acquire)
}

pub fn trace_alloc(phys: u64, _size: usize) {
    if !is_enabled() {
        return;
    }
    ALLOC_EVENTS.fetch_add(1, Ordering::Relaxed);
    // Log disabled for now
}

pub fn trace_free(_phys: u64) {
    if !is_enabled() {
        return;
    }
    FREE_EVENTS.fetch_add(1, Ordering::Relaxed);
}

pub fn trace_fault(_virt: u64, _error: u64) {
    if !is_enabled() {
        return;
    }
    FAULT_EVENTS.fetch_add(1, Ordering::Relaxed);
}

pub fn trace_map(_virt: u64, _phys: u64) {
    if !is_enabled() {
        return;
    }
    MAP_EVENTS.fetch_add(1, Ordering::Relaxed);
}

pub fn trace_unmap(_virt: u64) {
    if !is_enabled() {
        return;
    }
    UNMAP_EVENTS.fetch_add(1, Ordering::Relaxed);
}

#[derive(Debug, Clone, Copy)]
pub struct TraceStats {
    pub allocs: u64,
    pub frees: u64,
    pub faults: u64,
    pub maps: u64,
    pub unmaps: u64,
}

pub fn stats() -> TraceStats {
    TraceStats {
        allocs: ALLOC_EVENTS.load(Ordering::Relaxed),
        frees: FREE_EVENTS.load(Ordering::Relaxed),
        faults: FAULT_EVENTS.load(Ordering::Relaxed),
        maps: MAP_EVENTS.load(Ordering::Relaxed),
        unmaps: UNMAP_EVENTS.load(Ordering::Relaxed),
    }
}
