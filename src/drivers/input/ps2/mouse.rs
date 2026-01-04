//! # Driver PS/2 Mouse
//!
//! Implementação do driver de mouse PS/2.

use super::io;
use super::ports::{commands, config};
use crate::sync::Spinlock;

/// Estado do mouse
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MouseState {
    /// Posição X absoluta
    pub x: i32,
    /// Posição Y absoluta
    pub y: i32,
    /// Delta X desde última leitura
    pub delta_x: i32,
    /// Delta Y desde última leitura
    pub delta_y: i32,
    /// Botões pressionados (bits 0-2)
    pub buttons: u8,
    /// Largura da tela
    pub screen_width: i32,
    /// Altura da tela
    pub screen_height: i32,
}

static MOUSE_STATE: Spinlock<MouseState> = Spinlock::new(MouseState {
    x: 0,
    y: 0,
    delta_x: 0,
    delta_y: 0,
    buttons: 0,
    screen_width: 1280,
    screen_height: 800,
});

/// Máquina de estados do pacote PS/2 (3 bytes)
static MOUSE_CYCLE: Spinlock<u8> = Spinlock::new(0);
static MOUSE_BYTES: Spinlock<[u8; 3]> = Spinlock::new([0; 3]);

/// Comandos do mouse
mod mouse_cmd {
    /// Set defaults
    pub const SET_DEFAULTS: u8 = 0xF6;
    /// Enable streaming mode
    pub const ENABLE_STREAM: u8 = 0xF4;
    /// Disable streaming mode
    #[allow(dead_code)]
    pub const DISABLE_STREAM: u8 = 0xF5;
    /// Set sample rate
    #[allow(dead_code)]
    pub const SET_SAMPLE_RATE: u8 = 0xF3;
    /// Get device ID
    #[allow(dead_code)]
    pub const GET_DEVICE_ID: u8 = 0xF2;
    /// Reset
    #[allow(dead_code)]
    pub const RESET: u8 = 0xFF;
}

/// Inicializa o mouse PS/2
pub fn init() {
    // 1. Habilitar porta auxiliar
    io::send_command(commands::ENABLE_AUX);

    // 2. Configurar para habilitar IRQ do mouse
    if let Some(cfg) = io::read_config() {
        let new_cfg = cfg | config::AUX_IRQ;
        io::write_config(new_cfg);
    }

    // 3. Configurar dispositivo mouse
    io::write_aux(mouse_cmd::SET_DEFAULTS);
    io::write_aux(mouse_cmd::ENABLE_STREAM);

    // 4. Habilitar IRQ 12 no PIC
    crate::arch::x86_64::interrupts::pic_enable_irq(12);

    crate::kinfo!("(Input) PS/2 Mouse initialized");
}

/// Handler de interrupção (IRQ 12)
pub fn handle_irq() {
    // Verificar se é realmente dados do mouse
    if !io::is_mouse_data() {
        return;
    }

    let data = io::read_data();

    let mut cycle = MOUSE_CYCLE.lock();
    let mut bytes = MOUSE_BYTES.lock();

    match *cycle {
        0 => {
            // Byte 1: Flags - bit 3 deve ser 1
            if (data & 0x08) == 0x08 {
                bytes[0] = data;
                *cycle = 1;
            }
        }
        1 => {
            // Byte 2: Delta X
            bytes[1] = data;
            *cycle = 2;
        }
        2 => {
            // Byte 3: Delta Y
            bytes[2] = data;
            *cycle = 0;
            process_packet(bytes[0], bytes[1], bytes[2]);
        }
        _ => {
            *cycle = 0;
        }
    }
}

fn process_packet(flags: u8, x_byte: u8, y_byte: u8) {
    let mut state = MOUSE_STATE.lock();

    // Calcular deltas com sign extension
    let mut dx = x_byte as i16;
    let mut dy = y_byte as i16;

    if (flags & 0x10) != 0 {
        dx |= 0xFF00u16 as i16;
    }
    if (flags & 0x20) != 0 {
        dy |= 0xFF00u16 as i16;
    }

    let dx_i32 = dx as i32;
    let dy_i32 = dy as i32;

    // Acumular deltas
    state.delta_x += dx_i32;
    state.delta_y -= dy_i32; // Y invertido

    // Atualizar botões
    state.buttons = flags & 0x07;

    // Atualizar posição absoluta
    state.x = (state.x + dx_i32).clamp(0, state.screen_width - 1);
    state.y = (state.y - dy_i32).clamp(0, state.screen_height - 1);
}

/// Obtém o estado atual do mouse e reseta deltas
pub fn get_state() -> MouseState {
    let mut state = MOUSE_STATE.lock();
    let current = *state;
    state.delta_x = 0;
    state.delta_y = 0;
    current
}

/// Define a resolução da tela
pub fn set_resolution(width: i32, height: i32) {
    let mut state = MOUSE_STATE.lock();
    state.screen_width = width;
    state.screen_height = height;
    state.x = width / 2;
    state.y = height / 2;
}

/// Pacote de mouse (compatibilidade)
pub struct MousePacket {
    pub flags: u8,
    pub x: i8,
    pub y: i8,
}

/// Read packet (legacy - agora é interrupt only)
pub fn read_packet() -> Option<MousePacket> {
    None
}
