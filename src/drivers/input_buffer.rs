//! Input Buffer
//!
//! Buffer circular thread-safe para armazenar caracteres de input.

use spin::Mutex;

extern crate alloc;
use alloc::string::String;

/// Buffer circular para input
pub struct InputBuffer {
    buffer: [char; 256],
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl InputBuffer {
    /// Cria novo buffer vazio
    pub const fn new() -> Self {
        Self {
            buffer: ['\0'; 256],
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    /// Adiciona caractere ao buffer
    pub fn push(&mut self, ch: char) {
        if self.count < 256 {
            self.buffer[self.write_pos] = ch;
            self.write_pos = (self.write_pos + 1) % 256;
            self.count += 1;
        }
    }

    /// Remove e retorna próximo caractere
    pub fn pop(&mut self) -> Option<char> {
        if self.count > 0 {
            let ch = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % 256;
            self.count -= 1;
            Some(ch)
        } else {
            None
        }
    }

    /// Verifica se tem caracteres disponíveis
    pub fn has_data(&self) -> bool {
        self.count > 0
    }

    /// Retorna número de caracteres no buffer
    pub fn len(&self) -> usize {
        self.count
    }

    /// Verifica se buffer está vazio
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Buffer global de input
pub static INPUT_BUFFER: Mutex<InputBuffer> = Mutex::new(InputBuffer::new());
