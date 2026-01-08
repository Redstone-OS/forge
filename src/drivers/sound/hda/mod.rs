//! # Intel High Definition Audio Driver
//!
//! Driver para controladores de áudio Intel HDA (High Definition Audio).
//! Padrão moderno presente na maioria dos PCs desde ~2004.
//!
//! ## Especificação:
//! Intel High Definition Audio Specification, Revision 1.0a
//!
//! ## Arquitetura HDA:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           HDA Controller                │
//! │  ┌─────────────────────────────────┐    │
//! │  │  CORB (Command Output Ring Buf) │───►│ Comandos para Codecs
//! │  │  RIRB (Response Input Ring Buf) │◄───│ Respostas dos Codecs
//! │  │  Stream Descriptors (DMA)       │◄──►│ Dados PCM
//! │  └─────────────────────────────────┘    │
//! │                  │                      │
//! │            HDA Link                     │
//! │                  │                      │
//! │    ┌─────────┬───┴───┬─────────┐        │
//! │    ▼         ▼       ▼         ▼        │
//! │  Codec 0   Codec 1  Codec 2  Codec 3    │
//! │  (Audio)   (Modem)  ...      ...        │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Status:
//! - [x] Estrutura básica do driver (RDS)
//! - [x] Detecção PCI (class 0x04, subclass 0x03)
//! - [ ] Inicialização do controlador
//! - [ ] Enumeração de codecs
//! - [ ] Configuração de streams
//! - [ ] DMA ring buffers

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::sound::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

// =============================================================================
// CONSTANTES HDA
// =============================================================================

/// PCI Class: Multimedia
const PCI_CLASS_MULTIMEDIA: u8 = 0x04;
/// PCI Subclass: HD Audio
const PCI_SUBCLASS_HDA: u8 = 0x03;

/// Vendor IDs conhecidos com HDA
const VENDOR_INTEL: u16 = 0x8086;
const VENDOR_AMD: u16 = 0x1002;
const VENDOR_NVIDIA: u16 = 0x10DE;
const VENDOR_REALTEK: u16 = 0x10EC;

// Registros do controlador HDA (offsets do BAR0)
#[allow(dead_code)]
mod regs {
    /// Global Capabilities
    pub const GCAP: u16 = 0x00;
    /// Minor Version
    pub const VMIN: u16 = 0x02;
    /// Major Version
    pub const VMAJ: u16 = 0x03;
    /// Output Payload Capability
    pub const OUTPAY: u16 = 0x04;
    /// Input Payload Capability
    pub const INPAY: u16 = 0x06;
    /// Global Control
    pub const GCTL: u16 = 0x08;
    /// Wake Enable
    pub const WAKEEN: u16 = 0x0C;
    /// State Change Status
    pub const STATESTS: u16 = 0x0E;
    /// Global Status
    pub const GSTS: u16 = 0x10;
    /// Interrupt Control
    pub const INTCTL: u16 = 0x20;
    /// Interrupt Status
    pub const INTSTS: u16 = 0x24;
    /// CORB Lower Base Address
    pub const CORBLBASE: u16 = 0x40;
    /// CORB Upper Base Address
    pub const CORBUBASE: u16 = 0x44;
    /// CORB Write Pointer
    pub const CORBWP: u16 = 0x48;
    /// CORB Read Pointer
    pub const CORBRP: u16 = 0x4A;
    /// CORB Control
    pub const CORBCTL: u16 = 0x4C;
    /// CORB Status
    pub const CORBSTS: u16 = 0x4D;
    /// CORB Size
    pub const CORBSIZE: u16 = 0x4E;
    /// RIRB Lower Base Address
    pub const RIRBLBASE: u16 = 0x50;
    /// RIRB Upper Base Address
    pub const RIRBUBASE: u16 = 0x54;
    /// RIRB Write Pointer
    pub const RIRBWP: u16 = 0x58;
    /// RIRB Interrupt Count
    pub const RINTCNT: u16 = 0x5A;
    /// RIRB Control
    pub const RIRBCTL: u16 = 0x5C;
    /// RIRB Status
    pub const RIRBSTS: u16 = 0x5D;
    /// RIRB Size
    pub const RIRBSIZE: u16 = 0x5E;

    // GCTL bits
    pub const GCTL_RESET: u32 = 1 << 0;
    pub const GCTL_FCNTRL: u32 = 1 << 1;
    pub const GCTL_UNSOL: u32 = 1 << 8;
}

// =============================================================================
// DRIVER RDS
// =============================================================================

/// Driver Intel HDA para o RDS.
pub struct IntelHdaDriver;

impl Driver for IntelHdaDriver {
    fn name(&self) -> &'static str {
        "intel-hda"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Audio
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // Verifica class/subclass PCI
        if dev.class_code != PCI_CLASS_MULTIMEDIA || dev.subclass_code != PCI_SUBCLASS_HDA {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(Intel HDA) Controlador encontrado: {:04X}:{:04X}",
            dev.vendor_id,
            dev.device_id
        );

        // Cria instância do dispositivo
        let device = IntelHdaDevice::new(dev.vendor_id, dev.device_id);

        // Registra no subsistema de som
        crate::drivers::sound::register_device(Arc::new(device));

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(Intel HDA) Driver removido");
        crate::drivers::sound::unregister_device("Intel HDA");
        Ok(())
    }

    fn shutdown(&self, _dev: &mut Device) {
        crate::kinfo!("(Intel HDA) Shutdown");
    }
}

// =============================================================================
// DISPOSITIVO HDA
// =============================================================================

/// Estado interno do dispositivo HDA.
struct HdaState {
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
    /// Configuração atual de playback.
    playback_config: Option<StreamConfig>,
    /// Estatísticas.
    stats: SoundStats,
}

