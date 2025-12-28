//! PS/2 Mouse Driver

use crate::arch::x86_64::ports::{inb, outb};

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const CMD_PORT: u16 = 0x64;

pub fn init() {
    unsafe {
        // Habilitar mouse auxiliar
        wait_write();
        outb(CMD_PORT, 0xA8);

        // Habilitar interrupções (IRQ 12) - Compaq Status Byte
        wait_write();
        outb(CMD_PORT, 0x20); // Ler byte de comando
        wait_read();
        let status = inb(DATA_PORT) | 2; // Bit 1: Enable IRQ 12
        wait_write();
        outb(CMD_PORT, 0x60); // Escrever byte de comando
        wait_write();
        outb(DATA_PORT, status);

        // Set defaults
        write_mouse(0xF6);
        
        // Enable streaming
        write_mouse(0xF4);
    }
    crate::kinfo!("(Input) Mouse PS/2 inicializado");
}

unsafe fn wait_write() {
    // Esperar buffer de entrada estar vazio (bit 1 deve ser 0)
    while (inb(STATUS_PORT) & 0x02) != 0 {
        core::hint::spin_loop();
    }
}

unsafe fn wait_read() {
    // Esperar buffer de saída ter dados (bit 0 deve ser 1)
    while (inb(STATUS_PORT) & 0x01) == 0 {
        core::hint::spin_loop();
    }
}

unsafe fn write_mouse(byte: u8) {
    wait_write();
    outb(CMD_PORT, 0xD4); // Próximo byte vai para mouse
    wait_write();
    outb(DATA_PORT, byte);
    // Mouse envia ACK (0xFA), devemos ler
    wait_read();
    let _ack = inb(DATA_PORT);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MousePacket {
    pub flags: u8,
    pub x: i8,
    pub y: i8,
}

pub fn read_packet() -> Option<MousePacket> {
    unsafe {
        let status = inb(STATUS_PORT);
        if (status & 0x01) == 0 || (status & 0x20) == 0 {
            return None; // Sem dados ou dados não são de mouse
        }
        
        let flags = inb(DATA_PORT);
        wait_read();
        let x = inb(DATA_PORT) as i8;
        wait_read();
        let y = inb(DATA_PORT) as i8;
        
        Some(MousePacket { flags, x, y })
    }
}
