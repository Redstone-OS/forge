//! Driver de porta serial (UART 16550)

use crate::arch::x86_64::ports::{inb, outb};
use crate::sync::Spinlock;

/// Porta COM1
const COM1_PORT: u16 = 0x3F8;

/// Estado da serial
static SERIAL: Spinlock<SerialPort> = Spinlock::new(SerialPort::new(COM1_PORT));

struct SerialPort {
    port: u16,
    initialized: bool,
}

impl SerialPort {
    const fn new(port: u16) -> Self {
        Self { port, initialized: false }
    }
    
    fn init(&mut self) {
        if self.initialized {
            return;
        }
        
        // Desabilitar interrupções
        outb(self.port + 1, 0x00);
        // Habilitar DLAB (set baud rate)
        outb(self.port + 3, 0x80);
        // Divisor low byte (115200 baud)
        outb(self.port + 0, 0x03);
        // Divisor high byte
        outb(self.port + 1, 0x00);
        // 8 bits, no parity, 1 stop bit
        outb(self.port + 3, 0x03);
        // Enable FIFO
        outb(self.port + 2, 0xC7);
        // IRQs enabled, RTS/DSR set
        outb(self.port + 4, 0x0B);
        
        self.initialized = true;
    }
    
    fn is_transmit_empty(&self) -> bool {
        (inb(self.port + 5) & 0x20) != 0
    }
    
    fn write_byte(&self, byte: u8) {
        // Esperar FIFO estar pronto
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        outb(self.port, byte);
    }
}

/// Inicializa serial
pub fn init() {
    SERIAL.lock().init();
}

/// Escreve byte
pub fn write_byte(byte: u8) {
    SERIAL.lock().write_byte(byte);
}

/// Escreve string por referência (fixando a assinatura usada anteriormente em klog)
pub fn write(s: &str) {
    write_str(s);
}

/// Escreve string
pub fn write_str(s: &str) {
    let serial = SERIAL.lock();
    for byte in s.bytes() {
        serial.write_byte(byte);
    }
}

/// Escreve número hexadecimal
pub fn write_hex(value: u64) {
    let serial = SERIAL.lock();
    
    // Escrever dígitos
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
