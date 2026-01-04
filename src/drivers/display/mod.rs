//! # Display Drivers Layer
//!
//! Este módulo contém os drivers de display do RedstoneOS. Segue a
//! arquitetura do Redstone Driver System (RDS) para saída visual.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │            Window System                │  (Compositing)
//! ├─────────────────────────────────────────┤
//! │           Display Subsystem             │  (este módulo)
//! ├──────────┬────────┬─────────┬───────────┤
//! │   Bochs  │ VirtIO │   GPU   │    FB     │
//! ├──────────┴────────┴─────────┴───────────┤
//! │              drivers/base               │  (RDS Core)
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Drivers Implementados:
//!
//! | Driver   | Status | Descrição                     |
//! |----------|--------|-------------------------------|
//! | `fb`     | Func   | Framebuffer genérico (GOP)    |
//! | `bochs`  | Func   | Bochs/QEMU BGA                |
//! | `virtio` | Stub   | VirtIO-GPU                    |
//! | `gpu`    | Stub   | Intel/AMD/NVIDIA              |
//! | `edid`   | Func   | Parser de informações monitor |
//!
//! ## Hierarquia de Fallback:
//! 1. GPU nativa (se disponível)
//! 2. VirtIO-GPU (se em VM)
//! 3. Bochs BGA (se em QEMU/Bochs)
//! 4. Framebuffer GOP/VBE (sempre disponível)

pub mod bochs;
pub mod edid;
pub mod fb;
pub mod gpu;
pub mod traits;
pub mod virtio;

// Re-exports
pub use fb::{BUFFER_MANAGER, DISPLAY_CRTC};
pub use traits::*;

use crate::core::boot::handoff::FramebufferInfo as HandoffFbInfo;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de dispositivos de display registrados.
static DISPLAY_DEVICES: Spinlock<Vec<DisplayDeviceRef>> = Spinlock::new(Vec::new());

/// Display primário.
static PRIMARY_DISPLAY: Spinlock<Option<usize>> = Spinlock::new(None);

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema de display.
pub fn init(info: HandoffFbInfo) {
    crate::kinfo!("(Display) Inicializando subsistema gráfico...");

    // Fase 1: Framebuffer legado (fallback garantido)
    fb::init(info);

    // Fase 2: Drivers virtuais (para VMs)
    bochs::init();
    virtio::init();

    // Fase 3: Drivers de GPU real
    gpu::init();

    *INITIALIZED.lock() = true;

    let count = device_count();
    crate::kinfo!("(Display) Subsistema inicializado: {} display(s)", count);
}

/// Desliga o subsistema.
pub fn shutdown() {
    crate::kinfo!("(Display) Shutdown do subsistema...");

    let devices = DISPLAY_DEVICES.lock();
    for dev in devices.iter() {
        dev.disable();
    }
}

// =============================================================================
// REGISTRO DE DISPOSITIVOS
// =============================================================================

/// Registra um novo dispositivo de display.
pub fn register_device(dev: DisplayDeviceRef) {
    let name = dev.name();
    let mode = dev.current_mode();
    crate::kinfo!(
        "(Display) Registrando: {} ({}x{})",
        name,
        mode.width,
        mode.height
    );

    let mut devices = DISPLAY_DEVICES.lock();
    let index = devices.len();
    devices.push(dev);

    // Primeiro display se torna primário
    let mut primary = PRIMARY_DISPLAY.lock();
    if primary.is_none() {
        *primary = Some(index);
        crate::kinfo!("(Display) Display primário: {}", name);
    }
}

/// Remove um dispositivo de display.
pub fn unregister_device(name: &str) {
    crate::kinfo!("(Display) Removendo: {}", name);
    DISPLAY_DEVICES.lock().retain(|d| d.name() != name);
}

// =============================================================================
// CONSULTA DE DISPOSITIVOS
// =============================================================================

/// Retorna número de displays.
pub fn device_count() -> usize {
    DISPLAY_DEVICES.lock().len()
}

/// Verifica se está inicializado.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock()
}

/// Busca display por nome.
pub fn find_device(name: &str) -> Option<DisplayDeviceRef> {
    DISPLAY_DEVICES
        .lock()
        .iter()
        .find(|d| d.name() == name)
        .cloned()
}

/// Retorna display por índice.
pub fn get_device(index: usize) -> Option<DisplayDeviceRef> {
    DISPLAY_DEVICES.lock().get(index).cloned()
}

/// Retorna display primário.
pub fn get_primary() -> Option<DisplayDeviceRef> {
    let primary = PRIMARY_DISPLAY.lock();
    primary.and_then(|idx| DISPLAY_DEVICES.lock().get(idx).cloned())
}

/// Define display primário.
pub fn set_primary(name: &str) -> bool {
    let devices = DISPLAY_DEVICES.lock();
    if let Some(idx) = devices.iter().position(|d| d.name() == name) {
        *PRIMARY_DISPLAY.lock() = Some(idx);
        true
    } else {
        false
    }
}

// =============================================================================
// FUNÇÕES DE CONVENIÊNCIA
// =============================================================================

/// Retorna resolução do display primário.
pub fn resolution() -> (u32, u32) {
    if let Some(dev) = get_primary() {
        let mode = dev.current_mode();
        (mode.width, mode.height)
    } else {
        (0, 0)
    }
}

/// Retorna ponteiro do framebuffer primário.
pub fn framebuffer() -> Option<*mut u8> {
    get_primary().map(|dev| dev.framebuffer())
}

/// Limpa display primário.
pub fn clear(color: u32) {
    if let Some(dev) = get_primary() {
        dev.clear(color);
    }
}
