//! # VirtIO-Sound Driver
//!
//! Driver de áudio paravirtualizado (VirtIO 1.1+).

use super::traits::{AudioBuffer, SoundCard, StreamConfig};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct VirtioSoundDriver;

impl Driver for VirtioSoundDriver {
    fn name(&self) -> &'static str {
        "VirtIO Sound Adapter"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Sound
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização VirtIO-Sound
        // 1. Negociar features (VIRTIO_SND_F_*)
        // 2. Configurar Virtqueues: Control, Event, Tx, Rx
        // 3. Enviar comando VIRTIO_SND_R_PCM_INFO para descobrir streams
        crate::kinfo!("(Sound/Virtio) Driver VirtIO-Sound iniciado.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

impl SoundCard for VirtioSoundDriver {
    fn name(&self) -> &'static str {
        "virtio-snd"
    }

    fn supports_format(&self, _config: &StreamConfig) -> bool {
        // TODO: Checar bits retornados pelo PCM_INFO
        true
    }

    fn set_volume(&self, _volume: f32) -> Result<(), &'static str> {
        // TODO: Enviar comando VIRTIO_SND_R_PCM_SET_PARAMS (gain)
        Ok(())
    }

    fn write_stream(&self, _buffer: &AudioBuffer) -> Result<(), &'static str> {
        // TODO: Dividir em chunks e colocar na TX Virtqueue
        Ok(())
    }

    fn start_playback(&self) -> Result<(), &'static str> {
        // TODO: Enviar VIRTIO_SND_R_PCM_START
        Ok(())
    }

    fn stop_playback(&self) -> Result<(), &'static str> {
        // TODO: Enviar VIRTIO_SND_R_PCM_STOP
        Ok(())
    }
}
