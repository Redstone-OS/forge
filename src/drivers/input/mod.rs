//! # Subsistema de Entrada (Input)
//!
//! Este módulo gerencia todos os dispositivos de entrada do RedstoneOS,
//! desde o teclado PS/2 legado até dispositivos USB HID complexos.
//!
//! ## Estrutura:
//! - **hid/**: Parser universal para dispositivos HID (USB, I2C).
//! - **ps2/**: Suporte para o controlador legado 8042 (Teclado/Mouse).
//! - **touch/**: Suporte para touchpads e telas sensíveis ao toque (I2C/HID).
//! - **virtio/**: Input virtualizado (QEMU/KVM).
//! - **usb/**: Bridge para dispositivos de entrada via barramento USB.

pub mod hid;
pub mod ps2;
pub mod touch;
pub mod traits;
pub mod usb;
pub mod virtio;

pub use traits::{KeyEvent, PointerState, PointerType};

/// Inicializa os drivers de entrada disponíveis.
pub fn init() {
    crate::kinfo!("(Input) Inicializando subsistema de entrada...");

    // 1. HID (Instancia o parser universal)
    hid::init();

    // 2. PS/2 (Suporte legado sempre ativo como fallback)
    ps2::init();

    // 3. VirtIO Input
    virtio::init();

    // 4. Touchpads e outros
    touch::init();

    crate::kdebug!("(Input) Todos os drivers de entrada registrados no RDM.");
}

// Camada de compatibilidade para syscalls e arch/ interrupt handlers
pub mod keyboard {
    pub use super::ps2::{keyboard_irq as handle_irq, pop_scancode, read_scancode};
}

pub mod mouse {
    pub use super::ps2::{mouse_get_state as get_state, mouse_irq as handle_irq};
}

// Re-exports globais
pub use ps2::{keyboard_irq as handle_keyboard_irq, mouse_irq as handle_mouse_irq};
