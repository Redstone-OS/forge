//! # Sound Device Traits and Types
//!
//! Define as interfaces e tipos fundamentais para dispositivos de som.
//!
//! ## Arquitetura RDS:
//! Todos os drivers de som implementam `SoundDevice` e são registrados
//! no subsistema central via `sound::register_device()`.
//!
//! ## Tipos de Dispositivos:
//! - **HDA**: Intel High Definition Audio (moderno)
//! - **AC'97**: Audio Codec '97 (legado)
//! - **VirtIO**: Paravirtualizado (QEMU/KVM)
//! - **USB Audio**: Class driver (futuro)

// TODO: Revisar no futuro
#[allow(unused_imports)]
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// TIPOS DE DADOS DE ÁUDIO
// =============================================================================

/// Formatos de amostra suportados (PCM).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// 8-bit Unsigned (0-255).
    U8,
    /// 16-bit Signed Little Endian.
    S16LE,
    /// 16-bit Signed Big Endian.
    S16BE,
    /// 24-bit Signed Little Endian (packed).
    S24LE,
    /// 24-bit Signed Little Endian (in 32-bit container).
    S24LE32,
    /// 32-bit Signed Little Endian.
    S32LE,
    /// 32-bit Floating Point.
    Float32,
}

impl SampleFormat {
    /// Retorna o tamanho em bytes de uma amostra.
    pub const fn bytes_per_sample(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::S16LE | Self::S16BE => 2,
            Self::S24LE => 3,
            Self::S24LE32 | Self::S32LE | Self::Float32 => 4,
        }
    }

    /// Retorna bits por sample.
    pub const fn bits_per_sample(&self) -> u8 {
        match self {
            Self::U8 => 8,
            Self::S16LE | Self::S16BE => 16,
            Self::S24LE | Self::S24LE32 => 24,
            Self::S32LE | Self::Float32 => 32,
        }
    }

    /// Verifica se é formato signed.
    pub const fn is_signed(&self) -> bool {
        !matches!(self, Self::U8)
    }
}

impl Default for SampleFormat {
    fn default() -> Self {
        Self::S16LE
    }
}

/// Configuração de stream de áudio.
#[derive(Debug, Clone, Copy)]
pub struct StreamConfig {
    /// Número de canais (1=Mono, 2=Stereo, etc).
    pub channels: u8,
    /// Taxa de amostragem em Hz (ex: 44100, 48000).
    pub sample_rate: u32,
    /// Formato da amostra.
    pub format: SampleFormat,
    /// Tamanho do buffer em frames (0 = default do driver).
    pub buffer_frames: u32,
    /// Período em frames (0 = default do driver).
    pub period_frames: u32,
}

impl StreamConfig {
    /// Configuração padrão: Stereo, 48kHz, S16LE.
    pub const fn default_playback() -> Self {
        Self {
            channels: 2,
            sample_rate: 48000,
            format: SampleFormat::S16LE,
            buffer_frames: 4096,
            period_frames: 1024,
        }
    }

    /// Bytes por frame (sample_size * channels).
    pub const fn bytes_per_frame(&self) -> usize {
        self.format.bytes_per_sample() * self.channels as usize
    }

    /// Calcula latência em microsegundos.
    pub fn latency_us(&self) -> u64 {
        if self.sample_rate == 0 {
            return 0;
        }
        (self.buffer_frames as u64 * 1_000_000) / self.sample_rate as u64
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self::default_playback()
    }
}

/// Direção do stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamDirection {
    /// Playback (saída de áudio).
    Playback,
    /// Capture (entrada de áudio).
    Capture,
}

/// Estado de um stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Stream parado.
    Stopped,
    /// Stream em execução.
    Running,
    /// Stream pausado.
    Paused,
    /// Stream em draining (finalizando buffers).
    Draining,
    /// Stream em erro.
    Error,
}

// =============================================================================
// ERROS DE ÁUDIO
// =============================================================================