impl Default for HdaState {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 100,
            muted: false,
            playback_state: StreamState::Stopped,
            capture_state: StreamState::Stopped,
            playback_config: None,
            stats: SoundStats::default(),
        }
    }
}

// TODO: Revisar no futuro
#[allow(unused)]
/// Dispositivo Intel HDA.
pub struct IntelHdaDevice {
    /// Vendor ID PCI.
    vendor_id: u16,
    /// Device ID PCI.
    device_id: u16,
    /// Estado interno protegido.
    state: Spinlock<HdaState>,
}

impl IntelHdaDevice {
    /// Cria nova instância do dispositivo.
    pub fn new(vendor_id: u16, device_id: u16) -> Self {
        Self {
            vendor_id,
            device_id,
            state: Spinlock::new(HdaState::default()),
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna nome do fabricante.
    fn vendor_name(&self) -> &'static str {
        match self.vendor_id {
            VENDOR_INTEL => "Intel",
            VENDOR_AMD => "AMD",
            VENDOR_NVIDIA => "NVIDIA",
            VENDOR_REALTEK => "Realtek",
            _ => "Generic",
        }
    }
}

impl SoundDevice for IntelHdaDevice {
    fn name(&self) -> &'static str {
        "Intel HDA"
    }

    fn card_name(&self) -> &'static str {
        "hw:0"
    }

    fn description(&self) -> &'static str {
        match self.vendor_id {
            VENDOR_INTEL => "Intel High Definition Audio Controller",
            VENDOR_AMD => "AMD High Definition Audio Controller",
            VENDOR_NVIDIA => "NVIDIA High Definition Audio Controller",
            VENDOR_REALTEK => "Realtek High Definition Audio Controller",
            _ => "Generic High Definition Audio Controller",
        }
    }

    fn capabilities(&self) -> SoundCapabilities {
        SoundCapabilities::hda_default()
    }

    fn prepare(&self, direction: StreamDirection, config: StreamConfig) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        if !self.capabilities().supports(&config) {
            return Err(SoundError::InvalidConfig);
        }

        match direction {
            StreamDirection::Playback => {
                state.playback_config = Some(config);
                state.playback_state = StreamState::Stopped;
                crate::kinfo!(
                    "(Intel HDA) Preparando playback: {}Hz, {} canais",
                    config.sample_rate,
                    config.channels
                );
            }
            StreamDirection::Capture => {
                state.capture_state = StreamState::Stopped;
                crate::kinfo!("(Intel HDA) Preparando capture");
            }
        }

        // TODO: Configurar DMA buffers e stream descriptors

        Ok(())
    }

    fn start(&self, direction: StreamDirection) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        match direction {
            StreamDirection::Playback => {
                if state.playback_config.is_none() {
                    return Err(SoundError::InvalidConfig);
                }
                state.playback_state = StreamState::Running;
                crate::kinfo!("(Intel HDA) Playback iniciado");
            }
            StreamDirection::Capture => {
                state.capture_state = StreamState::Running;
                crate::kinfo!("(Intel HDA) Capture iniciado");
            }
        }

        Ok(())
    }

    fn stop(&self, direction: StreamDirection) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        match direction {
            StreamDirection::Playback => {
                state.playback_state = StreamState::Stopped;
                crate::kinfo!("(Intel HDA) Playback parado");
            }
            StreamDirection::Capture => {
                state.capture_state = StreamState::Stopped;
                crate::kinfo!("(Intel HDA) Capture parado");
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

        // TODO: Escrever no DMA buffer real
        // Por enquanto, simula escrita bem-sucedida

        let config = state
            .playback_config
            .as_ref()
            .ok_or(SoundError::InvalidConfig)?;
        let bytes_per_frame = config.bytes_per_frame();
        let frames = data.len() / bytes_per_frame;

        state.stats.frames_played += frames as u64;
        state.stats.bytes_played += data.len() as u64;

        Ok(frames)
    }

    fn read(&self, buffer: &mut [u8]) -> Result<usize, SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        if state.capture_state != StreamState::Running {
            return Err(SoundError::NotInitialized);
        }

        // TODO: Ler do DMA buffer real
        // Por enquanto, retorna silêncio

        buffer.fill(0);
        let frames = buffer.len() / 4; // Assume S16LE stereo

        state.stats.frames_captured += frames as u64;
        state.stats.bytes_captured += buffer.len() as u64;

        Ok(frames)
    }

    fn set_master_volume(&self, volume: u8) -> Result<(), SoundError> {
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(SoundError::NotEnabled);
        }

        state.volume = volume.min(100);
        crate::kinfo!("(Intel HDA) Volume: {}%", state.volume);

        // TODO: Configurar volume via codec verbs

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
        crate::kinfo!("(Intel HDA) Mute: {}", if muted { "ON" } else { "OFF" });

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

        crate::kinfo!("(Intel HDA) Habilitando controlador...");

        // TODO: Inicialização real do hardware:
        // 1. Reset do controlador (GCTL.CRST)
        // 2. Aguardar link ativo
        // 3. Configurar CORB/RIRB
        // 4. Enumerar codecs
        // 5. Configurar widgets

        state.enabled = true;
        crate::kinfo!("(Intel HDA) Controlador habilitado");

        Ok(())
    }

    fn disable(&self) {
        let mut state = self.state.lock();

        if !state.enabled {
            return;
        }

        crate::kinfo!("(Intel HDA) Desabilitando controlador...");

        // Para streams ativos
        state.playback_state = StreamState::Stopped;
        state.capture_state = StreamState::Stopped;

        // TODO: Reset do hardware

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

/// Registra o driver Intel HDA no DriverManager.
pub fn init() {
    // crate::kinfo!("(Intel HDA) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(IntelHdaDriver));
}
