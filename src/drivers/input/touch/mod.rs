//! # Touch Input Subsystem
//!
//! Este módulo gerencia dispositivos de toque: **touchpads** e **touchscreens**.
//!
//! ## Tipos de Dispositivos:
//! - **Touchpad**: Via I2C HID (laptops modernos) ou PS/2 Synaptics
//! - **Touchscreen**: Via USB HID ou I2C HID
//!
//! ## Protocolos:
//! - **HID-over-I2C**: Padrão moderno
//! - **Synaptics RMI4**: Proprietário
//! - **ELAN**: Proprietário
//!
//! ## Multi-touch:
//! Suporte para até 10 pontos de contato simultâneos.
//!
//! ## STUB:
//! Estrutura definida. Implementação pendente.

pub mod state; // Estado de touch

use super::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static TOUCH_STATE: Spinlock<TouchState> = Spinlock::new(TouchState {
    contacts: [None; 10],
    count: 0,
});

static DEVICES: Spinlock<Vec<Arc<dyn InputDevice>>> = Spinlock::new(Vec::new());
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa subsistema de touch.
pub fn init() {
    crate::kinfo!("(Touch) Inicializando subsistema de touch...");

    // TODO: Detectar touchpads I2C
    // TODO: Detectar touchscreens

    *INITIALIZED.lock() = true;

    crate::kwarn!("(Touch) Detecção de dispositivos não implementada");
    crate::kinfo!("(Touch) Subsistema inicializado");
}

/// Desliga subsistema de touch.
pub fn shutdown() {
    crate::kinfo!("(Touch) Shutdown");
}

/// Retorna estado atual de touch.
pub fn get_state() -> Option<TouchState> {
    let state = TOUCH_STATE.lock();
    if state.count > 0 {
        Some(state.clone())
    } else {
        None
    }
}

/// Atualiza estado de touch a partir de eventos.
pub fn update_state(event: TouchEvent) {
    let mut state = TOUCH_STATE.lock();

    match event.event_type {
        TouchEventType::Down => {
            // Novo contato
            for i in 0..10 {
                if state.contacts[i].is_none() {
                    state.contacts[i] = Some(TouchContact {
                        id: event.id,
                        x: event.x,
                        y: event.y,
                        pressure: event.pressure,
                    });
                    state.count += 1;
                    break;
                }
            }
        }
        TouchEventType::Move => {
            // Atualiza contato existente
            for contact in state.contacts.iter_mut().flatten() {
                if contact.id == event.id {
                    contact.x = event.x;
                    contact.y = event.y;
                    contact.pressure = event.pressure;
                    break;
                }
            }
        }
        TouchEventType::Up | TouchEventType::Cancel => {
            // Remove contato
            for i in 0..10 {
                if let Some(contact) = &state.contacts[i] {
                    if contact.id == event.id {
                        state.contacts[i] = None;
                        if state.count > 0 {
                            state.count -= 1;
                        }
                        break;
                    }
                }
            }
        }
    }
}

/// Limpa todos os contatos.
pub fn clear_contacts() {
    let mut state = TOUCH_STATE.lock();
    state.contacts = [None; 10];
    state.count = 0;
}

/// Retorna número de dispositivos de touch.
pub fn device_count() -> usize {
    DEVICES.lock().len()
}

/// Verifica se há algum touchpad.
pub fn has_touchpad() -> bool {
    DEVICES
        .lock()
        .iter()
        .any(|d| d.device_type() == InputDeviceType::Touchpad)
}

/// Verifica se há algum touchscreen.
pub fn has_touchscreen() -> bool {
    DEVICES
        .lock()
        .iter()
        .any(|d| d.device_type() == InputDeviceType::Touchscreen)
}
