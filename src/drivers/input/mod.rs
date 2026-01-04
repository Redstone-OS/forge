//! # Input Subsystem
//!
//! Este módulo gerencia todos os **dispositivos de entrada** do RedstoneOS,
//! desde o teclado PS/2 legado até dispositivos USB HID complexos.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                 Aplicações                      │
//! ├─────────────────────────────────────────────────┤
//! │              Event Queue                        │  (eventos unificados)
//! ├─────────────────────────────────────────────────┤
//! │              Input Core                         │  (este módulo)
//! ├─────────────┬─────────────┬─────────────────────┤
//! │     HID     │     PS/2    │       VirtIO        │
//! │   Parser    │  Controller │       Input         │
//! ├─────────────┼─────────────┼─────────────────────┤
//! │  USB/I2C    │   Platform  │        PCI          │
//! └─────────────┴─────────────┴─────────────────────┘
//! ```
//!
//! ## Tipos de Dispositivos:
//!
//! ### Teclados:
//! - **PS/2**: Controlador 8042 legado
//! - **USB HID**: Via USB stack
//! - **VirtIO**: Paravirtualizado
//!
//! ### Ponteiros (Mouse/Touchpad):
//! - **PS/2 Mouse**: Controlador 8042
//! - **USB Mouse**: Via USB HID
//! - **Touchpad I2C**: Via HID-over-I2C
//! - **VirtIO Mouse/Tablet**: Paravirtualizado
//!
//! ### Touch:
//! - **Touchscreen**: Multi-touch via HID
//!
//! ## Fluxo de Eventos:
//! 1. Hardware gera interrupção
//! 2. Driver específico processa dados brutos
//! 3. Driver converte para `InputEvent`
//! 4. Evento é adicionado à fila global
//! 5. Compositor/aplicação consome evento

pub mod hid; // HID Parser (USB/I2C)
pub mod ps2; // PS/2 Controller (8042)
pub mod touch; // Touchpad/Touchscreen
pub mod traits; // Traits e tipos de eventos
pub mod usb; // USB Input devices
pub mod virtio; // VirtIO Input

// Re-exports principais
pub use traits::*;

use crate::sync::Spinlock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Fila global de eventos de entrada.
static EVENT_QUEUE: Spinlock<VecDeque<InputEvent>> = Spinlock::new(VecDeque::new());

/// Lista de dispositivos de entrada registrados.
static INPUT_DEVICES: Spinlock<Vec<Arc<dyn InputDevice>>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

/// Tamanho máximo da fila de eventos.
const MAX_EVENT_QUEUE_SIZE: usize = 256;

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema de entrada.
pub fn init() {
    crate::kinfo!("(Input) Inicializando subsistema de entrada...");

    // 1. Inicializa HID parser (necessário para USB e I2C HID)
    hid::init();

    // 2. PS/2 Controller (sempre disponível como fallback)
    ps2::init();

    // 3. VirtIO Input (para VMs)
    virtio::init();

    // 4. Touchpads e touchscreens
    touch::init();

    // 5. USB Input devices (após USB stack)
    usb::init();

    *INITIALIZED.lock() = true;

    crate::kinfo!("(Input) Subsistema inicializado");
}

/// Desliga o subsistema de entrada.
pub fn shutdown() {
    crate::kinfo!("(Input) Shutdown do subsistema de entrada...");

    usb::shutdown();
    virtio::shutdown();
    touch::shutdown();
    ps2::shutdown();
    hid::shutdown();
}

// =============================================================================
// FILA DE EVENTOS
// =============================================================================

/// Adiciona um evento à fila global.
pub fn push_event(event: InputEvent) {
    let mut queue = EVENT_QUEUE.lock();

    // Descarta eventos antigos se fila estiver cheia
    if queue.len() >= MAX_EVENT_QUEUE_SIZE {
        queue.pop_front();
    }

    queue.push_back(event);
}

/// Remove e retorna o próximo evento da fila.
pub fn pop_event() -> Option<InputEvent> {
    EVENT_QUEUE.lock().pop_front()
}

/// Verifica se há eventos pendentes.
pub fn has_events() -> bool {
    !EVENT_QUEUE.lock().is_empty()
}

/// Retorna número de eventos pendentes.
pub fn event_count() -> usize {
    EVENT_QUEUE.lock().len()
}

/// Limpa a fila de eventos.
pub fn clear_events() {
    EVENT_QUEUE.lock().clear();
}

// =============================================================================
// REGISTRO DE DISPOSITIVOS
// =============================================================================

/// Registra um novo dispositivo de entrada.
pub fn register_device(device: Arc<dyn InputDevice>) {
    let name = device.name();
    crate::kinfo!("(Input) Registrando dispositivo:", name);

    INPUT_DEVICES.lock().push(device);
}

/// Remove um dispositivo de entrada.
pub fn unregister_device(name: &str) {
    crate::kinfo!("(Input) Removendo dispositivo:", name);
    INPUT_DEVICES.lock().retain(|d| d.name() != name);
}

/// Retorna número de dispositivos registrados.
pub fn device_count() -> usize {
    INPUT_DEVICES.lock().len()
}

/// Retorna lista de dispositivos.
pub fn get_devices() -> Vec<Arc<dyn InputDevice>> {
    INPUT_DEVICES.lock().clone()
}

// =============================================================================
// CAMADA DE COMPATIBILIDADE
// =============================================================================

/// Compatibilidade com handlers de interrupção existentes.
pub mod keyboard {
    pub use super::ps2::{keyboard_irq as handle_irq, pop_scancode, read_scancode};
}

/// Compatibilidade com handlers de mouse existentes.
pub mod mouse {
    pub use super::ps2::{mouse_get_state as get_state, mouse_irq as handle_irq};
}

// Re-exports globais para IRQ handlers
pub use ps2::{keyboard_irq as handle_keyboard_irq, mouse_irq as handle_mouse_irq};

// =============================================================================
// FUNÇÕES DE POLLING
// =============================================================================

/// Obtém o estado atual do teclado.
pub fn get_keyboard_state() -> KeyboardState {
    // TODO: Implementar estado completo
    KeyboardState::default()
}

/// Obtém o estado atual do ponteiro (mouse/touchpad).
pub fn get_pointer_state() -> PointerState {
    let s = ps2::mouse_get_state();
    PointerState {
        x: s.x,
        y: s.y,
        delta_x: s.delta_x,
        delta_y: s.delta_y,
        buttons: s.buttons,
        scroll_y: 0,
        scroll_x: 0,
    }
}

/// Obtém o estado atual do touch.
pub fn get_touch_state() -> Option<TouchState> {
    touch::get_state()
}
