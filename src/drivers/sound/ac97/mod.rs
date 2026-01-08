//! # AC'97 Audio Driver
//!
//! Driver para o codec de áudio legado AC'97 (Audio Codec '97).
//! Comum em hardware antigo e emuladores (QEMU com -soundhw ac97).
//!
//! ## Especificação:
//! Intel AC'97 Component Specification, Revision 2.3
//!
//! ## Arquitetura AC'97:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           AC'97 Controller              │
//! │  ┌─────────────────────────────────┐    │
//! │  │  Bus Master Registers           │    │
//! │  │  - PCM Out (Playback)           │    │
//! │  │  - PCM In (Capture)             │    │
//! │  │  - Mic In                       │    │
//! │  └─────────────────────────────────┘    │
//! │                  │                      │
//! │           AC-Link (serial)              │
//! │                  │                      │
//! │                  ▼                      │
//! │           AC'97 Codec                   │
//! │  ┌─────────────────────────────────┐    │
//! │  │  Mixer Registers                │    │
//! │  │  - Master Volume                │    │
//! │  │  - PCM Out Volume               │    │
//! │  │  - Record Select                │    │
//! │  └─────────────────────────────────┘    │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Limitações:
//! - Sample rate fixo ou limitado (tipicamente 48kHz)
//! - Apenas 16-bit signed
//! - Máximo 6 canais (5.1)
//!
//! ## Status:
//! - [x] Estrutura básica do driver (RDS)
//! - [x] Detecção PCI (class 0x04, subclass 0x01)
//! - [ ] Inicialização do codec
//! - [ ] Configuração de DMA
//! - [ ] Controle de mixer

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::sound::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

// =============================================================================
// CONSTANTES AC'97
// =============================================================================

/// PCI Class: Multimedia
const PCI_CLASS_MULTIMEDIA: u8 = 0x04;
/// PCI Subclass: Audio Device
const PCI_SUBCLASS_AUDIO: u8 = 0x01;

/// Intel ICH AC'97 (muito comum em QEMU)
const VENDOR_INTEL: u16 = 0x8086;
// TODO: Revisar no futuro
#[allow(unused)]
const DEVICE_ICH_AC97: u16 = 0x2415; // 82801AA AC'97

// Registros do Mixer AC'97 (via BAR0)
#[allow(dead_code)]
mod mixer {
    /// Reset Register
    pub const RESET: u16 = 0x00;
    /// Master Volume
    pub const MASTER_VOL: u16 = 0x02;
    /// Headphone Volume
    pub const HP_VOL: u16 = 0x04;
    /// Mono Volume
    pub const MONO_VOL: u16 = 0x06;
    /// PCM Out Volume
    pub const PCM_VOL: u16 = 0x18;
    /// Record Select
    pub const REC_SEL: u16 = 0x1A;
    /// Record Gain
    pub const REC_GAIN: u16 = 0x1C;
    /// General Purpose
    pub const GP: u16 = 0x20;
    /// Powerdown Ctrl/Stat
    pub const POWERDOWN: u16 = 0x26;
    /// Extended Audio ID
    pub const EXT_AUDIO_ID: u16 = 0x28;
    /// Extended Audio Ctrl
    pub const EXT_AUDIO_CTRL: u16 = 0x2A;
    /// PCM Front DAC Rate
    pub const PCM_FRONT_DAC_RATE: u16 = 0x2C;
    /// PCM LR ADC Rate
    pub const PCM_LR_ADC_RATE: u16 = 0x32;
    /// Vendor ID1
    pub const VENDOR_ID1: u16 = 0x7C;
    /// Vendor ID2
    pub const VENDOR_ID2: u16 = 0x7E;
}

// Registros do Bus Master (via BAR1)
#[allow(dead_code)]
mod busmaster {
    /// PCM In Buffer Descriptor Base
    pub const PI_BDBAR: u16 = 0x00;
    /// PCM In Current Index
    pub const PI_CIV: u16 = 0x04;
    /// PCM In Last Valid Index
    pub const PI_LVI: u16 = 0x05;
    /// PCM In Status
    pub const PI_SR: u16 = 0x06;
    /// PCM In Position in Current Buffer
    pub const PI_PICB: u16 = 0x08;
    /// PCM In Prefetch Index
    pub const PI_PIV: u16 = 0x0A;
    /// PCM In Control
    pub const PI_CR: u16 = 0x0B;

