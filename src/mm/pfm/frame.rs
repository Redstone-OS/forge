//! # Frame Info
//!
//! Metadados de um frame físico.

use super::Pid;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

// =============================================================================
// FRAME STATE
// =============================================================================

/// Estado de um frame físico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameState {
    Free,
    Owned { owner: Pid },
    Shared { ref_count: u32, original_owner: Pid },
    Kernel,
    Pinned { owner: Pid },
    Device,
}

impl FrameState {
    fn to_bits(&self) -> u64 {
        match self {
            FrameState::Free => 0,
            FrameState::Owned { owner } => 1 | ((*owner as u64) << 8),
            FrameState::Shared {
                ref_count,
                original_owner,
            } => 2 | ((*ref_count as u64) << 8) | ((*original_owner as u64) << 40),
            FrameState::Kernel => 3,
            FrameState::Pinned { owner } => 4 | ((*owner as u64) << 8),
            FrameState::Device => 5,
        }
    }

    fn from_bits(bits: u64) -> Self {
        match bits & 0xFF {
            0 => FrameState::Free,
            1 => FrameState::Owned {
                owner: ((bits >> 8) & 0xFFFF_FFFF) as Pid,
            },
            2 => FrameState::Shared {
                ref_count: ((bits >> 8) & 0xFFFF_FFFF) as u32,
                original_owner: ((bits >> 40) & 0xFF_FFFF) as Pid,
            },
            3 => FrameState::Kernel,
            4 => FrameState::Pinned {
                owner: ((bits >> 8) & 0xFFFF_FFFF) as Pid,
            },
            5 => FrameState::Device,
            _ => FrameState::Free,
        }
    }
}

// =============================================================================
// FRAME FLAGS
// =============================================================================

/// Flags de um frame físico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameFlags(u32);

impl FrameFlags {
    pub const USER: Self = Self(1 << 0);
    pub const DIRTY: Self = Self(1 << 1);
    pub const ACCESSED: Self = Self(1 << 2);

    pub const fn empty() -> Self {
        Self(0)
    }
    pub const fn bits(&self) -> u32 {
        self.0
    }
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for FrameFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

// =============================================================================
// RMAP DATA
// =============================================================================

#[derive(Debug)]
pub struct RMapData {
    data: [u64; 2],
}

impl RMapData {
    pub const fn new() -> Self {
        Self { data: [0, 0] }
    }

    pub fn add(&mut self, pte_addr: u64) {
        for slot in &mut self.data {
            if *slot == 0 {
                *slot = pte_addr;
                return;
            }
        }
    }

    pub fn remove(&mut self, pte_addr: u64) {
        for slot in &mut self.data {
            if *slot == pte_addr {
                *slot = 0;
                return;
            }
        }
    }

    pub fn clear(&mut self) {
        self.data = [0, 0];
    }
    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.data.iter().copied().filter(|&a| a != 0)
    }
    pub fn count(&self) -> usize {
        self.data.iter().filter(|&&a| a != 0).count()
    }
}

impl Default for RMapData {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// FRAME INFO
// =============================================================================

#[repr(C, align(32))]
pub struct FrameInfo {
    state_bits: AtomicU64,
    ref_count: AtomicU32,
    numa_node: u16,
    flags: AtomicU32,
    _pad: u16,
    rmap: RMapData,
}

impl FrameInfo {
    pub const fn new() -> Self {
        Self {
            state_bits: AtomicU64::new(0),
            ref_count: AtomicU32::new(0),
            numa_node: 0,
            flags: AtomicU32::new(0),
            _pad: 0,
            rmap: RMapData::new(),
        }
    }

    pub fn state(&self) -> FrameState {
        FrameState::from_bits(self.state_bits.load(Ordering::Acquire))
    }

    pub fn set_state(&self, state: FrameState) {
        self.state_bits.store(state.to_bits(), Ordering::Release);
    }

    pub fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::Acquire)
    }
    pub fn set_ref_count(&self, count: u32) {
        self.ref_count.store(count, Ordering::Release);
    }
    pub fn inc_ref_count(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::AcqRel) + 1
    }
    pub fn dec_ref_count(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::AcqRel) - 1
    }

    pub fn flags(&self) -> FrameFlags {
        FrameFlags(self.flags.load(Ordering::Acquire))
    }
    pub fn set_flags(&self, flags: FrameFlags) {
        self.flags.store(flags.bits(), Ordering::Release);
    }

    pub fn rmap_add(&mut self, pte_addr: u64) {
        self.rmap.add(pte_addr);
    }
    pub fn rmap_remove(&mut self, pte_addr: u64) {
        self.rmap.remove(pte_addr);
    }
    pub fn rmap_clear(&mut self) {
        self.rmap.clear();
    }
    pub fn rmap_iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.rmap.iter()
    }
    pub fn rmap_count(&self) -> usize {
        self.rmap.count()
    }
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for FrameInfo {}
unsafe impl Sync for FrameInfo {}
