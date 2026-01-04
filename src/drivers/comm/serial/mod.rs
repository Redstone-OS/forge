//! # Serial Port Driver (UART 16550)
//!
//! Driver para portas seriais **COM1-COM4** usando o chip UART 16550.
//!
//! ## ⚠️ DRIVER CRÍTICO
//! Este é um dos drivers mais importantes do sistema:
//! - Inicializado **ANTES** do DriverManager
//! - Inicializado **ANTES** do heap (usa constantes estáticas)
//! - Usado para **DEBUG** via QEMU/hardware real
//! - Canal principal de **LOG** durante o boot
//!
//! ## Arquitetura:
//! ```text
//! ┌──────────────┐
//! │   kinfo!()   │  Macros de log
//! │   kerror!()  │
//! └──────┬───────┘
//!        │
//! ┌──────v───────┐
//! │ Serial Port  │  Este driver
//! │ Buffer 16KB  │
//! └──────┬───────┘
//!        │
//! ┌──────v───────┐
//! │ UART 16550   │  Hardware
//! │ Port 0x3F8   │
//! └──────────────┘
//! ```
//!
//! ## Portas Standard:
//! - **COM1**: 0x3F8 (IRQ 4) - Usado para debug
//! - **COM2**: 0x2F8 (IRQ 3)
//! - **COM3**: 0x3E8 (IRQ 4)
//! - **COM4**: 0x2E8 (IRQ 3)
//!
//! ## Registradores (offset do base):
//! | Offset | DLAB=0 Read    | DLAB=0 Write   | DLAB=1        |
//! |--------|----------------|----------------|---------------|
//! | +0     | RBR (Receive)  | THR (Transmit) | Divisor Low   |
//! | +1     | IER (Int En)   | IER            | Divisor High  |
//! | +2     | IIR (Int ID)   | FCR (FIFO Ctl) | FCR           |
//! | +3     | LCR (Line Ctl) | LCR            | LCR           |
//! | +4     | MCR (Modem Ctl)| MCR            | MCR           |
//! | +5     | LSR (Line Sts) | -              | LSR           |
//! | +6     | MSR (Modem Sts)| -              | MSR           |
//! | +7     | SCR (Scratch)  | SCR            | SCR           |
//!
//! ## Design Choices:
//! 1. **Buffer Circular**: 16KB para absorver picos de log
//! 2. **Fast Path**: Se buffer vazio e HW pronto, escreve direto
//! 3. **Greedy Drain**: Descarrega até 128 bytes por operação
//! 4. **Minimal Dependencies**: Apenas `arch::ports` e `sync::Spinlock`
//!
//! ## Considerações de Localização:
//! Este driver poderia também estar em `core::debug` por ser tão
//! fundamental. A localização atual em `drivers::comm` mantém
//! consistência arquitetural, mas a integração com o log é íntima.

use crate::arch::x86_64::ports::{inb, outb};
use crate::sync::Spinlock;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Endereço base da porta COM1.
pub const COM1_PORT: u16 = 0x3F8;

/// Endereço base da porta COM2.
pub const COM2_PORT: u16 = 0x2F8;

/// Endereço base da porta COM3.
pub const COM3_PORT: u16 = 0x3E8;

/// Endereço base da porta COM4.
pub const COM4_PORT: u16 = 0x2E8;

// Offsets dos registradores
const REG_DATA: u16 = 0; // RBR/THR/Divisor Low
const REG_INT_ENABLE: u16 = 1; // IER/Divisor High
const REG_FIFO_CTRL: u16 = 2; // IIR/FCR
const REG_LINE_CTRL: u16 = 3; // LCR
const REG_MODEM_CTRL: u16 = 4; // MCR
const REG_LINE_STATUS: u16 = 5; // LSR
#[allow(unused)] // TODO: Revisar no futuro
const REG_MODEM_STATUS: u16 = 6; // MSR
#[allow(unused)] // TODO: Revisar no futuro
const REG_SCRATCH: u16 = 7; // Scratch Register

