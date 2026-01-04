//! # USB Input Devices
//!
//! Este módulo fornece a ponte entre o USB stack e o subsistema de input
//! para dispositivos USB HID.
//!
//! ## Dispositivos:
//! - Teclados USB
//! - Mouses USB
//! - Gamepads USB
//! - Outros dispositivos HID
//!
//! ## Fluxo:
//! 1. USB stack detecta dispositivo HID
//! 2. Este módulo é notificado
//! 3. Report Descriptor é parseado
//! 4. Dispositivo é registrado no input subsystem
//! 5. Reports são convertidos em InputEvents
//!
//! ## STUB:
//! Estrutura definida. Implementação pendente após USB stack.

use super::hid;
use super::traits::*;
use crate::drivers::bus::usb::device::UsbDevice;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static DEVICES: Spinlock<Vec<Arc<UsbInputDevice>>> = Spinlock::new(Vec::new());
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// DRIVER USB INPUT
// =============================================================================

/// Driver para dispositivo USB Input.
pub struct UsbInputDevice {
    /// Nome do dispositivo.
    name: alloc::string::String,
    /// Tipo de dispositivo.
    device_type: InputDeviceType,
    /// Endereço USB.
    usb_address: u8,
    /// Report Descriptor parseado.
    hid_descriptor: Option<hid::types::HidReportDescriptor>,
    /// Habilitado?
    enabled: Spinlock<bool>,
}

impl UsbInputDevice {
    /// Cria novo dispositivo USB Input.
    pub fn new(name: &str, device_type: InputDeviceType, usb_address: u8) -> Self {
        Self {
            name: alloc::string::String::from(name),
            device_type,
            usb_address,
            hid_descriptor: None,
            enabled: Spinlock::new(false),
        }
    }

    /// Probe de dispositivo USB.
    ///
    /// ## STUB:
    /// Não implementado.
    pub fn probe(_usb_dev: &UsbDevice) -> Option<Self> {
        crate::kwarn!("(USB Input) probe() não implementado");
        None
    }

    /// Processa HID report.
    pub fn process_report(&self, data: &[u8]) -> Vec<InputEvent> {
        let mut events = Vec::new();

        if let Some(ref descriptor) = self.hid_descriptor {
            let values = hid::parse_report(descriptor, data);

            // Converte valores para eventos
            for (usage_page, usage, value) in values {
                if let Some(event) = self.value_to_event(usage_page, usage, value) {
                    events.push(event);
                }
            }
        }

        events
    }

    /// Converte valor HID para InputEvent.
    fn value_to_event(&self, usage_page: u16, usage: u16, value: i32) -> Option<InputEvent> {
        match usage_page {
            hid::usage::USAGE_PAGE_KEYBOARD => {
                // Evento de teclado
                if value != 0 {
                    Some(InputEvent::Key(KeyEvent {
                        scancode: usage,
                        keycode: KeyCode::Unknown,
                        state: KeyState::Pressed,
                        modifiers: Modifiers::empty(),
                        timestamp: 0,
                    }))
                } else {
                    None
                }
            }
            hid::usage::USAGE_PAGE_BUTTON => {
                // Botão de mouse/gamepad
                Some(InputEvent::Button(ButtonEvent {
                    button: usage,
                    pressed: value != 0,
                    timestamp: 0,
                }))
            }
            hid::usage::USAGE_PAGE_GENERIC_DESKTOP => {
                // Mouse X/Y, Wheel, etc
                match usage {
                    hid::usage::USAGE_X | hid::usage::USAGE_Y => {
                        // Movimento de mouse
                        None // TODO: Acumular e gerar PointerEvent
                    }
                    hid::usage::USAGE_WHEEL => Some(InputEvent::Scroll(ScrollEvent {
                        delta_x: 0,
                        delta_y: value,
                        precise: false,
                        timestamp: 0,
                    })),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl InputDevice for UsbInputDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> InputDeviceType {
        self.device_type
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
        None // Usa IRQ/interrupt transfer
    }
}

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa subsistema USB Input.
pub fn init() {
    crate::kinfo!("(USB Input) Inicializando...");

    *INITIALIZED.lock() = true;

    crate::kinfo!("(USB Input) Subsistema inicializado (aguardando dispositivos)");
}

/// Desliga subsistema USB Input.
pub fn shutdown() {
    crate::kinfo!("(USB Input) Shutdown");
}

/// Registra um novo dispositivo USB HID.
pub fn register_device(device: Arc<UsbInputDevice>) {
    crate::kinfo!("(USB Input) Registrando:", device.name());
    DEVICES.lock().push(device.clone());
    super::register_device(device);
}

/// Remove um dispositivo USB HID.
pub fn unregister_device(usb_address: u8) {
    DEVICES.lock().retain(|d| d.usb_address != usb_address);
}

/// Retorna número de dispositivos USB Input.
pub fn device_count() -> usize {
    DEVICES.lock().len()
}
