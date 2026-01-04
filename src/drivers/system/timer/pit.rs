//! # Programmable Interval Timer (8254/8253)
//!
//! Driver para o timer legado PIT. Presente em todo PC compatível.
//! Usado como fallback quando HPET não está disponível.
//!
//! ## Canais:
//! - Channel 0: System timer (IRQ 0)
//! - Channel 1: DRAM refresh (obsoleto)
//! - Channel 2: PC Speaker

use crate::arch::x86_64::ports::outb;

/// Frequência base do PIT (Hz).
const PIT_BASE_FREQ: u32 = 1193182;

/// Portas I/O do PIT.
const PIT_CHANNEL0: u16 = 0x40;
const PIT_CHANNEL2: u16 = 0x42;
const PIT_COMMAND: u16 = 0x43;

/// Modos do PIT.
#[allow(dead_code)]
mod modes {
    pub const MODE0_INTERRUPT: u8 = 0b000 << 1; // Interrupt on terminal count
    pub const MODE1_ONESHOT: u8 = 0b001 << 1; // Hardware retriggerable one-shot
    pub const MODE2_RATE: u8 = 0b010 << 1; // Rate generator
    pub const MODE3_SQUARE: u8 = 0b011 << 1; // Square wave generator
    pub const MODE4_STROBE: u8 = 0b100 << 1; // Software triggered strobe
    pub const MODE5_STROBE_HW: u8 = 0b101 << 1; // Hardware triggered strobe
}

/// Inicializa PIT Channel 0 para frequência específica.
///
/// ## Parâmetros:
/// - `frequency_hz`: Frequência desejada em Hz (tipicamente 100 ou 1000)
pub fn init(frequency_hz: u32) {
    let divisor = if frequency_hz == 0 {
        65536 // Frequência mínima (~18.2 Hz)
    } else {
        PIT_BASE_FREQ / frequency_hz
    };

    // Channel 0, Access mode lobyte/hibyte, Mode 3 (square wave)
    let command: u8 = 0b00_11_011_0; // Channel 0, lo/hi, mode 3, binary
    outb(PIT_COMMAND, command);

    // Envia divisor (low byte primeiro, depois high byte)
    outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);

    // Atualiza frequência global
    super::set_frequency(frequency_hz as u64);

    // NOTA: IRQ0 é habilitado separadamente após o scheduler estar pronto
    // NÃO habilitar aqui para evitar interrupções durante a inicialização

    crate::kinfo!(
        "(PIT) Inicializado: {}Hz (divisor={})",
        frequency_hz,
        divisor
    );
}

/// Lê o contador atual do Channel 0.
///
/// NOTA: Esta operação requer latch command e pode não ser precisa.
pub fn read_count() -> u16 {
    // Latch command para Channel 0
    outb(PIT_COMMAND, 0b00_00_00_00);

    // Lê low byte, depois high byte
    let low = unsafe { crate::arch::x86_64::ports::inb(PIT_CHANNEL0) };
    let high = unsafe { crate::arch::x86_64::ports::inb(PIT_CHANNEL0) };

    ((high as u16) << 8) | (low as u16)
}

/// Configura Channel 2 para o PC Speaker.
pub fn configure_speaker(frequency_hz: u32) {
    if frequency_hz == 0 {
        return;
    }

    let divisor = PIT_BASE_FREQ / frequency_hz;

    // Channel 2, Access mode lobyte/hibyte, Mode 3 (square wave)
    let command: u8 = 0b10_11_011_0; // Channel 2, lo/hi, mode 3, binary
    outb(PIT_COMMAND, command);

    outb(PIT_CHANNEL2, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL2, ((divisor >> 8) & 0xFF) as u8);
}
