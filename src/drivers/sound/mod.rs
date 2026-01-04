//! # Sound Drivers Layer
//!
//! Este módulo contém os drivers de som do RedstoneOS. Segue a arquitetura
//! do Redstone Driver System (RDS) para integração com o subsistema de áudio.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Applications               │
//! ├─────────────────────────────────────────┤
//! │             Audio Services              │  (Userspace)
//! ├─────────────────────────────────────────┤
//! │            Sound Subsystem              │  (este módulo)
//! ├──────────┬──────────┬──────────┬────────┤
//! │    HDA   │   AC97   │  VirtIO  │ Mixer  │
//! ├──────────┴──────────┴──────────┴────────┤
//! │             drivers/base                │  (RDS Core)
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Drivers Implementados:
//!
//! | Driver   | Status | Descrição                    |
//! |----------|--------|------------------------------|
//! | `hda`    | Stub   | Intel High Definition Audio  |
//! | `ac97`   | Stub   | Audio Codec '97 (Legado)     |
//! | `virtio` | Stub   | VirtIO-Sound (QEMU/KVM)      |
//! | `mixer`  | Stub   | Software mixer               |
//!
//! ## Integração RDS:
//! - Drivers implementam `SoundDevice` + `drivers::base::Driver`
//! - Registro via `sound::register_device()`
//! - Recuperação automática via `RecoveryManager`

pub mod ac97;
pub mod hda;
pub mod mixer;
pub mod traits;
pub mod virtio;

// Re-exports
pub use traits::*;

use crate::drivers::base::device::Device;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL DO SUBSISTEMA
// =============================================================================

/// Lista de dispositivos de som registrados.
static SOUND_DEVICES: Spinlock<Vec<SoundDeviceRef>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

/// Dispositivo padrão selecionado.
static DEFAULT_DEVICE: Spinlock<Option<usize>> = Spinlock::new(None);

// =============================================================================
// INICIALIZAÇÃO DO SUBSISTEMA
// =============================================================================

/// Inicializa o subsistema de som.
///
/// Registra todos os drivers de som disponíveis no DriverManager.
/// A ordem de inicialização define prioridade de fallback.
pub fn init() {
    crate::kinfo!("(Sound) Inicializando subsistema de som...");

    // Fase 1: Drivers Modernos (prioridade)
    hda::init();

    // Fase 2: VirtIO (para VMs)
    virtio::init();

    // Fase 3: Legado (fallback)
    ac97::init();

    // Fase 4: Mixer de Software
    mixer::init();

    *INITIALIZED.lock() = true;

    let count = device_count();
    crate::kinfo!("(Sound) Subsistema inicializado: {} dispositivo(s)", count);
}

/// Escaneia por dispositivos de som via Bus.
///
/// Chamado pelo BusManager durante descoberta de hardware.
/// Retorna dispositivos encontrados para probing.
pub fn scan() -> Vec<Device> {
    crate::kinfo!("(Sound) Escaneando dispositivos de som...");

    // Dispositivos de som são descobertos via PCI scan
    // Class 0x04 (Multimedia), Subclass 0x01 (Audio) ou 0x03 (HDA)
    // O BusManager já faz isso, então retornamos vazio aqui

    Vec::new()
}

/// Desliga o subsistema de som.
///
/// Para todos os streams ativos e desabilita dispositivos.
pub fn shutdown() {
    crate::kinfo!("(Sound) Shutdown do subsistema de som...");

    let devices = SOUND_DEVICES.lock();
    for dev in devices.iter() {
        // Para streams ativos
        let _ = dev.stop(StreamDirection::Playback);
        let _ = dev.stop(StreamDirection::Capture);

        // Desabilita dispositivo
        dev.disable();
    }

    crate::kinfo!("(Sound) Subsistema desligado");
}

// =============================================================================
// REGISTRO DE DISPOSITIVOS
// =============================================================================

/// Registra um novo dispositivo de som.
///
/// Chamado pelos drivers durante `probe()`.
pub fn register_device(dev: SoundDeviceRef) {
    let name = dev.name();
    let desc = dev.description();
    crate::kinfo!("(Sound) Registrando: {} ({})", name, desc);

    let mut devices = SOUND_DEVICES.lock();
    let index = devices.len();
    devices.push(dev);

    // Se é o primeiro dispositivo, define como padrão
    let mut default = DEFAULT_DEVICE.lock();
    if default.is_none() {
        *default = Some(index);
        crate::kinfo!("(Sound) Dispositivo padrão definido: {}", name);
    }
}

/// Remove um dispositivo de som.
///
/// Chamado durante `remove()` do driver.
pub fn unregister_device(name: &str) {
    crate::kinfo!("(Sound) Removendo dispositivo: {}", name);

    let mut devices = SOUND_DEVICES.lock();
    let prev_len = devices.len();
    devices.retain(|d| d.name() != name);

    if devices.len() < prev_len {
        // Atualiza dispositivo padrão se necessário
        let mut default = DEFAULT_DEVICE.lock();
        if devices.is_empty() {
            *default = None;
        } else if default.map_or(false, |i| i >= devices.len()) {
            *default = Some(0);
        }
    }
}

