//! Memory Management
//!
//! Gerenciamento de memória física e virtual.

pub mod heap;
pub mod pmm;
pub mod vmm;

pub use pmm::{Frame, PhysicalMemoryManager, FRAME_SIZE};
pub use vmm::VirtualMemoryManager;
