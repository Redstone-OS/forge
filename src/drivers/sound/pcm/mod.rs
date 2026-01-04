//! # Generic PCM Driver (Software Fallback)
//!
//! Driver de som "dummy" que serve para testes e como fallback quando
//! nenhum hardware de áudio é detectado.

use super::traits::{AudioBuffer, SoundCard, StreamConfig};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct PcmDriver;

impl Driver for PcmDriver {
    fn name(&self) -> &'static str {
        "Generic PCM Dummy Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Sound
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização do driver Dummy
        // 1. Alocar buffers de memória circular virtuais
        // 2. Configurar timer para simular consumo de samples
        crate::kinfo!("(Sound/PCM) Driver Dummy inicializado.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

impl SoundCard for PcmDriver {
    fn name(&self) -> &'static str {
        "pcm-dummy"
    }

    fn supports_format(&self, _config: &StreamConfig) -> bool {
        // Suporta tudo (pois descarta tudo)
        true
    }

    fn set_volume(&self, _volume: f32) -> Result<(), &'static str> {
        Ok(())
    }

    fn write_stream(&self, _buffer: &AudioBuffer) -> Result<(), &'static str> {
        // STUB: Simular escrita (descarte ou log)
        // crate::kdebug!("(Sound/PCM) Escrevendo {} bytes (dummy)", buffer.data.len());
        Ok(())
    }

    fn start_playback(&self) -> Result<(), &'static str> {
        Ok(())
    }

    fn stop_playback(&self) -> Result<(), &'static str> {
        Ok(())
    }
}
