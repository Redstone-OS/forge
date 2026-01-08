//! # VirtIO Sound Driver
//!
//! Driver para dispositivos de som paravirtualizados VirtIO (virtio-snd).
//! Ideal para máquinas virtuais QEMU/KVM com desempenho otimizado.
//!
//! ## Spec: OASIS VirtIO v1.2 - Section 5.14 Sound Device

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::sound::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_SOUND_DEVICE: u16 = 0x1059;

/// Driver VirtIO Sound para o RDS.
pub struct VirtioSoundDriver;

impl Driver for VirtioSoundDriver {
    fn name(&self) -> &'static str {
        "virtio-sound"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::Audio
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        let is_virtio = dev.vendor_id == VIRTIO_VENDOR
            && (dev.device_id == VIRTIO_SOUND_DEVICE || dev.device_id == 0x1040 + 25);

        if !is_virtio {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(VirtIO Sound) Dispositivo: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );
        crate::drivers::sound::register_device(Arc::new(VirtioSoundDevice::new()));
        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::drivers::sound::unregister_device("VirtIO Sound");
        Ok(())
    }
}

// TODO: Revisar no futuro
#[allow(unused)]
struct VirtioState {
    enabled: bool,
    volume: u8,
    muted: bool,
    playback: StreamState,
    capture: StreamState,
    config: Option<StreamConfig>,
    stats: SoundStats,
}

impl Default for VirtioState {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 100,
            muted: false,
            playback: StreamState::Stopped,
            capture: StreamState::Stopped,
            config: None,
            stats: SoundStats::default(),
        }
    }
}

pub struct VirtioSoundDevice {
    state: Spinlock<VirtioState>,
}

impl VirtioSoundDevice {
    pub fn new() -> Self {
        Self {
            state: Spinlock::new(VirtioState::default()),
        }
    }
}

impl SoundDevice for VirtioSoundDevice {
    fn name(&self) -> &'static str {
        "VirtIO Sound"
    }
    fn card_name(&self) -> &'static str {
        "virtio:0"
    }
    fn description(&self) -> &'static str {
        "VirtIO Sound (Paravirtualized)"
    }
    fn capabilities(&self) -> SoundCapabilities {
        SoundCapabilities::virtio_default()
    }

    fn prepare(&self, dir: StreamDirection, cfg: StreamConfig) -> Result<(), SoundError> {
        let mut s = self.state.lock();
        if !s.enabled {
            return Err(SoundError::NotEnabled);
        }
        match dir {
            StreamDirection::Playback => {
                s.config = Some(cfg);
                s.playback = StreamState::Stopped;
            }
            StreamDirection::Capture => {
                s.capture = StreamState::Stopped;
            }
        }
        Ok(())
    }

    fn start(&self, dir: StreamDirection) -> Result<(), SoundError> {
        let mut s = self.state.lock();
        if !s.enabled {
            return Err(SoundError::NotEnabled);
        }
        match dir {
            StreamDirection::Playback => s.playback = StreamState::Running,
            StreamDirection::Capture => s.capture = StreamState::Running,
        }
        Ok(())
    }

    fn stop(&self, dir: StreamDirection) -> Result<(), SoundError> {
        let mut s = self.state.lock();
        match dir {
            StreamDirection::Playback => s.playback = StreamState::Stopped,
            StreamDirection::Capture => s.capture = StreamState::Stopped,
        }
        Ok(())
    }

    fn stream_state(&self, dir: StreamDirection) -> StreamState {
        let s = self.state.lock();
        match dir {
            StreamDirection::Playback => s.playback,
            StreamDirection::Capture => s.capture,
        }
    }

    fn write(&self, data: &[u8]) -> Result<usize, SoundError> {
        let mut s = self.state.lock();
        if !s.enabled {
            return Err(SoundError::NotEnabled);
        }
        if s.playback != StreamState::Running {
            return Err(SoundError::NotInitialized);
        }
        let cfg = s.config.as_ref().ok_or(SoundError::InvalidConfig)?;
        let frames = data.len() / cfg.bytes_per_frame();
        s.stats.frames_played += frames as u64;
        s.stats.bytes_played += data.len() as u64;
        Ok(frames)
    }

    fn set_master_volume(&self, vol: u8) -> Result<(), SoundError> {
        let mut s = self.state.lock();
        if !s.enabled {
            return Err(SoundError::NotEnabled);
        }
        s.volume = vol.min(100);
        Ok(())
    }

    fn get_master_volume(&self) -> u8 {
        self.state.lock().volume
    }

    fn enable(&self) -> Result<(), SoundError> {
        let mut s = self.state.lock();
        if !s.enabled {
            crate::kinfo!("(VirtIO Sound) Habilitado");
            s.enabled = true;
        }
        Ok(())
    }

    fn disable(&self) {
        let mut s = self.state.lock();
        s.playback = StreamState::Stopped;
        s.capture = StreamState::Stopped;
        s.enabled = false;
    }

    fn is_enabled(&self) -> bool {
        self.state.lock().enabled
    }
    fn get_stats(&self) -> SoundStats {
        self.state.lock().stats
    }
    fn reset_stats(&self) {
        self.state.lock().stats = SoundStats::default();
    }
}

pub fn init() {
    // crate::kinfo!("(VirtIO Sound) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(VirtioSoundDriver));
}

pub fn shutdown() {}