/// Erros de operações de áudio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundError {
    /// Dispositivo não inicializado.
    NotInitialized,
    /// Dispositivo não habilitado.
    NotEnabled,
    /// Configuração de stream não suportada.
    InvalidConfig,
    /// Taxa de amostragem não suportada.
    UnsupportedSampleRate,
    /// Formato não suportado.
    UnsupportedFormat,
    /// Número de canais não suportado.
    UnsupportedChannels,
    /// Buffer de DMA cheio.
    BufferFull,
    /// Buffer de DMA vazio.
    BufferEmpty,
    /// Erro de underrun (playback).
    Underrun,
    /// Erro de overrun (capture).
    Overrun,
    /// Erro de hardware (timeout, etc).
    HardwareError,
    /// Codec não respondeu.
    CodecError,
    /// Recurso ocupado.
    Busy,
    /// Operação não suportada.
    NotSupported,
    /// Erro genérico.
    Unknown,
}

impl SoundError {
    /// Retorna descrição do erro.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotInitialized => "Device Not Initialized",
            Self::NotEnabled => "Device Not Enabled",
            Self::InvalidConfig => "Invalid Stream Configuration",
            Self::UnsupportedSampleRate => "Unsupported Sample Rate",
            Self::UnsupportedFormat => "Unsupported Sample Format",
            Self::UnsupportedChannels => "Unsupported Channel Count",
            Self::BufferFull => "DMA Buffer Full",
            Self::BufferEmpty => "DMA Buffer Empty",
            Self::Underrun => "Buffer Underrun",
            Self::Overrun => "Buffer Overrun",
            Self::HardwareError => "Hardware Error",
            Self::CodecError => "Codec Error",
            Self::Busy => "Resource Busy",
            Self::NotSupported => "Operation Not Supported",
            Self::Unknown => "Unknown Error",
        }
    }

    /// Verifica se é erro recuperável.
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::BufferFull | Self::BufferEmpty | Self::Underrun | Self::Overrun | Self::Busy
        )
    }
}

// =============================================================================
// CAPACIDADES DO DISPOSITIVO
// =============================================================================

/// Capacidades de um dispositivo de som.
#[derive(Debug, Clone, Default)]
pub struct SoundCapabilities {
    /// Taxas de amostragem suportadas.
    pub sample_rates: Vec<u32>,
    /// Formatos de amostra suportados.
    pub formats: Vec<SampleFormat>,
    /// Número mínimo de canais.
    pub min_channels: u8,
    /// Número máximo de canais.
    pub max_channels: u8,
    /// Suporta playback.
    pub playback: bool,
    /// Suporta capture.
    pub capture: bool,
    /// Suporta controle de volume por hardware.
    pub hw_volume: bool,
    /// Suporta mute por hardware.
    pub hw_mute: bool,
}

impl SoundCapabilities {
    /// Verifica se uma configuração é suportada.
    pub fn supports(&self, config: &StreamConfig) -> bool {
        self.sample_rates.contains(&config.sample_rate)
            && self.formats.contains(&config.format)
            && config.channels >= self.min_channels
            && config.channels <= self.max_channels
    }

    /// Capacidades padrão para HDA.
    pub fn hda_default() -> Self {
        Self {
            sample_rates: alloc::vec![
                8000, 11025, 16000, 22050, 32000, 44100, 48000, 96000, 192000
            ],
            formats: alloc::vec![
                SampleFormat::S16LE,
                SampleFormat::S24LE32,
                SampleFormat::S32LE,
            ],
            min_channels: 1,
            max_channels: 8,
            playback: true,
            capture: true,
            hw_volume: true,
            hw_mute: true,
        }
    }

