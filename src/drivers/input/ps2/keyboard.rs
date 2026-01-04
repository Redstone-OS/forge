//! # Driver PS/2 Keyboard
//!
//! Implementação do driver de teclado PS/2.

use super::io;
use super::ports::{commands, config};
use crate::sync::Spinlock;

/// Tamanho do buffer circular
const BUFFER_SIZE: usize = 256;

/// Buffer circular para scancodes
struct KeyboardBuffer {
    data: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            data: [0; BUFFER_SIZE],
            head: 0,
            tail: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        let next_head = (self.head + 1) % BUFFER_SIZE;
        if next_head != self.tail {
            self.data[self.head] = byte;
            self.head = next_head;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            None
        } else {
            let byte = self.data[self.tail];
            self.tail = (self.tail + 1) % BUFFER_SIZE;
            Some(byte)
        }
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.head == self.tail
    }
}

static KBD_BUFFER: Spinlock<KeyboardBuffer> = Spinlock::new(KeyboardBuffer::new());

/// Inicializa o teclado PS/2
pub fn init() {
    // 1. Habilitar porta do teclado
    io::send_command(commands::ENABLE_KEYBOARD);

    // 2. Limpar buffer
    io::flush_output();

    // 3. Configurar para habilitar IRQ do teclado
    if let Some(cfg) = io::read_config() {
        let new_cfg = cfg | config::KEYBOARD_IRQ;
        io::write_config(new_cfg);
    }

    // 4. Habilitar IRQ 1 no PIC
    crate::arch::x86_64::interrupts::pic_enable_irq(1);

    crate::kinfo!("(Input) PS/2 Keyboard initialized");
}

/// Handler de interrupção (IRQ 1)
pub fn handle_irq() {
    let scancode = io::read_data();
    crate::kdebug!("(KBD) IRQ: scancode=", scancode as u64);
    KBD_BUFFER.lock().push(scancode);
}

/// Consome um scancode do buffer
pub fn pop_scancode() -> Option<u8> {
    KBD_BUFFER.lock().pop()
}

/// Alias para compatibilidade
pub fn read_scancode() -> Option<u8> {
    pop_scancode()
}