// =============================================================================
// CONSULTA DE DISPOSITIVOS
// =============================================================================

/// Retorna número de dispositivos de som registrados.
pub fn device_count() -> usize {
    SOUND_DEVICES.lock().len()
}

/// Verifica se o subsistema está inicializado.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock()
}

/// Busca dispositivo por nome.
pub fn find_device(name: &str) -> Option<SoundDeviceRef> {
    SOUND_DEVICES
        .lock()
        .iter()
        .find(|d| d.name() == name)
        .cloned()
}

/// Busca dispositivo por índice.
pub fn get_device(index: usize) -> Option<SoundDeviceRef> {
    SOUND_DEVICES.lock().get(index).cloned()
}

/// Retorna lista de todos os dispositivos.
pub fn get_all_devices() -> Vec<SoundDeviceRef> {
    SOUND_DEVICES.lock().clone()
}

/// Retorna o dispositivo padrão.
pub fn get_default_device() -> Option<SoundDeviceRef> {
    let default = DEFAULT_DEVICE.lock();
    default.and_then(|idx| SOUND_DEVICES.lock().get(idx).cloned())
}

/// Define o dispositivo padrão.
pub fn set_default_device(name: &str) -> bool {
    let devices = SOUND_DEVICES.lock();
    if let Some(idx) = devices.iter().position(|d| d.name() == name) {
        *DEFAULT_DEVICE.lock() = Some(idx);
        crate::kinfo!("(Sound) Novo dispositivo padrão: {}", name);
        true
    } else {
        false
    }
}

/// Retorna lista de nomes de dispositivos.
pub fn list_device_names() -> Vec<&'static str> {
    // Precisamos retornar strings estáticas, então copiamos os nomes
    // Na prática, os drivers retornam &'static str
    SOUND_DEVICES
        .lock()
        .iter()
        .map(|d| {
            // Leak do nome para obter 'static lifetime
            // Isso é seguro porque drivers nunca são removidos em runtime normal
            let name: &str = d.name();
            // Retornamos diretamente, já que name() retorna &str do driver
            name
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|s| {
            // Na prática os nomes são literais, mas precisamos do tipo correto
            s as &str
        })
        .collect::<Vec<_>>()
        .leak()
        .to_vec()
}

// =============================================================================
// ESTATÍSTICAS GLOBAIS
// =============================================================================

/// Estatísticas agregadas do subsistema de som.
#[derive(Debug, Clone, Default)]
pub struct GlobalSoundStats {
    /// Número de dispositivos ativos.
    pub active_devices: usize,
    /// Total de frames reproduzidos.
    pub total_frames_played: u64,
    /// Total de frames capturados.
    pub total_frames_captured: u64,
    /// Total de underruns.
    pub total_underruns: u32,
    /// Total de overruns.
    pub total_overruns: u32,
}

/// Retorna estatísticas agregadas de todos os dispositivos.
pub fn get_global_stats() -> GlobalSoundStats {
    let devices = SOUND_DEVICES.lock();
    let mut global = GlobalSoundStats {
        active_devices: devices.iter().filter(|d| d.is_enabled()).count(),
        ..Default::default()
    };

    for dev in devices.iter() {
        let stats = dev.get_stats();
        global.total_frames_played += stats.frames_played;
        global.total_frames_captured += stats.frames_captured;
        global.total_underruns += stats.underruns;
        global.total_overruns += stats.overruns;
    }

    global
}

// =============================================================================
// FUNÇÕES DE CONVENIÊNCIA
// =============================================================================

/// Reproduz dados de áudio no dispositivo padrão.
///
/// Função de alto nível para reprodução simples.
pub fn play(data: &[u8], config: StreamConfig) -> Result<usize, SoundError> {
    let device = get_default_device().ok_or(SoundError::NotInitialized)?;

    if !device.is_enabled() {
        device.enable()?;
    }

    device.prepare(StreamDirection::Playback, config)?;
    device.start(StreamDirection::Playback)?;

    device.write(data)
}

/// Define volume mestre do dispositivo padrão.
pub fn set_volume(volume: u8) -> Result<(), SoundError> {
    let device = get_default_device().ok_or(SoundError::NotInitialized)?;
    device.set_master_volume(volume.min(100))
}

/// Obtém volume mestre do dispositivo padrão.
pub fn get_volume() -> Result<u8, SoundError> {
    let device = get_default_device().ok_or(SoundError::NotInitialized)?;
    Ok(device.get_master_volume())
}

/// Muta/desmuta o dispositivo padrão.
pub fn set_mute(muted: bool) -> Result<(), SoundError> {
    let device = get_default_device().ok_or(SoundError::NotInitialized)?;
    device.set_mute(muted)
}
