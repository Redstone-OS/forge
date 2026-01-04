//! # VirtIO Input Driver
//!
//! Driver para **VirtIO Input** - dispositivos de entrada paravirtualizados
//! usados por QEMU, KVM e outros hypervisors.
//!
//! ## Tipos de Dispositivos VirtIO Input:
//! - **Keyboard**: Teclado virtual
//! - **Mouse**: Mouse com movimento relativo
//! - **Tablet**: Tablet com posição absoluta
//!
//! ## Vantagens:
//! - Baixa latência (sem emulação)
//! - Eventos precisos
//! - Suporte a multi-touch
//!
//! ## STUB:
//! Estrutura definida. Implementação pendente após VirtIO bus.

use super::traits::*;
use crate::drivers::bus::virtio::{VirtioDevice, VirtioDeviceType};
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Tipo de evento VirtIO Input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VirtioInputEventType {
    Syn = 0x00,
    Key = 0x01,
    Rel = 0x02,
    Abs = 0x03,
}

// Códigos EV_REL
pub const REL_X: u16 = 0x00;
pub const REL_Y: u16 = 0x01;
pub const REL_Z: u16 = 0x02;
pub const REL_WHEEL: u16 = 0x08;

// Códigos EV_ABS
pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;
pub const ABS_Z: u16 = 0x02;
pub const ABS_MT_SLOT: u16 = 0x2F;
pub const ABS_MT_POSITION_X: u16 = 0x35;
pub const ABS_MT_POSITION_Y: u16 = 0x36;

// =============================================================================
// ESTRUTURAS
// =============================================================================

/// Evento VirtIO Input.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtioInputEvent {
    pub event_type: u16,
    pub code: u16,
    pub value: u32,
}

/// Configuração de eixo absoluto.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtioInputAbsInfo {
    pub min: u32,
    pub max: u32,
    pub fuzz: u32,
    pub flat: u32,
    pub res: u32,
}

// =============================================================================
// DRIVER
// =============================================================================

/// Driver VirtIO Input.
pub struct VirtioInputDevice {
    /// Nome do dispositivo.
    name: &'static str,
    /// Tipo (keyboard, mouse, tablet).
    input_type: VirtioInputType,
    /// Habilitado?
    enabled: Spinlock<bool>,
}

/// Tipo de dispositivo VirtIO Input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioInputType {
    Keyboard,
    Mouse,
    Tablet,
    Unknown,
}

impl VirtioInputDevice {
    /// Cria novo dispositivo.
    pub fn new(name: &'static str, input_type: VirtioInputType) -> Self {
        Self {
            name,
            input_type,
            enabled: Spinlock::new(false),
        }
    }

    /// Probe de dispositivo VirtIO.
    ///
    /// ## STUB:
    /// Não implementado.
    pub fn probe(_virtio_dev: &VirtioDevice) -> Option<Self> {
        crate::kwarn!("(VirtIO Input) probe() não implementado");
        None
    }

    /// Processa evento VirtIO.
    pub fn process_event(&self, event: VirtioInputEvent) -> Option<InputEvent> {
        match event.event_type {
            t if t == VirtioInputEventType::Key as u16 => Some(InputEvent::Key(KeyEvent {
                scancode: event.code,
                keycode: KeyCode::Unknown,
                state: if event.value != 0 {
                    KeyState::Pressed
                } else {
                    KeyState::Released
                },
                modifiers: Modifiers::empty(),
                timestamp: 0,
            })),
            t if t == VirtioInputEventType::Rel as u16 => {
                // Movimento relativo de mouse
                // TODO: Acumular e gerar PointerEvent
                None
            }
            t if t == VirtioInputEventType::Abs as u16 => {
                // Posição absoluta (tablet)
                // TODO: Gerar PointerEvent com posição absoluta
                None
            }
            _ => None,
        }
    }
}

impl InputDevice for VirtioInputDevice {
    fn name(&self) -> &str {
        self.name
    }

    fn device_type(&self) -> InputDeviceType {
        match self.input_type {
            VirtioInputType::Keyboard => InputDeviceType::Keyboard,
            VirtioInputType::Mouse => InputDeviceType::Mouse,
            VirtioInputType::Tablet => InputDeviceType::Tablet,
            VirtioInputType::Unknown => InputDeviceType::Other,
        }
    }

    fn is_connected(&self) -> bool {
        *self.enabled.lock()
    }

    fn enable(&self) -> bool {
        *self.enabled.lock() = true;
        true
    }

    fn disable(&self) {
        *self.enabled.lock() = false;
    }

    fn poll(&self) -> Option<InputEvent> {
        None // Usa IRQ
    }
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static DEVICES: Spinlock<Vec<Arc<VirtioInputDevice>>> = Spinlock::new(Vec::new());

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa driver VirtIO Input.
pub fn init() {
    crate::kinfo!("(VirtIO Input) Inicializando driver...");

    // Busca dispositivos VirtIO Input
    if let Some(_vdev) = crate::drivers::bus::virtio::find_by_type(VirtioDeviceType::Input) {
        crate::kinfo!("(VirtIO Input) Dispositivo encontrado");
        crate::kwarn!("(VirtIO Input) Inicialização não implementada");
    }

    crate::kinfo!("(VirtIO Input) Driver inicializado (stub)");
}

/// Desliga driver.
pub fn shutdown() {
    crate::kinfo!("(VirtIO Input) Shutdown");
}

/// Retorna número de dispositivos.
pub fn device_count() -> usize {
    DEVICES.lock().len()
}
