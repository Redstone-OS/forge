//! Input subsystems (Keyboard/Mouse)

pub mod keyboard;
pub mod mouse;

/// Inicializa subsistema de entrada
pub fn init() {
    keyboard::init();
    mouse::init();
}
