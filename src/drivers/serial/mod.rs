//! Driver de porta serial simplificado (QEMU Debug Port)
//!
//! Escreve diretamente na porta 0xE9 (Hack do Bochs/QEMU) para debug.
//! Não requer inicialização de UART, baud rate, etc.

use crate::arch::x86_64::ports::outb;
use crate::sync::Spinlock;

/// Porta de debug do QEMU/Bochs
const QEMU_PORT: u16 = 0xE9;

/// Serial "fake" que apenas escreve no QEMU
pub struct SerialPort;

static SERIAL: Spinlock<SerialPort> = Spinlock::new(SerialPort);

impl SerialPort {
    /// Inicialização "fake" (não necessária para 0xE9)
    pub fn init(&self) {
        // Nada a fazer para porta 0xE9
    }

    /// Escreve byte diretamente
    pub fn write_byte(&self, byte: u8) {
        outb(QEMU_PORT, byte);
    }
}

/// Inicializa serial (No-op para QEMU port, mantido para compatibilidade de API)
pub fn init() {
    SERIAL.lock().init();
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
    let serial = SERIAL.lock();
    for byte in s.bytes() {
        serial.write_byte(byte);
    }
}

/// Escreve número hexadecimal
pub fn write_hex(value: u64) {
    let serial = SERIAL.lock();
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