    /// PCM Out Buffer Descriptor Base
    pub const PO_BDBAR: u16 = 0x10;
    /// PCM Out Current Index
    pub const PO_CIV: u16 = 0x14;
    /// PCM Out Last Valid Index
    pub const PO_LVI: u16 = 0x15;
    /// PCM Out Status
    pub const PO_SR: u16 = 0x16;
    /// PCM Out Position in Current Buffer
    pub const PO_PICB: u16 = 0x18;
    /// PCM Out Prefetch Index
    pub const PO_PIV: u16 = 0x1A;
    /// PCM Out Control
    pub const PO_CR: u16 = 0x1B;

    /// Global Control
    pub const GLOB_CNT: u16 = 0x2C;
    /// Global Status
    pub const GLOB_STA: u16 = 0x30;

    // Control Register bits
    pub const CR_RPBM: u8 = 1 << 0; // Run/Pause Bus Master
    pub const CR_RR: u8 = 1 << 1; // Reset Registers
    pub const CR_LVBIE: u8 = 1 << 2; // Last Valid Buffer Interrupt Enable
    pub const CR_FEIE: u8 = 1 << 3; // FIFO Error Interrupt Enable
    pub const CR_IOCE: u8 = 1 << 4; // Interrupt on Completion Enable
}

// =============================================================================
// DRIVER RDS
// =============================================================================

/// Driver AC'97 para o RDS.
pub struct Ac97Driver;

impl Driver for Ac97Driver {
    fn name(&self) -> &'static str {
        "ac97"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Audio
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // AC'97 é class 0x04, subclass 0x01
        if dev.class_code != PCI_CLASS_MULTIMEDIA || dev.subclass_code != PCI_SUBCLASS_AUDIO {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(AC97) Controlador legado encontrado: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // Cria instância do dispositivo
        let device = Ac97Device::new(dev.vendor_id, dev.device_id);

        // Registra no subsistema de som
        crate::drivers::sound::register_device(Arc::new(device));

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(AC97) Driver removido");
        crate::drivers::sound::unregister_device("AC97");
        Ok(())
    }

    fn shutdown(&self, _dev: &mut Device) {
        crate::kinfo!("(AC97) Shutdown");
    }
}

// =============================================================================
// DISPOSITIVO AC'97
// =============================================================================

/// Estado interno do dispositivo AC'97.
struct Ac97State {
    /// Dispositivo habilitado?
    enabled: bool,
    /// Volume mestre (0-100).
    volume: u8,
    /// Em mute?
    muted: bool,
    /// Estado do stream de playback.
    playback_state: StreamState,
    /// Estado do stream de capture.
    capture_state: StreamState,
    /// Estatísticas.
    stats: SoundStats,
}

impl Default for Ac97State {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 80, // AC'97 default é um pouco mais baixo
            muted: false,
            playback_state: StreamState::Stopped,
            capture_state: StreamState::Stopped,
            stats: SoundStats::default(),
        }
    }
}

// TODO: Revisar no futuro
#[allow(unused)]
/// Dispositivo AC'97.
pub struct Ac97Device {
    /// Vendor ID PCI.
    vendor_id: u16,
    /// Device ID PCI.
    device_id: u16,
    /// Estado interno protegido.
    state: Spinlock<Ac97State>,
}

impl Ac97Device {
    /// Cria nova instância do dispositivo.
    pub fn new(vendor_id: u16, device_id: u16) -> Self {
        Self {
            vendor_id,
            device_id,
            state: Spinlock::new(Ac97State::default()),
        }
    }

    /// Verifica se é Intel ICH AC'97.
    fn is_intel_ich(&self) -> bool {
        self.vendor_id == VENDOR_INTEL
    }
}

