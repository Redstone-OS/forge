//! Driver de Porta Serial (COM1).
//!
//! Usado como saída primária de logs para debug (host/QEMU).

use crate::arch::platform::ports::Port;
use crate::sync::Mutex;
use core::fmt;

const COM1_PORT: u16 = 0x3F8;

/// Driver Serial protegido por Mutex global.
pub static SERIAL1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(COM1_PORT) });

/// Estrutura do Driver Serial (sem lock interno).
pub struct SerialPort {
    data: Port<u8>,
    int_en: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts: Port<u8>,
}

impl SerialPort {
    /// Cria uma nova instância da porta serial.
    ///
    /// # Safety
    /// Caller deve garantir que o endereço base é válido.
    pub const unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_en: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_sts: Port::new(base + 5),
        }
    }

    /// Inicializa a porta serial UART 16550.
    pub fn init(&mut self) {
        unsafe {
            self.int_en.write(0x00); // Disable interrupts
            self.line_ctrl.write(0x80); // Enable DLAB (set baud rate divisor)
            self.data.write(0x03); // Set divisor to 3 (lo byte) 38400 baud
            self.int_en.write(0x00); //                  (hi byte)
            self.line_ctrl.write(0x03); // 8 bits, no parity, one stop bit
            self.fifo_ctrl.write(0xC7); // Enable FIFO, clear them, with 14-byte threshold
            self.modem_ctrl.write(0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    /// Envia um byte pela serial.
    pub fn send(&mut self, data: u8) {
        unsafe {
            // Espera o buffer de transmissão estar vazio (Bit 5 do Line Status)
            while (self.line_sts.read() & 0x20) == 0 {}
            self.data.write(data);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