// Bits do Line Status Register (LSR)
const LSR_DATA_READY: u8 = 0x01; // Dados disponíveis para leitura
#[allow(unused)] // TODO: Revisar no futuro
const LSR_OVERRUN_ERR: u8 = 0x02; // Overrun error
#[allow(unused)] // TODO: Revisar no futuro
const LSR_PARITY_ERR: u8 = 0x04; // Parity error
#[allow(unused)] // TODO: Revisar no futuro
const LSR_FRAMING_ERR: u8 = 0x08; // Framing error
#[allow(unused)] // TODO: Revisar no futuro
const LSR_BREAK_IND: u8 = 0x10; // Break indicator
#[allow(unused)] // TODO: Revisar no futuro
const LSR_TX_EMPTY: u8 = 0x20; // THR empty
#[allow(unused)] // TODO: Revisar no futuro
const LSR_TX_IDLE: u8 = 0x40; // THR empty & line idle
#[allow(unused)] // TODO: Revisar no futuro
const LSR_FIFO_ERR: u8 = 0x80; // Error in received FIFO

// Configuração
const SERIAL_BUFFER_SIZE: usize = 16 * 1024; // 16KB
const SERIAL_BUFFER_MASK: usize = SERIAL_BUFFER_SIZE - 1;
const MAX_DRAIN_BYTES: usize = 128; // Máximo de bytes por drain

// =============================================================================
// ESTRUTURA DA PORTA SERIAL
// =============================================================================

/// Porta serial com buffer circular.
///
/// Usa buffer interno para absorver picos de log e evitar
/// perda de dados quando o hardware está ocupado.
pub struct SerialPort {
    /// Buffer circular para transmissão.
    buffer: [u8; SERIAL_BUFFER_SIZE],
    /// Índice de escrita (produtor).
    head: usize,
    /// Índice de leitura (consumidor).
    tail: usize,
    /// Contador de bytes descartados (overflow).
    dropped_count: usize,
    /// Porta base.
    port: u16,
    /// Porta está inicializada?
    initialized: bool,
}

impl SerialPort {
    /// Cria nova porta serial (não inicializada).
    const fn new(port: u16) -> Self {
        Self {
            buffer: [0; SERIAL_BUFFER_SIZE],
            head: 0,
            tail: 0,
            dropped_count: 0,
            port,
            initialized: false,
        }
    }

    /// Inicializa a porta serial.
    ///
    /// Configura:
    /// - 115200 baud
    /// - 8 data bits
    /// - No parity
    /// - 1 stop bit
    /// - Hardware FIFOs enabled
    pub fn init(&mut self) {
        let port = self.port;

        // 1. Desabilita interrupções
        outb(port + REG_INT_ENABLE, 0x00);

        // 2. Habilita DLAB para setar baud rate
        outb(port + REG_LINE_CTRL, 0x80);

        // 3. Seta divisor para 115200 baud (divisor = 1)
        outb(port + REG_DATA, 0x01); // Divisor low
        outb(port + REG_INT_ENABLE, 0x00); // Divisor high

        // 4. Configura formato: 8N1 (8 bits, no parity, 1 stop), desabilita DLAB
        outb(port + REG_LINE_CTRL, 0x03);

        // 5. Habilita FIFOs, limpa buffers, threshold de 14 bytes
        outb(port + REG_FIFO_CTRL, 0xC7);

        // 6. Habilita RTS/DSR, IRQs no modem control
        outb(port + REG_MODEM_CTRL, 0x0B);

        // 7. Marca como inicializado
        self.initialized = true;
    }

    /// Verifica se o transmitter está pronto para novos dados.
    #[inline]
    fn is_transmit_ready(&self) -> bool {
        inb(self.port + REG_LINE_STATUS) & LSR_TX_EMPTY != 0
    }

    /// Verifica se há dados para receber.
    #[inline]
    fn is_data_ready(&self) -> bool {
        inb(self.port + REG_LINE_STATUS) & LSR_DATA_READY != 0
    }

    /// Escreve um byte no buffer (requer lock já adquirido).
    ///
    /// ## Fast Path:
    /// Se buffer vazio E hardware pronto, escreve diretamente.
    /// Isso garante que logs apareçam imediatamente no QEMU.
    fn write_byte_internal(&mut self, byte: u8) {
        // Fast path: buffer vazio e hardware pronto
        if self.head == self.tail && self.is_transmit_ready() {
            outb(self.port + REG_DATA, byte);
            return;
        }

        // Slow path: adiciona ao buffer
        let next_head = (self.head + 1) & SERIAL_BUFFER_MASK;

        // Se buffer cheio, descarta byte mais antigo
        if next_head == self.tail {
            self.tail = (self.tail + 1) & SERIAL_BUFFER_MASK;
            self.dropped_count += 1;
        }

        self.buffer[self.head] = byte;
        self.head = next_head;

        // Tenta descarregar o que puder
        self.drain_greedy();
    }

