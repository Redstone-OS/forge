//! # Intel High Definition Audio (HDA)
//!
//! Driver específico para controladores Intel HD Audio (Azalia).

use super::traits::{AudioBuffer, SoundCard, StreamConfig};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct IntelHdaDriver;

impl Driver for IntelHdaDriver {
    fn name(&self) -> &'static str {
        "Intel HDA Controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Sound
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização detalhada Intel HDA
        // Implementação real da spec Intel High Definition Audio
        crate::kinfo!("(Sound/IntelHDA) Controlador Azalia detectado.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

impl SoundCard for IntelHdaDriver {
    fn name(&self) -> &'static str {
        "intel-hda"
    }

    fn supports_format(&self, _config: &StreamConfig) -> bool {
        true
    }

    fn set_volume(&self, _volume: f32) -> Result<(), &'static str> {
        Ok(())
    }

    fn write_stream(&self, _buffer: &AudioBuffer) -> Result<(), &'static str> {
        Ok(())
    }

    fn start_playback(&self) -> Result<(), &'static str> {
        Ok(())
    }

    fn stop_playback(&self) -> Result<(), &'static str> {
        Ok(())
    }
}
