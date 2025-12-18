//! PS/2 Keyboard Driver
//!
//! Driver baseado na arquitetura do pc-keyboard, mas implementado do zero.
//! Usa state machine para processar scancodes corretamente.

use core::arch::asm;
use spin::Mutex;

/// Estado do decoder de scancodes
#[derive(Debug, Copy, Clone, PartialEq)]
enum DecodeState {
    /// Estado inicial
    Start,
    /// Recebeu 0xE0 (extended key)
    Extended,
}

/// Decoder de Scancode Set 1
pub struct ScancodeDecoder {
    state: DecodeState,
    modifiers: Modifiers,
}

/// Modificadores de teclado
#[derive(Debug, Copy, Clone)]
struct Modifiers {
    lshift: bool,
    rshift: bool,
    caps_lock: bool,
}

impl Modifiers {
    const fn new() -> Self {
        Self {
            lshift: false,
            rshift: false,
            caps_lock: false,
        }
    }

    fn is_shifted(&self) -> bool {
        self.lshift || self.rshift
    }
}

impl ScancodeDecoder {
    const fn new() -> Self {
        Self {
            state: DecodeState::Start,
            modifiers: Modifiers::new(),
        }
    }

    /// Processa um scancode e retorna o caractere (se houver)
    fn process_scancode(&mut self, code: u8) -> Option<char> {
        match self.state {
            DecodeState::Start => {
                if code == 0xE0 {
                    // Extended key prefix
                    self.state = DecodeState::Extended;
                    None
                } else if code >= 0x80 {
                    // Break code (tecla liberada)
                    self.handle_key_up(code - 0x80);
                    None
                } else {
                    // Make code (tecla pressionada)
                    self.handle_key_down(code)
                }
            }
            DecodeState::Extended => {
                // Volta ao estado inicial
                self.state = DecodeState::Start;

                if code >= 0x80 {
                    // Extended break code
                    None
                } else {
                    // Extended make code - ignorar por enquanto
                    None
                }
            }
        }
    }