    /// Descarrega o máximo possível do buffer para o hardware.
    ///
    /// Limitado a MAX_DRAIN_BYTES para não prender a CPU
    /// em hardware lento, mas agressivo o suficiente para QEMU.
    fn drain_greedy(&mut self) {
        let mut count = 0;

        while self.head != self.tail && self.is_transmit_ready() && count < MAX_DRAIN_BYTES {
            outb(self.port + REG_DATA, self.buffer[self.tail]);
            self.tail = (self.tail + 1) & SERIAL_BUFFER_MASK;
            count += 1;
        }
    }

    /// Força descarga completa do buffer (BLOQUEANTE).
    ///
    /// Use apenas em situações críticas como panic.
    pub fn force_flush(&mut self) {
        while self.head != self.tail {
            // Espera hardware ficar pronto
            while !self.is_transmit_ready() {
                core::hint::spin_loop();
            }
            outb(self.port + REG_DATA, self.buffer[self.tail]);
            self.tail = (self.tail + 1) & SERIAL_BUFFER_MASK;
        }
    }

    /// Escreve valor hexadecimal (16 dígitos).
    fn write_hex_internal(&mut self, value: u64) {
        const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

        for i in (0..16).rev() {
            let digit = ((value >> (i * 4)) & 0xF) as usize;
            self.write_byte_internal(HEX_CHARS[digit]);
        }
    }

    /// Lê um byte da porta (polling).
    pub fn read_byte(&self) -> Option<u8> {
        if self.is_data_ready() {
            Some(inb(self.port + REG_DATA))
        } else {
            None
        }
    }

    /// Retorna número de bytes descartados por overflow.
    pub fn dropped_count(&self) -> usize {
        self.dropped_count
    }

    /// Retorna bytes pendentes no buffer.
    pub fn pending_bytes(&self) -> usize {
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            SERIAL_BUFFER_SIZE - self.tail + self.head
        }
    }
}

// =============================================================================
// ESTADO GLOBAL (COM1)
// =============================================================================

/// Porta serial primária (COM1) - usada para debug.
static SERIAL: Spinlock<SerialPort> = Spinlock::new(SerialPort::new(COM1_PORT));

// =============================================================================
// API PÚBLICA
// =============================================================================

/// Inicializa a porta serial COM1.
///
/// ## ⚠️ DEVE SER CHAMADA CEDO
/// Esta função deve ser uma das primeiras chamadas no boot,
/// antes mesmo do heap estar disponível.
pub fn init() {
    SERIAL.lock().init();
}

/// Escreve uma string na porta serial.
pub fn write_str(s: &str) {
    let mut serial = SERIAL.lock();
    for byte in s.bytes() {
        serial.write_byte_internal(byte);
    }
}

/// Escreve um byte na porta serial.
pub fn write_byte(byte: u8) {
    SERIAL.lock().write_byte_internal(byte);
}

/// Alias para write_byte.
pub fn emit(byte: u8) {
    write_byte(byte);
}

/// Escreve valor hexadecimal com prefixo "0x".
pub fn write_hex(value: u64) {
    let mut serial = SERIAL.lock();
    serial.write_byte_internal(b'0');
    serial.write_byte_internal(b'x');
    serial.write_hex_internal(value);
}

/// Escreve uma linha de log formatada.
///
/// Formato: `{prefix}{msg} 0x{value}\n`
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

/// Tenta descarregar o buffer (não-bloqueante).
pub fn try_drain() {
    SERIAL.lock().drain_greedy();
}

/// Força descarga completa do buffer (BLOQUEANTE).
///
/// Use apenas em panic ou shutdown.
pub fn force_flush() {
    SERIAL.lock().force_flush();
}

/// Lê um byte da porta serial (polling).
pub fn read_byte() -> Option<u8> {
    SERIAL.lock().read_byte()
}

/// Retorna número de bytes descartados.
pub fn dropped_count() -> usize {
    SERIAL.lock().dropped_count()
}

/// Retorna bytes pendentes no buffer.
pub fn pending_bytes() -> usize {
    SERIAL.lock().pending_bytes()
}

/// Verifica se a porta está inicializada.
pub fn is_initialized() -> bool {
    SERIAL.lock().initialized
}

// =============================================================================
// TRAIT IMPLEMENTATIONS
// =============================================================================

use core::fmt::{self, Write};

/// Writer para integração com core::fmt.
pub struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

/// Retorna um writer para uso com write!/writeln!.
pub fn writer() -> SerialWriter {
    SerialWriter
}