impl SoundDevice for Ac97Device {
    fn name(&self) -> &'static str {
        "AC97"
    }

    fn card_name(&self) -> &'static str {
        "hw:1"
    }

    fn description(&self) -> &'static str {
        if self.is_intel_ich() {
            "Intel ICH AC'97 Audio Controller"
        } else {
            "AC'97 Audio Controller"
        }
    }

    fn capabilities(&self) -> SoundCapabilities {
        SoundCapabilities::ac97_default()
    }

    fn prepare(&self, direction: StreamDirection, config: StreamConfig) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        // AC'97 é mais restrito: tipicamente 48kHz e S16LE apenas
        if config.format != SampleFormat::S16LE {
            return Err(SoundError::UnsupportedFormat);
        }

        if config.sample_rate != 48000 && config.sample_rate != 44100 {
            crate::kwarn!(
                "(AC97) Sample rate {} pode não ser suportado, usando 48000",
                config.sample_rate
            );
        }

        match direction {
            StreamDirection::Playback => {
                state.playback_state = StreamState::Stopped;
                crate::kinfo!("(AC97) Preparando playback");
            }
            StreamDirection::Capture => {
                state.capture_state = StreamState::Stopped;
                crate::kinfo!("(AC97) Preparando capture");
            }
        }

        Ok(())
    }

    fn start(&self, direction: StreamDirection) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        match direction {
            StreamDirection::Playback => {
                state.playback_state = StreamState::Running;
                crate::kinfo!("(AC97) Playback iniciado");
            }
            StreamDirection::Capture => {
                state.capture_state = StreamState::Running;
                crate::kinfo!("(AC97) Capture iniciado");
            }
        }

        Ok(())
    }

    fn stop(&self, direction: StreamDirection) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        match direction {
            StreamDirection::Playback => {
                state.playback_state = StreamState::Stopped;
                crate::kinfo!("(AC97) Playback parado");
            }
            StreamDirection::Capture => {
                state.capture_state = StreamState::Stopped;
                crate::kinfo!("(AC97) Capture parado");
            }
        }

        Ok(())
    }

    fn stream_state(&self, direction: StreamDirection) -> StreamState {
        let state = self.state.lock();
        match direction {
            StreamDirection::Playback => state.playback_state,
            StreamDirection::Capture => state.capture_state,
        }
    }

    fn write(&self, data: &[u8]) -> Result<usize, SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        if state.playback_state != StreamState::Running {
            return Err(SoundError::NotInitialized);
        }

        // TODO: Escrever no DMA buffer
        let frames = data.len() / 4; // S16LE stereo

        state.stats.frames_played += frames as u64;
        state.stats.bytes_played += data.len() as u64;

        Ok(frames)
    }

    fn set_master_volume(&self, volume: u8) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        state.volume = volume.min(100);

        // AC'97 usa atenuação 0-63 (0 = 0dB, 63 = mute)
        // Convertendo: 100% -> 0, 0% -> 63
        let _attenuation = ((100 - state.volume) as u16 * 63) / 100;

        crate::kinfo!("(AC97) Volume: {}%", state.volume);

        // TODO: Escrever no registro MASTER_VOL

        Ok(())
    }

    fn get_master_volume(&self) -> u8 {
        self.state.lock().volume
    }

    fn set_mute(&self, muted: bool) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        state.muted = muted;

        // AC'97: bit 15 = mute em registros de volume
        crate::kinfo!("(AC97) Mute: {}", if muted { "ON" } else { "OFF" });

        Ok(())
    }

    fn is_muted(&self) -> bool {
        self.state.lock().muted
    }

    fn enable(&self) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if state.enabled {
            return Ok(());
        }

        crate::kinfo!("(AC97) Habilitando codec...");

        // TODO: Inicialização real:
        // 1. Cold reset via GLOB_CNT
        // 2. Aguardar codec ready
        // 3. Configurar sample rate
        // 4. Configurar mixer

        state.enabled = true;
        crate::kinfo!("(AC97) Codec habilitado");

        Ok(())
    }

    fn disable(&self) {
        let mut state = self.state.lock();

        if !state.enabled {
            return;
        }

        crate::kinfo!("(AC97) Desabilitando codec...");

        state.playback_state = StreamState::Stopped;
        state.capture_state = StreamState::Stopped;
        state.enabled = false;
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

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Registra o driver AC'97 no DriverManager.
pub fn init() {
    // crate::kinfo!("(AC97) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(Ac97Driver));
}
