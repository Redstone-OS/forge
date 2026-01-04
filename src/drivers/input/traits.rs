//! # Input Device Traits and Types
//!
//! Define as interfaces e tipos fundamentais para dispositivos de entrada.
//!
//! ## Tipos de Eventos:
//! - **Key**: Tecla pressionada/liberada
//! - **Pointer**: Movimento de mouse/touchpad
//! - **Touch**: Eventos de toque (multi-touch)
//! - **Scroll**: Eventos de rolagem

// TODO: Revisar no futuro
#[allow(unused_imports)]
use alloc::string::String;

// =============================================================================
// EVENTO DE ENTRADA UNIFICADO
// =============================================================================

/// Evento de entrada unificado.
///
/// Representa qualquer tipo de entrada do usuário.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Evento de teclado.
    Key(KeyEvent),

    /// Evento de ponteiro (mouse/touchpad).
    Pointer(PointerEvent),

    /// Evento de toque.
    Touch(TouchEvent),

    /// Evento de scroll.
    Scroll(ScrollEvent),

    /// Botão de gamepad/joystick.
    Button(ButtonEvent),
}

impl InputEvent {
    /// Retorna timestamp do evento (se disponível).
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::Key(e) => e.timestamp,
            Self::Pointer(e) => e.timestamp,
            Self::Touch(e) => e.timestamp,
            Self::Scroll(e) => e.timestamp,
            Self::Button(e) => e.timestamp,
        }
    }
}

// =============================================================================
// EVENTOS DE TECLADO
// =============================================================================

/// Evento de teclado.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// Scancode físico da tecla.
    pub scancode: u16,

    /// Keycode virtual (layout-independent).
    pub keycode: KeyCode,

    /// Estado da tecla.
    pub state: KeyState,

    /// Modificadores ativos.
    pub modifiers: Modifiers,

    /// Timestamp em microsegundos.
    pub timestamp: u64,
}

impl KeyEvent {
    /// Cria evento a partir de scancode bruto.
    pub fn from_scancode(scancode: u8) -> Self {
        let pressed = (scancode & 0x80) == 0;
        let scancode_clean = scancode & 0x7F;

        Self {
            scancode: scancode_clean as u16,
            keycode: scancode_to_keycode(scancode_clean),
            state: if pressed {
                KeyState::Pressed
            } else {
                KeyState::Released
            },
            modifiers: Modifiers::empty(),
            timestamp: 0, // Será preenchido pelo sistema
        }
    }

    /// Verifica se é key press.
    pub fn is_pressed(&self) -> bool {
        self.state == KeyState::Pressed
    }

    /// Verifica se é key release.
    pub fn is_released(&self) -> bool {
        self.state == KeyState::Released
    }
}

/// Estado de uma tecla.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// Tecla pressionada.
    Pressed,
    /// Tecla liberada.
    Released,
    /// Tecla em repeat.
    Repeat,
}

/// Modificadores de teclado.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Windows/Super/Command
    pub caps_lock: bool,
    pub num_lock: bool,
}

impl Modifiers {
    pub const fn empty() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
            caps_lock: false,
            num_lock: false,
        }
    }
}

/// Keycode virtual (independente de layout físico).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum KeyCode {
    Unknown = 0,

    // Letras
    A = 4,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Números
    Num1 = 30,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,

    // Função
    Escape = 41,
    Backspace = 42,
    Tab = 43,
    Space = 44,
    Enter = 40,

    // Modificadores
    LShift = 225,
    RShift = 229,
    LCtrl = 224,
    RCtrl = 228,
    LAlt = 226,
    RAlt = 230,
    LMeta = 227,
    RMeta = 231,

    // Setas
    Up = 82,
    Down = 81,
    Left = 80,
    Right = 79,

    // Teclas de função
    F1 = 58,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Outros
    Insert = 73,
    Delete = 76,
    Home = 74,
    End = 77,
    PageUp = 75,
    PageDown = 78,
    PrintScreen = 70,
    ScrollLock = 71,
    Pause = 72,
    CapsLock = 57,
    NumLock = 83,
}

