//! PS/2 Keyboard Driver

use crate::arch::x86_64::ports::{inb, outb};

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

pub fn init() {
    // TODO: Configurar controlador PS/2 se necessário
    // Por enquanto assumimos que o BIOS iniciou o básico
    crate::kinfo!("(Input) Teclado PS/2 inicializado");
}

/// Lê scancode do teclado (polling, para debug/boot)
pub fn read_scancode() -> Option<u8> {
    unsafe {
        let status = inb(STATUS_PORT);
        if (status & 0x01) != 0 {
            Some(inb(DATA_PORT))
        } else {
            None
        }
    }
}
