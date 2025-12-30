//! PS/2 Mouse Driver (Interrupt Based)

use crate::arch::x86_64::ports::{inb, outb};
use crate::sync::Spinlock;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const CMD_PORT: u16 = 0x64;

// Expose MouseState for Syscall usage
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub delta_x: i32,
    pub delta_y: i32,
    pub buttons: u8,
    pub screen_width: i32,
    pub screen_height: i32,
}

static MOUSE_STATE: Spinlock<MouseState> = Spinlock::new(MouseState {
    x: 0,
    y: 0,
    delta_x: 0,
    delta_y: 0,
    buttons: 0,
    screen_width: 800,
    screen_height: 600,
});

static MOUSE_CYCLE: Spinlock<u8> = Spinlock::new(0);
static MOUSE_BYTES: Spinlock<[u8; 3]> = Spinlock::new([0; 3]);

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
    crate::kinfo!("(Input) PS/2 Mouse Initialized (Interrupt Mode)");
}

/// Handler de Interrupção (Chamado pelo IRQ 12)
pub fn handle_irq() {
    let status = inb(STATUS_PORT);
    if (status & 0x20) == 0 {
        return; // Not mouse
    }

    let data = inb(DATA_PORT);

    // Precisamos de lock para atomicidade na máquina de estados
    let mut cycle = MOUSE_CYCLE.lock();
    let mut bytes = MOUSE_BYTES.lock(); // Talvez separar o lock seja ruim?
                                        // Não, está OK, mas cuidado com inversion order se outro lugar travar.
                                        // Aqui só IRQ usa. (E talvez init se resetar).

    match *cycle {
        0 => {
            // Byte 1: Flags
            // Bit 3 deve ser 1. Se não for, desincronizou.
            if (data & 0x08) == 0x08 {
                bytes[0] = data;
                *cycle = 1;
            }
        }
        1 => {
            // Byte 2: X
            bytes[1] = data;
            *cycle = 2;
        }
        2 => {
            // Byte 3: Y
            bytes[2] = data;
            *cycle = 0;

            // Process packet
            process_packet(bytes[0], bytes[1], bytes[2]);
        }
        _ => {
            *cycle = 0;
        }
    }
}

fn process_packet(flags: u8, x_byte: u8, y_byte: u8) {
    let mut state = MOUSE_STATE.lock();

    // Calcular deltas
    // X e Y são 9 bits signed. O 9o bit está nas flags.
    // Mas geralmente modo padrão é 8 bits signed e overflow bit.
    // Vamos assumir modo padrão PS/2 3-byte packet.

    let mut dx = x_byte as i16;
    let mut dy = y_byte as i16;

    // Sign extension bits in flags
    if (flags & 0x10) != 0 {
        dx |= 0xFF00u16 as i16;
    } // X sign bit
    if (flags & 0x20) != 0 {
        dy |= 0xFF00u16 as i16;
    } // Y sign bit

    let dx_i32 = dx as i32;
    let dy_i32 = dy as i32;

    // Y é invertido (pra cima é positivo no mouse, negativo na tela geralmente?)
    // PS/2: Y aumenta pra cima. Tela: Y aumenta pra baixo. Então dy deve ser subtraído.

    state.delta_x += dx_i32;
    state.delta_y -= dy_i32;

    // Update buttons
    state.buttons = flags & 0x07;

    // Update absolute pos
    state.x = (state.x + dx_i32).clamp(0, state.screen_width - 1);
    state.y = (state.y - dy_i32).clamp(0, state.screen_height - 1);
}

pub fn get_state() -> MouseState {
    let mut state = MOUSE_STATE.lock();
    let current = *state;
    // Reset deltas after read
    state.delta_x = 0;
    state.delta_y = 0;
    current
}

pub fn set_resolution(width: i32, height: i32) {
    let mut state = MOUSE_STATE.lock();
    state.screen_width = width;
    state.screen_height = height;
    state.x = width / 2;
    state.y = height / 2;
}

// Helpers
unsafe fn wait_write() {
    while (inb(STATUS_PORT) & 0x02) != 0 {
        core::hint::spin_loop();
    }
}

unsafe fn wait_read() {
    while (inb(STATUS_PORT) & 0x01) == 0 {
        core::hint::spin_loop();
    }
}

unsafe fn write_mouse(byte: u8) {
    wait_write();
    outb(CMD_PORT, 0xD4);
    wait_write();
    outb(DATA_PORT, byte);
    wait_read();
    let _ack = inb(DATA_PORT);
}

// Legacy compat
pub struct MousePacket {
    pub flags: u8,
    pub x: i8,
    pub y: i8,
}
pub fn read_packet() -> Option<MousePacket> {
    None // Agora é interrupt only
}