/// Converte scancode PS/2 para KeyCode.
fn scancode_to_keycode(scancode: u8) -> KeyCode {
    match scancode {
        0x01 => KeyCode::Escape,
        0x02..=0x0A => KeyCode::Num1, // Simplificação
        0x0E => KeyCode::Backspace,
        0x0F => KeyCode::Tab,
        0x1C => KeyCode::Enter,
        0x1D => KeyCode::LCtrl,
        0x2A => KeyCode::LShift,
        0x36 => KeyCode::RShift,
        0x38 => KeyCode::LAlt,
        0x39 => KeyCode::Space,
        0x3A => KeyCode::CapsLock,
        0x3B..=0x44 => KeyCode::F1, // Simplificação para F1-F10
        0x48 => KeyCode::Up,
        0x4B => KeyCode::Left,
        0x4D => KeyCode::Right,
        0x50 => KeyCode::Down,
        _ => KeyCode::Unknown,
    }
}

// =============================================================================
// EVENTOS DE PONTEIRO
// =============================================================================

/// Evento de ponteiro (mouse/touchpad).
#[derive(Debug, Clone, Copy)]
pub struct PointerEvent {
    /// Posição X absoluta.
    pub x: i32,
    /// Posição Y absoluta.
    pub y: i32,
    /// Delta X relativo.
    pub delta_x: i32,
    /// Delta Y relativo.
    pub delta_y: i32,
    /// Estado dos botões.
    pub buttons: PointerButtons,
    /// Timestamp.
    pub timestamp: u64,
}

/// Estado dos botões do ponteiro.
#[derive(Debug, Clone, Copy, Default)]
pub struct PointerButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub button4: bool,
    pub button5: bool,
}

impl PointerButtons {
    /// Cria a partir de bitmap.
    pub fn from_bits(bits: u8) -> Self {
        Self {
            left: (bits & 0x01) != 0,
            right: (bits & 0x02) != 0,
            middle: (bits & 0x04) != 0,
            button4: (bits & 0x08) != 0,
            button5: (bits & 0x10) != 0,
        }
    }

    /// Converte para bitmap.
    pub fn to_bits(&self) -> u8 {
        let mut bits = 0u8;
        if self.left {
            bits |= 0x01;
        }
        if self.right {
            bits |= 0x02;
        }
        if self.middle {
            bits |= 0x04;
        }
        if self.button4 {
            bits |= 0x08;
        }
        if self.button5 {
            bits |= 0x10;
        }
        bits
    }
}

/// Estado do ponteiro (compatibilidade).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PointerState {
    /// Posição X absoluta.
    pub x: i32,
    /// Posição Y absoluta.
    pub y: i32,
    /// Delta X desde última leitura.
    pub delta_x: i32,
    /// Delta Y desde última leitura.
    pub delta_y: i32,
    /// Botões (bit 0 = esquerdo, bit 1 = direito, bit 2 = meio).
    pub buttons: u8,
    /// Scroll vertical.
    pub scroll_y: i8,
    /// Scroll horizontal.
    pub scroll_x: i8,
}

impl PointerState {
    /// Botão esquerdo pressionado.
    pub fn left_button(&self) -> bool {
        (self.buttons & 0x01) != 0
    }

    /// Botão direito pressionado.
    pub fn right_button(&self) -> bool {
        (self.buttons & 0x02) != 0
    }

    /// Botão do meio pressionado.
    pub fn middle_button(&self) -> bool {
        (self.buttons & 0x04) != 0
    }
}

/// Tipo de dispositivo de ponteiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerType {
    Ps2Mouse,
    Touchpad,
    UsbMouse,
    VirtioMouse,
    VirtioTablet,
    None,
}

// =============================================================================
// EVENTOS DE TOQUE
// =============================================================================

