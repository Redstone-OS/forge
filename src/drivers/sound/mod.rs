//! # Sound Drivers Subsystem
//!
//! This module orchestrates all audio drivers and registers them with the RDM.

pub mod ac97;
pub mod hda; // Mantido como wrapper genérico se necessário, ou alias
pub mod intel_hda;
pub mod mixer;
pub mod pcm;
pub mod traits;
pub mod virtio;

use crate::drivers::base::driver::Driver;
use alloc::sync::Arc;

/// Initialize and register all sound drivers.
pub fn init() {
    crate::kinfo!("(Sound) Initializing sound subsystem...");

    // 1. PCM Fallback
    crate::drivers::base::register_driver(Arc::new(pcm::PcmDriver) as Arc<dyn Driver>);

    // 2. AC'97 Legacy
    crate::drivers::base::register_driver(Arc::new(ac97::Ac97Driver) as Arc<dyn Driver>);

    // 3. Intel HDA
    crate::drivers::base::register_driver(Arc::new(intel_hda::IntelHdaDriver) as Arc<dyn Driver>);

    // 4. VirtIO Sound
    crate::drivers::base::register_driver(Arc::new(virtio::VirtioSoundDriver) as Arc<dyn Driver>);

    crate::kinfo!("(Sound) All sound drivers registered.");
}