    /// Trata tecla pressionada
    fn handle_key_down(&mut self, code: u8) -> Option<char> {
        match code {
            // Modificadores
            0x2A => {
                self.modifiers.lshift = true;
                None
            } // Left Shift
            0x36 => {
                self.modifiers.rshift = true;
                None
            } // Right Shift
            0x3A => {
                self.modifiers.caps_lock = !self.modifiers.caps_lock;
                None
            } // Caps Lock

            // Letras (linha QWERTY)
            0x10 => Some(self.apply_modifiers('q')),
            0x11 => Some(self.apply_modifiers('w')),
            0x12 => Some(self.apply_modifiers('e')),
            0x13 => Some(self.apply_modifiers('r')),
            0x14 => Some(self.apply_modifiers('t')),
            0x15 => Some(self.apply_modifiers('y')),
            0x16 => Some(self.apply_modifiers('u')),
            0x17 => Some(self.apply_modifiers('i')),
            0x18 => Some(self.apply_modifiers('o')),
            0x19 => Some(self.apply_modifiers('p')),

            // Letras (linha ASDF)
            0x1E => Some(self.apply_modifiers('a')),
            0x1F => Some(self.apply_modifiers('s')),
            0x20 => Some(self.apply_modifiers('d')),
            0x21 => Some(self.apply_modifiers('f')),
            0x22 => Some(self.apply_modifiers('g')),
            0x23 => Some(self.apply_modifiers('h')),
            0x24 => Some(self.apply_modifiers('j')),
            0x25 => Some(self.apply_modifiers('k')),
            0x26 => Some(self.apply_modifiers('l')),

            // Letras (linha ZXCV)
            0x2C => Some(self.apply_modifiers('z')),
            0x2D => Some(self.apply_modifiers('x')),
            0x2E => Some(self.apply_modifiers('c')),
            0x2F => Some(self.apply_modifiers('v')),
            0x30 => Some(self.apply_modifiers('b')),
            0x31 => Some(self.apply_modifiers('n')),
            0x32 => Some(self.apply_modifiers('m')),

            // Números
            0x02 => Some(if self.modifiers.is_shifted() {
                '!'
            } else {
                '1'
            }),
            0x03 => Some(if self.modifiers.is_shifted() {
                '@'
            } else {
                '2'
            }),
            0x04 => Some(if self.modifiers.is_shifted() {
                '#'
            } else {
                '3'
            }),
            0x05 => Some(if self.modifiers.is_shifted() {
                '$'
            } else {
                '4'
            }),
            0x06 => Some(if self.modifiers.is_shifted() {
                '%'
            } else {
                '5'
            }),
            0x07 => Some(if self.modifiers.is_shifted() {
                '^'
            } else {
                '6'
            }),
            0x08 => Some(if self.modifiers.is_shifted() {
                '&'
            } else {
                '7'
            }),
            0x09 => Some(if self.modifiers.is_shifted() {
                '*'
            } else {
                '8'
            }),
            0x0A => Some(if self.modifiers.is_shifted() {
                '('
            } else {
                '9'
            }),
            0x0B => Some(if self.modifiers.is_shifted() {
                ')'
            } else {
                '0'
            }),

            // Símbolos
            0x0C => Some(if self.modifiers.is_shifted() {
                '_'
            } else {
                '-'
            }),
            0x0D => Some(if self.modifiers.is_shifted() {
                '+'
            } else {
                '='
            }),
            0x33 => Some(if self.modifiers.is_shifted() {
                '<'
            } else {
                ','
            }),
            0x34 => Some(if self.modifiers.is_shifted() {
                '>'
            } else {
                '.'
            }),
            0x35 => Some(if self.modifiers.is_shifted() {
                '?'
            } else {
                '/'
            }),
            0x27 => Some(if self.modifiers.is_shifted() {
                ':'
            } else {
                ';'
            }),
            0x28 => Some(if self.modifiers.is_shifted() {
                '"'
            } else {
                '\''
            }),
            0x1A => Some(if self.modifiers.is_shifted() {
                '{'
            } else {
                '['
            }),
            0x1B => Some(if self.modifiers.is_shifted() {
                '}'
            } else {
                ']'
            }),
            0x2B => Some(if self.modifiers.is_shifted() {
                '|'
            } else {
                '\\'
            }),
            0x29 => Some(if self.modifiers.is_shifted() {
                '~'
            } else {
                '`'
            }),

            // Teclas especiais
            0x39 => Some(' '),    // Space
            0x1C => Some('\n'),   // Enter
            0x0E => Some('\x08'), // Backspace
            0x0F => Some('\t'),   // Tab

            _ => None,
        }
    }

    /// Trata tecla liberada
    fn handle_key_up(&mut self, code: u8) {
        match code {
            0x2A => self.modifiers.lshift = false, // Left Shift
            0x36 => self.modifiers.rshift = false, // Right Shift
            _ => {}
        }
    }

    /// Aplica modificadores (Shift, Caps Lock) a uma letra
    fn apply_modifiers(&self, ch: char) -> char {
        if ch.is_alphabetic() {
            let should_uppercase = self.modifiers.is_shifted() ^ self.modifiers.caps_lock;
            if should_uppercase {
                ch.to_ascii_uppercase()
            } else {
                ch
            }
        } else {
            ch
        }
    }
}

/// Decoder global
static DECODER: Mutex<ScancodeDecoder> = Mutex::new(ScancodeDecoder::new());

/// Inicializa o teclado
pub fn init() {
    // Nada a fazer - decoder já inicializado
}

/// Lê scancode da porta 0x60
pub fn read_scancode() -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") 0x60u16, out("al") value, options(nostack, preserves_flags));
    }
    value
}

/// Processa scancode e retorna caractere
pub fn process_scancode(scancode: u8) -> Option<char> {
    DECODER.lock().process_scancode(scancode)
}