/// Evento de toque.
#[derive(Debug, Clone, Copy)]
pub struct TouchEvent {
    /// ID do dedo/contato.
    pub id: u8,
    /// Tipo de evento.
    pub event_type: TouchEventType,
    /// Posição X.
    pub x: i32,
    /// Posição Y.
    pub y: i32,
    /// Pressão (0-255).
    pub pressure: u8,
    /// Área do contato.
    pub area: u16,
    /// Timestamp.
    pub timestamp: u64,
}

/// Tipo de evento de toque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchEventType {
    /// Dedo tocou a tela.
    Down,
    /// Dedo moveu.
    Move,
    /// Dedo levantou.
    Up,
    /// Toque cancelado.
    Cancel,
}

/// Estado de touch (multi-touch).
#[derive(Debug, Clone, Default)]
pub struct TouchState {
    /// Contatos ativos.
    pub contacts: [Option<TouchContact>; 10],
    /// Número de contatos ativos.
    pub count: u8,
}

/// Contato de touch individual.
#[derive(Debug, Clone, Copy, Default)]
pub struct TouchContact {
    pub id: u8,
    pub x: i32,
    pub y: i32,
    pub pressure: u8,
}

// =============================================================================
// EVENTOS DE SCROLL
// =============================================================================

/// Evento de scroll.
#[derive(Debug, Clone, Copy)]
pub struct ScrollEvent {
    /// Delta X (horizontal).
    pub delta_x: i32,
    /// Delta Y (vertical).
    pub delta_y: i32,
    /// Scroll é preciso (hi-res)?
    pub precise: bool,
    /// Timestamp.
    pub timestamp: u64,
}

// =============================================================================
// EVENTOS DE BOTÃO
// =============================================================================

/// Evento de botão genérico (gamepad, etc).
#[derive(Debug, Clone, Copy)]
pub struct ButtonEvent {
    /// Código do botão.
    pub button: u16,
    /// Pressionado/Liberado.
    pub pressed: bool,
    /// Timestamp.
    pub timestamp: u64,
}

// =============================================================================
// ESTADO DO TECLADO
// =============================================================================

/// Estado completo do teclado.
#[derive(Debug, Clone, Default)]
pub struct KeyboardState {
    /// Bitmap de teclas pressionadas (256 bits).
    pub keys: [u64; 4],
    /// Modificadores ativos.
    pub modifiers: Modifiers,
}

impl KeyboardState {
    /// Verifica se uma tecla está pressionada.
    pub fn is_key_pressed(&self, keycode: u16) -> bool {
        let index = (keycode / 64) as usize;
        let bit = keycode % 64;
        if index >= 4 {
            return false;
        }
        (self.keys[index] & (1 << bit)) != 0
    }

    /// Define estado de uma tecla.
    pub fn set_key(&mut self, keycode: u16, pressed: bool) {
        let index = (keycode / 64) as usize;
        let bit = keycode % 64;
        if index >= 4 {
            return;
        }
        if pressed {
            self.keys[index] |= 1 << bit;
        } else {
            self.keys[index] &= !(1 << bit);
        }
    }
}

// =============================================================================
// TRAIT DE DISPOSITIVO DE ENTRADA
// =============================================================================

/// Interface para dispositivos de entrada.
pub trait InputDevice: Send + Sync {
    /// Retorna nome do dispositivo.
    fn name(&self) -> &str;

    /// Retorna tipo do dispositivo.
    fn device_type(&self) -> InputDeviceType;

    /// Verifica se dispositivo está conectado.
    fn is_connected(&self) -> bool;

    /// Habilita o dispositivo.
    fn enable(&self) -> bool;

    /// Desabilita o dispositivo.
    fn disable(&self);

    /// Polling de eventos (se suportado).
    fn poll(&self) -> Option<InputEvent>;
}

/// Tipo de dispositivo de entrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Touchpad,
    Touchscreen,
    Gamepad,
    Tablet,
    Other,
}
