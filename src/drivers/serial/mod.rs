//! Driver de porta serial COM1 (0x3F8)
//!
//! Driver simples para saída serial via porta COM1.
//! Utilizado como fallback e debug principal.

use crate::arch::x86_64::ports::{inb, outb};
use crate::sync::Spinlock;

/// Endereço base da porta COM1
const COM1_PORT: u16 = 0x3F8;

const DATA_REG: u16 = 0;
const INT_ENABLE: u16 = 1;
const FIFO_CTRL: u16 = 2;
const LINE_CTRL: u16 = 3;
const MODEM_CTRL: u16 = 4;
const LINE_STATUS: u16 = 5;

const SERIAL_BUFFER_SIZE: usize = 16 * 1024; // 16KB

pub struct SerialPort {
    buffer: [u8; SERIAL_BUFFER_SIZE],
    head: usize,
    tail: usize,
}

static SERIAL: Spinlock<SerialPort> = Spinlock::new(SerialPort {
    buffer: [0; SERIAL_BUFFER_SIZE],
    head: 0,
    tail: 0,
});

impl SerialPort {
    /// Inicializa a porta serial COM1
    pub fn init(&mut self) {
        // Desabilitar interrupções
        outb(COM1_PORT + INT_ENABLE, 0x00);

        // Setar Baud Rate (DLAB enabled)
        outb(COM1_PORT + LINE_CTRL, 0x80);
        outb(COM1_PORT + DATA_REG, 0x01); // Divisor Low = 1 (115200 baud)
        outb(COM1_PORT + INT_ENABLE, 0x00); // Divisor High

        // Configurar linha: 8 bits, sem paridade, 1 stop bit
        outb(COM1_PORT + LINE_CTRL, 0x03);

        // Habilitar FIFO, limpar buffers, 14-byte threshold
        outb(COM1_PORT + FIFO_CTRL, 0xC7);

        // Habilitar IRQs, RTS/DSR set
        outb(COM1_PORT + MODEM_CTRL, 0x0B);
    }

    /// Verifica se pode transmitir
    fn is_transmit_empty(&self) -> bool {
        inb(COM1_PORT + LINE_STATUS) & 0x20 != 0
    }

    /// Escreve byte no buffer circular e tenta enviar
    pub fn write_byte(&mut self, byte: u8) {
        // 1. Inserir no buffer
        self.buffer[self.head] = byte;
        let next_head = (self.head + 1) % SERIAL_BUFFER_SIZE;

        // Se bateu na cauda, perdemos o dado mais antigo (avança cauda)
        if next_head == self.tail {
            self.tail = (self.tail + 1) % SERIAL_BUFFER_SIZE;
        }
        self.head = next_head;

        // 2. Tenta "descarregar" o buffer enquanto o hardware estiver livre
        self.try_drain();
    }

    /// Tenta enviar o máximo de bytes possível sem bloquear
    pub fn try_drain(&mut self) {
        while self.head != self.tail && self.is_transmit_empty() {
            outb(COM1_PORT, self.buffer[self.tail]);
            self.tail = (self.tail + 1) % SERIAL_BUFFER_SIZE;
        }
    }

    /// Força a descarga total do buffer (bloqueante).
    /// Útil para situações críticas como pânico.
    pub fn force_flush(&mut self) {
        while self.head != self.tail {
            // Espera hardware estar livre
            while !self.is_transmit_empty() {
                core::hint::spin_loop();
            }
            outb(COM1_PORT, self.buffer[self.tail]);
            self.tail = (self.tail + 1) % SERIAL_BUFFER_SIZE;
        }
    }
}

/// Inicializa serial
pub fn init() {
    SERIAL.lock().init();
}

/// Tenta descarregar o buffer (non-blocking)
pub fn try_drain() {
    SERIAL.lock().try_drain();
}

/// Escreve byte
pub fn write_byte(byte: u8) {
    SERIAL.lock().write_byte(byte);
}

/// Emite byte (alias para write_byte)
pub fn emit(byte: u8) {
    write_byte(byte);
}

/// Escreve string
pub fn write_str(s: &str) {
    let mut serial = SERIAL.lock();
    for byte in s.bytes() {
        serial.write_byte(byte);
    }
}

/// Força a descarga total do buffer (bloqueante)
pub fn force_flush() {
    SERIAL.lock().force_flush();
}

/// Escreve número hexadecimal
pub fn write_hex(value: u64) {
    let mut serial = SERIAL.lock();
    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as u8;
        let c = if digit < 10 {
            b'0' + digit
        } else {
            b'A' + digit - 10
        };
        serial.write_byte(c);
    }
}
