//! Data Management Unit (DMU)
//!
//! Handles objects, object sets (filesystems), and transactions.

use super::spa::Spa;
use alloc::sync::Arc;

/// Object Set (roughly equivalent to a filesystem instance)
pub struct ObjSet {
    spa: Arc<Spa>,
}

/// Generic Object (file, dir, etc)
pub struct Object {
    object_id: u64,
    size: u64,
}

impl ObjSet {
    pub fn open(spa: Arc<Spa>) -> Self {
        Self { spa }
    }

    pub fn tx_assign(&self) -> u64 {
        // TODO: Assign transaction group
        0
    }
    
    pub fn object_create(&self) -> Object {
        // TODO: Create object
        Object { object_id: 0, size: 0 }
    }
}
