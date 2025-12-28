//! Programmable Interval Timer (8254)

use crate::arch::x86_64::ports::outb;

/// Frequência base do PIT (Hz)
const PIT_FREQUENCY: u32 = 1193182;

/// Portas do PIT
const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

/// Inicializa PIT para frequência específica
pub fn init(frequency_hz: u32) {
    let divisor = PIT_FREQUENCY / frequency_hz;

    // Channel 0, lobyte/hibyte, mode 3 (square wave)
    outb(PIT_COMMAND, 0x36);

    // Divisor
    outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);

    crate::kinfo!("(PIT) Inicializado com freq=", frequency_hz as u64);
}
