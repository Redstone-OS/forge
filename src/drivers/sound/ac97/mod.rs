//! # AC'97 Audio Driver
//!
//! Driver para controladores de áudio Legacy AC'97.

use super::traits::{AudioBuffer, SoundCard, StreamConfig};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct Ac97Driver;

impl Driver for Ac97Driver {
    fn name(&self) -> &'static str {
        "AC'97 Audio Controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Sound
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização AC97
        // 1. Mapear NAMBAR/NABMBAR (Native Audio Mixer/Bus Mastering IO Bars)
        // 2. Resetar via registro de controle NABM
        // 3. Detectar codec primário
        crate::kinfo!("(Sound/AC97) Controlador Legacy AC97 detectado (stub).");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

impl SoundCard for Ac97Driver {
    fn name(&self) -> &'static str {
        "ac97"
    }

    fn supports_format(&self, config: &StreamConfig) -> bool {
        // AC97 geralmente é fixo em 48kHz 16-bit estéreo
        config.sample_rate == 48000 && config.channels == 2
    }

    fn set_volume(&self, _volume: f32) -> Result<(), &'static str> {
        // TODO: Escrever no registrador Master Volume (Offset 0x02)
        Ok(())
    }

    fn write_stream(&self, _buffer: &AudioBuffer) -> Result<(), &'static str> {
        // TODO: Preencher Buffer Descriptor List do AC97
        Ok(())
    }

    fn start_playback(&self) -> Result<(), &'static str> {
        // TODO: Habilitar bit 0 do Control Register
        Ok(())
    }

    fn stop_playback(&self) -> Result<(), &'static str> {
        Ok(())
    }
}
