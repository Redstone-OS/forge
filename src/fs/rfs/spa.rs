//! Storage Pool Allocator (SPA)
//!
//! Manages storage pools and virtual devices (vdevs).

use alloc::vec::Vec;
use crate::sync::Mutex;

/// Storage Pool
pub struct Spa {
    name: alloc::string::String,
    vdevs: Mutex<Vec<Vdev>>,
}

/// Virtual Device
pub struct Vdev {
    id: u64,
    // TODO: abstraction for block device
}

impl Spa {
    pub fn new(name: &str) -> Self {
        Self {
            name: alloc::string::String::from(name),
            vdevs: Mutex::new(Vec::new()),
        }
    }

    pub fn open(&self) {
        // TODO: Open pool
    }

    pub fn import(&mut self) {
        // TODO: Import pool
    }
}
