//! Driver do PIT (Programmable Interval Timer).
//!
//! Configura o chip 8254 para gerar interrupções na frequência desejada.
//! Fundamental para o Scheduler preemptivo.

use crate::arch::x86_64::ports::Port;
use crate::sync::Mutex;

const CHANNEL0: u16 = 0x40;
const COMMAND: u16 = 0x43;
const FREQUENCY: u32 = 1_193_182; // Frequência base do cristal (Hz)

pub struct Pit {
    channel0: Port<u8>,
    command: Port<u8>,
}

impl Pit {
    pub const unsafe fn new() -> Self {
        Self {
            channel0: Port::new(CHANNEL0),
            command: Port::new(COMMAND),
        }
    }

    /// Configura a frequência do Timer (em Hz).
    /// Retorna a frequência real configurada.
    pub fn set_frequency(&mut self, freq: u32) -> u32 {
        let divisor = FREQUENCY / freq;
        let actual_freq = FREQUENCY / divisor;

        unsafe {
            // Modo: Channel 0, Access lo/hi byte, Rate Generator (Mode 2), Binary
            self.command.write(0x36);

            // Enviar divisor (low byte, high byte)
            self.channel0.write((divisor & 0xFF) as u8);
            self.channel0.write((divisor >> 8) as u8);
        }

        actual_freq
    }
}

pub static PIT: Mutex<Pit> = Mutex::new(unsafe { Pit::new() });

/// Handler de alto nível chamado pela interrupção.
/// Incrementa ticks e (futuramente) chama o scheduler.
pub fn on_interrupt() {
    // Apenas um log visual simples para provar que está vivo
    // crate::kprint!(".");

    // TODO: Incrementar contador atômico de ticks
    // TODO: Chamar scheduler::tick()

    unsafe {
        crate::drivers::pic::PICS.lock().notify_eoi(32); // 32 = IRQ0 (Timer)
    }
}
