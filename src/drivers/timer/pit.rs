//! Timer PIT (Programmable Interval Timer)
//!
//! Driver para o 8253/8254 PIT.
//! Gera interrupções periódicas (timer ticks).

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_FREQUENCY: u32 = 1193182; // Hz

/// Contador de ticks
static TICKS: AtomicU64 = AtomicU64::new(0);

/// Escreve byte em porta I/O
#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
}

/// Inicializa o PIT com frequência especificada
pub fn init(frequency: u32) {
    let divisor = (PIT_FREQUENCY / frequency) as u16;

    unsafe {
        // Configurar PIT: Channel 0, rate generator, 16-bit
        outb(PIT_COMMAND, 0x36);

        // Enviar divisor (low byte, high byte)
        outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);
    }
}

/// Handler de interrupção do timer (chamado pelo assembly)
pub extern "C" fn timer_handler() {
    // Incrementar contador
    TICKS.fetch_add(1, Ordering::Relaxed);

    // Enviar EOI ao PIC
    crate::drivers::pic::send_eoi(0);
}

/// Obtém número de ticks desde o boot
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