    /// Capacidades padrão para AC'97.
    pub fn ac97_default() -> Self {
        Self {
            sample_rates: alloc::vec![8000, 11025, 16000, 22050, 44100, 48000],
            formats: alloc::vec![SampleFormat::S16LE],
            min_channels: 1,
            max_channels: 6,
            playback: true,
            capture: true,
            hw_volume: true,
            hw_mute: true,
        }
    }

    /// Capacidades padrão para VirtIO.
    pub fn virtio_default() -> Self {
        Self {
            sample_rates: alloc::vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 96000],
            formats: alloc::vec![
                SampleFormat::S16LE,
                SampleFormat::S32LE,
                SampleFormat::Float32
            ],
            min_channels: 1,
            max_channels: 8,
            playback: true,
            capture: true,
            hw_volume: true,
            hw_mute: false,
        }
    }
}

// =============================================================================
// ESTATÍSTICAS DO DISPOSITIVO
// =============================================================================

/// Estatísticas de um dispositivo de som.
#[derive(Debug, Clone, Copy, Default)]
pub struct SoundStats {
    /// Frames reproduzidos.
    pub frames_played: u64,
    /// Frames capturados.
    pub frames_captured: u64,
    /// Bytes transferidos (playback).
    pub bytes_played: u64,
    /// Bytes transferidos (capture).
    pub bytes_captured: u64,
    /// Contador de underruns.
    pub underruns: u32,
    /// Contador de overruns.
    pub overruns: u32,
    /// Contador de erros de hardware.
    pub hw_errors: u32,
    /// Interrupções processadas.
    pub interrupts: u64,
}

// =============================================================================
// TRAIT PRINCIPAL: SOUND DEVICE
// =============================================================================

/// Interface base para dispositivos de som.
///
/// Todo driver de áudio deve implementar esta trait.
/// Integra-se com o RDS via `drivers::base::Driver`.
pub trait SoundDevice: Send + Sync {
    // -------------------------------------------------------------------------
    // Identificação
    // -------------------------------------------------------------------------

    /// Retorna nome do dispositivo (ex: "Intel HDA").
    fn name(&self) -> &'static str;

