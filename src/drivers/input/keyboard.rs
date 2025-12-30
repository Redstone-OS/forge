//! PS/2 Keyboard Driver (Interrupt Driven)

use crate::arch::x86_64::ports::inb;
use crate::sync::Spinlock;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const BUFFER_SIZE: usize = 256;

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
        } else {
            // Buffer full, drop packet
            crate::kwarn!("(KBD) Buffer full");
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
}

static KBD_BUFFER: Spinlock<KeyboardBuffer> = Spinlock::new(KeyboardBuffer::new());

pub fn init() {
    // Limpar buffer do controlador
    while (inb(STATUS_PORT) & 0x01) != 0 {
        inb(DATA_PORT);
    }

    // Habilitar IRQ 1 no PIC/APIC é feito em interrupts.rs ou init_pics
    // Mas precisamos garantir que flag de interrupção do teclado esteja ativa no controlador 0x64?
    // O BIOS geralmente deixa ativo.

    crate::kinfo!("(Input) PS/2 Keyboard Initialized (Interrupt Mode)");
}

/// Handler de Interrupção (Chamado pelo ASM wrapper do IRQ 1)
pub fn handle_irq() {
    let status = inb(STATUS_PORT);
    if (status & 0x01) != 0 {
        let scancode = inb(DATA_PORT);
        KBD_BUFFER.lock().push(scancode);
    }
}

/// Consome um scancode do buffer
pub fn pop_scancode() -> Option<u8> {
    KBD_BUFFER.lock().pop()
}

// Manter compatibilidade caso algo use polling direto (opcional)
pub fn read_scancode() -> Option<u8> {
    pop_scancode()
}
