//! # Audio Subsystem Traits
//!
//! Abstrações comuns para dispositivos de áudio e buffers de som.

use alloc::vec::Vec;

/// Formato de áudio PCM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    PcmU8,    // 8-bit unsigned
    PcmS16Le, // 16-bit signed little-endian
    PcmS24Le, // 24-bit signed little-endian
    PcmS32Le, // 32-bit signed little-endian
}

/// Configuração de stream de áudio
#[derive(Debug, Clone, Copy)]
pub struct StreamConfig {
    pub channels: u8,
    pub sample_rate: u32,
    pub format: AudioFormat,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            channels: 2,
            sample_rate: 44100,
            format: AudioFormat::PcmS16Le,
        }
    }
}

/// Buffer de áudio contendo dados PCM brutos
pub struct AudioBuffer {
    pub data: Vec<u8>,
    pub config: StreamConfig,
}

/// Trait unificada para drivers de áudio
pub trait SoundCard {
    /// Nome amigável do dispositivo
    fn name(&self) -> &'static str;

    /// Verifica se o dispositivo suporta uma configuração específica
    fn supports_format(&self, config: &StreamConfig) -> bool;

    /// Ajusta o volume (0.0 a 1.0)
    fn set_volume(&self, volume: f32) -> Result<(), &'static str>;

    /// Escreve dados para o buffer de saída (reprodução)
    fn write_stream(&self, buffer: &AudioBuffer) -> Result<(), &'static str>;

    /// Inicia a reprodução
    fn start_playback(&self) -> Result<(), &'static str>;

    /// Para a reprodução
    fn stop_playback(&self) -> Result<(), &'static str>;
}
