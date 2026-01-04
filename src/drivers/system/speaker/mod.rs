//! # PC Speaker Driver
//!
//! Driver para o alto-falante do sistema (beep).
//! Usa PIT Channel 2 para gerar tons.

use crate::arch::x86_64::ports::{inb, outb};
use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

/// Porta de controle do speaker/PIT.
const SPEAKER_PORT: u16 = 0x61;

/// Driver do PC Speaker.
pub struct SpeakerDriver;

impl Driver for SpeakerDriver {
    fn name(&self) -> &'static str {
        "pc-speaker"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(Speaker) PC Speaker registrado");
        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        stop();
        Ok(())
    }
}

/// Inicializa o driver do speaker.
pub fn init() {
    crate::drivers::base::register_driver(Arc::new(SpeakerDriver) as Arc<dyn Driver>);
}

/// Toca um som na frequência especificada.
///
/// ## Parâmetros:
/// - `frequency_hz`: Frequência em Hz (ex: 440 = Lá central)
pub fn play(frequency_hz: u32) {
    if frequency_hz == 0 {
        stop();
        return;
    }

    // Configura PIT Channel 2 para a frequência
    super::timer::pit::configure_speaker(frequency_hz);

    // Habilita speaker (bits 0 e 1 da porta 0x61)
    let current = { inb(SPEAKER_PORT) };
    outb(SPEAKER_PORT, current | 0x03);
}

/// Para o som.
pub fn stop() {
    // Desabilita speaker (bits 0 e 1 da porta 0x61)
    let current = { inb(SPEAKER_PORT) };
    outb(SPEAKER_PORT, current & !0x03);
}

/// Toca um beep por duração especificada.
///
/// ## Parâmetros:
/// - `frequency_hz`: Frequência em Hz
/// - `duration_ms`: Duração em milissegundos
pub fn beep(frequency_hz: u32, duration_ms: u64) {
    play(frequency_hz);
    super::timer::delay_ms(duration_ms);
    stop();
}

/// Toca beep padrão do sistema (1000Hz, 100ms).
pub fn system_beep() {
    beep(1000, 100);
}

/// Toca sequência de beeps para indicar erro.
pub fn error_beep() {
    for _ in 0..3 {
        beep(800, 200);
        super::timer::delay_ms(100);
    }
}

/// Toca melodia simples de boot.
pub fn boot_melody() {
    // C4, E4, G4
    let notes = [(262, 100), (330, 100), (392, 200)];

    for (freq, dur) in notes.iter() {
        beep(*freq, *dur as u64);
        super::timer::delay_ms(50);
    }
}
