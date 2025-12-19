//! Driver Serial (COM1/COM2)
//!
//! Driver básico para portas seriais 16550 UART.
//! Usado para debug e output do kernel.

use core::arch::asm;
use core::fmt;

/// Porta serial COM1 (0x3F8)
const COM1: u16 = 0x3F8;

/// Driver da porta serial
pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    /// Cria uma nova porta serial
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    /// Inicializa a porta serial
    pub fn init(&mut self) {
        unsafe {
            // Desabilita interrupções
            outb(self.port + 1, 0x00);
            // Habilita DLAB (set baud rate divisor)
            outb(self.port + 3, 0x80);
            // Set divisor to 3 (38400 baud)
            outb(self.port + 0, 0x03);
            outb(self.port + 1, 0x00);
            // 8 bits, no parity, one stop bit
            outb(self.port + 3, 0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 2, 0xC7);
            // IRQs enabled, RTS/DSR set
            outb(self.port + 4, 0x0B);
        }
    }

    /// Escreve um byte na porta serial
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            // Aguarda a porta estar pronta
            while (inb(self.port + 5) & 0x20) == 0 {}
            outb(self.port, byte);
        }
    }

    /// Escreve uma string na porta serial
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

/// Lê um byte de uma porta I/O
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

/// Escreve um byte em uma porta I/O
#[inline]
unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Porta serial global para output do kernel
static mut SERIAL: SerialPort = SerialPort::new(COM1);

/// Inicializa a porta serial global
pub fn init() {
    unsafe {
        (*core::ptr::addr_of_mut!(SERIAL)).init();
    }
}

/// Escreve na porta serial global
pub fn write_str(s: &str) {
    unsafe {
        (*core::ptr::addr_of_mut!(SERIAL)).write_str(s);
    }
}

/// Escreve um byte na porta serial
pub fn write_byte(byte: u8) {
    unsafe {
        (*core::ptr::addr_of_mut!(SERIAL)).write_byte(byte);
    }
}

/// Escreve formatado na porta serial
pub fn print(s: &str) {
    write_str(s);
}

/// Escreve formatado com newline na porta serial
pub fn println(s: &str) {
    write_str(s);
    write_str("\n");
}

/// Imprime um número em hexadecimal
pub fn print_hex(mut n: usize) {
    if n == 0 {
        write_str("0");
        return;
    }

    let mut buf = [0u8; 16];
    let mut i = 0;

    while n > 0 {
        let d = n % 16;
        buf[i] = if d < 10 {
            b'0' + d as u8
        } else {
            b'a' + (d - 10) as u8
        };
        n /= 16;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        write_byte(buf[i]);
    }
}
