//! PS/2 Keyboard Driver (Interrupt Driven)

use crate::arch::x86_64::ports::{inb, outb};
use crate::sync::Spinlock;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const CMD_PORT: u16 = 0x64;
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
    // 1. Habilitar a porta do teclado no controlador 8042
    unsafe {
        wait_write();
        outb(CMD_PORT, 0xAE); // Enable first PS/2 port
    }

    // 2. Limpar buffer do controlador
    while (inb(STATUS_PORT) & 0x01) != 0 {
        inb(DATA_PORT);
    }

    // 3. Configurar o Command Byte para habilitar interrupções de teclado (Bit 0)
    // Preservamos os outros bits para não quebrar o mouse se ele já estiver ativo
    unsafe {
        wait_write();
        outb(CMD_PORT, 0x20); // Ler Command Byte
        wait_read();
        let config = inb(DATA_PORT) | 0x01; // Forçar Bit 0 (Keyboard IRQ)

        wait_write();
        outb(CMD_PORT, 0x60); // Escrever Command Byte
        wait_write();
        outb(DATA_PORT, config);
    }

    // 4. Habilitar IRQ 1 no PIC
    crate::arch::x86_64::interrupts::pic_enable_irq(1);

    crate::kinfo!("(Input) PS/2 Keyboard Initialized (Interrupt Mode)");
}

/// Helper para esperar o buffer de entrada do controlador estar vazio (pronto para escrita)
fn wait_write() {
    while (inb(STATUS_PORT) & 0x02) != 0 {
        core::hint::spin_loop();
    }
}

/// Helper para esperar o buffer de saída do controlador ter dados (pronto para leitura)
fn wait_read() {
    while (inb(STATUS_PORT) & 0x01) == 0 {
        core::hint::spin_loop();
    }
}

/// Handler de Interrupção (Chamado pelo ASM wrapper do IRQ 1)
pub fn handle_irq() {
    // IMPORTANTE: Quando a IRQ 1 é disparada, SEMPRE há um scancode disponível.
    // A verificação do status bit é desnecessária e pode causar race conditions
    // onde o bit já foi limpo por outra operação.
    let scancode = inb(DATA_PORT);
    crate::kdebug!("(KBD) IRQ: scancode=", scancode as u64);
    KBD_BUFFER.lock().push(scancode);
}

/// Consome um scancode do buffer
pub fn pop_scancode() -> Option<u8> {
    KBD_BUFFER.lock().pop()
}

// Manter compatibilidade caso algo use polling direto (opcional)
pub fn read_scancode() -> Option<u8> {
    pop_scancode()
}
