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
const SERIAL_BUFFER_MASK: usize = SERIAL_BUFFER_SIZE - 1;

pub struct SerialPort {
    buffer: [u8; SERIAL_BUFFER_SIZE],
    head: usize,
    tail: usize,
    dropped_count: usize,
}

static SERIAL: Spinlock<SerialPort> = Spinlock::new(SerialPort {
    buffer: [0; SERIAL_BUFFER_SIZE],
    head: 0,
    tail: 0,
    dropped_count: 0,
});

impl SerialPort {
    /// Inicializa a porta serial COM1
    pub fn init(&mut self) {
        // Desabilitar interrupções do hardware
        outb(COM1_PORT + INT_ENABLE, 0x00);

        // Setar Baud Rate (DLAB enabled)
        outb(COM1_PORT + LINE_CTRL, 0x80);
        outb(COM1_PORT + DATA_REG, 0x01); // Divisor Low = 1 (115200 baud)
        outb(COM1_PORT + INT_ENABLE, 0x00); // Divisor High

        // Configurar linha: 8 bits, sem paridade, 1 stop bit
        outb(COM1_PORT + LINE_CTRL, 0x03);

        // Habilitar FIFO, limpar buffers, 14-byte threshold
        outb(COM1_PORT + FIFO_CTRL, 0xC7);

        // Habilitar IRQs (Master), RTS/DSR set
        outb(COM1_PORT + MODEM_CTRL, 0x0B);
    }

    /// Verifica se pode transmitir
    fn is_transmit_empty(&self) -> bool {
        inb(COM1_PORT + LINE_STATUS) & 0x20 != 0
    }

    /// Escreve byte no buffer circular interna (requer lock ja adquirido)
    fn write_byte_internal(&mut self, byte: u8) {
        let next_head = (self.head + 1) & SERIAL_BUFFER_MASK;

        // Se o buffer estiver cheio, avançamos a cauda (perdemos o mais antigo)
        if next_head == self.tail {
            self.tail = (self.tail + 1) & SERIAL_BUFFER_MASK;
            self.dropped_count += 1;
        }

        self.buffer[self.head] = byte;
        self.head = next_head;

        // Tenta enviar o que puder
        self.drain_internal();
    }

    /// Tenta enviar o máximo de bytes possível sem bloquear (requer lock)
    fn drain_internal(&mut self) {
        while self.head != self.tail && self.is_transmit_empty() {
            outb(COM1_PORT, self.buffer[self.tail]);
            self.tail = (self.tail + 1) & SERIAL_BUFFER_MASK;
        }
    }

    /// Força a descarga total do buffer (bloqueante).
    /// Útil para situações críticas como pânico.
    pub fn force_flush(&mut self) {
        while self.head != self.tail {
            while !self.is_transmit_empty() {
                core::hint::spin_loop();
            }
            outb(COM1_PORT, self.buffer[self.tail]);
            self.tail = (self.tail + 1) & SERIAL_BUFFER_MASK;
        }
    }

    /// Escreve hex interno (sem lock).
    /// Removido o prefixo " 0x" pois já é gerado pelo klog SerialDebug trait.
    fn write_hex_internal(&mut self, value: u64) {
        for i in (0..16).rev() {
            let digit = ((value >> (i * 4)) & 0xF) as u8;
            let c = if digit < 10 {
                b'0' + digit
            } else {
                b'A' + digit - 10
            };
            self.write_byte_internal(c);
        }
    }
}

/// Inicializa serial
pub fn init() {
    SERIAL.lock().init();
}

/// Tenta descarregar o buffer (non-blocking)
pub fn try_drain() {
    SERIAL.lock().drain_internal();
}

/// Escreve uma linha completa de forma atômica (um único lock)
pub fn write_log(prefix: &str, msg: &str, val: Option<u64>) {
    let mut serial = SERIAL.lock();
    for b in prefix.bytes() {
        serial.write_byte_internal(b);
    }
    for b in msg.bytes() {
        serial.write_byte_internal(b);
    }
    if let Some(v) = val {
        serial.write_byte_internal(b' ');
        serial.write_byte_internal(b'0');
        serial.write_byte_internal(b'x');
        serial.write_hex_internal(v);
    }
    serial.write_byte_internal(b'\n');
}

/// Escreve byte (com lock)
pub fn write_byte(byte: u8) {
    SERIAL.lock().write_byte_internal(byte);
}

/// Emite byte (alias para write_byte)
pub fn emit(byte: u8) {
    write_byte(byte);
}

/// Escreve string (atômico)
pub fn write_str(s: &str) {
    let mut serial = SERIAL.lock();
    for byte in s.bytes() {
        serial.write_byte_internal(byte);
    }
}

/// Força a descarga total do buffer (bloqueante)
pub fn force_flush() {
    SERIAL.lock().force_flush();
}

/// Escreve número hexadecimal
pub fn write_hex(value: u64) {
    let mut serial = SERIAL.lock();
    serial.write_byte_internal(b'0');
    serial.write_byte_internal(b'x');
    serial.write_hex_internal(value);
}