    /// Retorna nome do card (ex: "hw:0").
    fn card_name(&self) -> &'static str {
        self.name()
    }

    /// Retorna descrição/modelo.
    fn description(&self) -> &'static str {
        "Generic Sound Device"
    }

    // -------------------------------------------------------------------------
    // Capacidades
    // -------------------------------------------------------------------------

    /// Retorna capacidades do dispositivo.
    fn capabilities(&self) -> SoundCapabilities;

    /// Verifica se o dispositivo suporta uma configuração específica.
    fn is_config_supported(&self, config: &StreamConfig) -> bool {
        self.capabilities().supports(config)
    }

    // -------------------------------------------------------------------------
    // Controle de Stream
    // -------------------------------------------------------------------------

    /// Prepara um stream para reprodução/captura.
    ///
    /// Configura buffers DMA e parâmetros de hardware.
    fn prepare(&self, direction: StreamDirection, config: StreamConfig) -> Result<(), SoundError>;

    /// Inicia o stream preparado.
    fn start(&self, direction: StreamDirection) -> Result<(), SoundError>;

    /// Para o stream.
    fn stop(&self, direction: StreamDirection) -> Result<(), SoundError>;

    /// Pausa o stream (mantém buffers).
    fn pause(&self, direction: StreamDirection) -> Result<(), SoundError> {
        // Default: para completamente
        self.stop(direction)
    }

    /// Resume stream pausado.
    fn resume(&self, direction: StreamDirection) -> Result<(), SoundError> {
        // Default: inicia novamente
        self.start(direction)
    }

    /// Drena buffers pendentes (espera finalizar).
    fn drain(&self, direction: StreamDirection) -> Result<(), SoundError> {
        Ok(())
    }

    /// Retorna estado atual do stream.
    fn stream_state(&self, direction: StreamDirection) -> StreamState;

    // -------------------------------------------------------------------------
    // Transferência de Dados
    // -------------------------------------------------------------------------

    /// Escreve dados PCM no buffer de playback.
    ///
    /// ## Retorno:
    /// - Ok(frames): Número de frames escritos
    /// - Err: Erro de escrita
    fn write(&self, data: &[u8]) -> Result<usize, SoundError>;

    /// Lê dados PCM do buffer de capture.
    ///
    /// ## Retorno:
    /// - Ok(frames): Número de frames lidos
    /// - Err: Erro de leitura
    fn read(&self, buffer: &mut [u8]) -> Result<usize, SoundError> {
        Err(SoundError::NotSupported)
    }

    /// Retorna espaço disponível no buffer de playback (em frames).
    fn available_playback(&self) -> usize {
        0
    }

    /// Retorna dados disponíveis no buffer de capture (em frames).
    fn available_capture(&self) -> usize {
        0
    }

    // -------------------------------------------------------------------------
    // Controle de Volume
    // -------------------------------------------------------------------------

    /// Define o volume mestre (0-100%).
    fn set_master_volume(&self, volume: u8) -> Result<(), SoundError>;

    /// Obtém o volume mestre atual (0-100%).
    fn get_master_volume(&self) -> u8;

    /// Define mute.
    fn set_mute(&self, muted: bool) -> Result<(), SoundError> {
        if muted {
            self.set_master_volume(0)
        } else {
            Ok(())
        }
    }

    /// Verifica se está em mute.
    fn is_muted(&self) -> bool {
        self.get_master_volume() == 0
    }

    // -------------------------------------------------------------------------
    // Gerenciamento de Energia (RDS)
    // -------------------------------------------------------------------------

    /// Habilita o dispositivo (power up).
    fn enable(&self) -> Result<(), SoundError>;

    /// Desabilita o dispositivo (power down).
    fn disable(&self);

    /// Verifica se está habilitado.
    fn is_enabled(&self) -> bool;

    /// Suspende dispositivo (sleep).
    fn suspend(&self) -> Result<(), SoundError> {
        self.disable();
        Ok(())
    }

    /// Resume dispositivo de suspensão.
    fn resume_device(&self) -> Result<(), SoundError> {
        self.enable()
    }

    // -------------------------------------------------------------------------
    // Diagnóstico
    // -------------------------------------------------------------------------

    /// Retorna estatísticas do dispositivo.
    fn get_stats(&self) -> SoundStats {
        SoundStats::default()
    }

    /// Reseta estatísticas.
    fn reset_stats(&self) {}

    /// Verifica saúde do dispositivo.
    fn is_healthy(&self) -> bool {
        self.is_enabled()
    }
}

/// Tipo wrapper para armazenar qualquer dispositivo de som.
pub type SoundDeviceRef = Arc<dyn SoundDevice>;

// =============================================================================
// MIXER CHANNEL (FUTURO)
// =============================================================================

/// Canal do mixer de áudio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixerChannel {
    /// Volume mestre.
    Master,
    /// Volume de PCM.
    Pcm,
    /// Volume de entrada de linha.
    LineIn,
    /// Volume do microfone.
    Mic,
    /// Volume do CD.
    Cd,
    /// Volume de headphone.
    Headphone,
    /// Volume de speaker.
    Speaker,
}

/// Controle de volume de um canal.
#[derive(Debug, Clone, Copy)]
pub struct VolumeControl {
    /// Canal.
    pub channel: MixerChannel,
    /// Volume esquerdo (0-100).
    pub left: u8,
    /// Volume direito (0-100).
    pub right: u8,
    /// Em mute.
    pub muted: bool,
}

impl VolumeControl {
    /// Volume mono (média L/R).
    pub fn mono(&self) -> u8 {
        (self.left as u16 + self.right as u16) as u8 / 2
    }

    /// Define volume mono (L = R).
    pub fn set_mono(&mut self, volume: u8) {
        self.left = volume;
        self.right = volume;
    }
}
