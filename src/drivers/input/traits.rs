//! # Traits de Dispositivos de Entrada
//!
//! Abstrações comuns para dispositivos de input.

/// Estado de ponteiro (mouse/touchpad unificado)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PointerState {
    /// Posição X absoluta
    pub x: i32,
    /// Posição Y absoluta
    pub y: i32,
    /// Delta X desde última leitura
    pub delta_x: i32,
    /// Delta Y desde última leitura
    pub delta_y: i32,
    /// Botões (bit 0 = esquerdo, bit 1 = direito, bit 2 = meio)
    pub buttons: u8,
    /// Scroll vertical
    pub scroll_y: i8,
    /// Scroll horizontal
    pub scroll_x: i8,
}

impl PointerState {
    /// Botão esquerdo pressionado
    pub fn left_button(&self) -> bool {
        (self.buttons & 0x01) != 0
    }

    /// Botão direito pressionado
    pub fn right_button(&self) -> bool {
        (self.buttons & 0x02) != 0
    }

    /// Botão do meio pressionado
    pub fn middle_button(&self) -> bool {
        (self.buttons & 0x04) != 0
    }
}

/// Tipo de dispositivo de ponteiro
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerType {
    /// Mouse PS/2
    Ps2Mouse,
    /// Touchpad I2C
    Touchpad,
    /// Mouse USB
    UsbMouse,
    /// Mouse Virtualizado (QEMU)
    VirtioMouse,
    /// Tablet/Touch Virtualizado (Absolute positioning)
    VirtioTablet,
    /// Nenhum
    None,
}

/// Evento de teclado
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    /// Scancode do teclado
    pub scancode: u8,
    /// Se é key down ou key up
    pub pressed: bool,
}

impl KeyEvent {
    pub fn new(scancode: u8) -> Self {
        let pressed = (scancode & 0x80) == 0;
        let scancode = scancode & 0x7F;
        Self { scancode, pressed }
    }
}
