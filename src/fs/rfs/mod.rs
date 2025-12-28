//! RedstoneFileSystem (RFS)
//!
//! Inspired by OpenZFS architecture.
//!
//! Layers:
//! - ZPL (ZFS POSIX Layer): Interface with VFS
//! - DMU (Data Management Unit): Transactional Object Store
//! - SPA (Storage Pool Allocator): Storage virtualization
//! - ARC (Adaptive Replacement Cache): Caching

pub mod arc;
pub mod dmu;
pub mod spa;
pub mod zpl;
