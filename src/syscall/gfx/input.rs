//! # Syscalls de Input (Mouse/Teclado)
//!
//! Permite que userspace leia eventos de mouse e teclado.

use crate::sync::Spinlock;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// ============================================================================
// TIPOS COMPARTILHADOS (ABI estável)
// ============================================================================

/// Estado do mouse para userspace
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MouseState {
    /// Posição X absoluta
    pub x: i32,
    /// Posição Y absoluta
    pub y: i32,
    /// Movimento delta X desde última leitura
    pub delta_x: i32,
    /// Movimento delta Y desde última leitura
    pub delta_y: i32,
    /// Botões pressionados (bit 0=esquerdo, 1=direito, 2=meio)
    pub buttons: u8,
    /// Padding para alinhamento
    pub _pad: [u8; 3],
}

/// Evento de teclado para userspace
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct KeyEvent {
    /// Scancode da tecla
    pub scancode: u8,
    /// Tecla pressionada (true) ou solta (false)
    pub pressed: bool,
    /// Padding
    pub _pad: [u8; 2],
}

// ============================================================================
// ESTADO GLOBAL DO MOUSE
// ============================================================================

/// Estado atual do mouse (gerenciado pelo kernel)
static MOUSE_STATE: Spinlock<MouseStateInternal> = Spinlock::new(MouseStateInternal::new());

struct MouseStateInternal {
    x: i32,
    y: i32,
    delta_x: i32,
    delta_y: i32,
    buttons: u8,
    screen_width: i32,
    screen_height: i32,
}

impl MouseStateInternal {
    const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            delta_x: 0,
            delta_y: 0,
            buttons: 0,
            screen_width: 800, // Default
            screen_height: 600,
        }
    }
}

/// Configura limites do mouse (chamado pelo kernel após init do framebuffer)
pub fn set_mouse_bounds(width: i32, height: i32) {
    let mut state = MOUSE_STATE.lock();
    state.screen_width = width;
    state.screen_height = height;
    // Centralizar cursor
    state.x = width / 2;
    state.y = height / 2;
}

/// Processa pacote de mouse (chamado pelo handler de IRQ)
pub fn process_mouse_packet(dx: i8, dy: i8, buttons: u8) {
    let mut state = MOUSE_STATE.lock();

    // Acumular deltas
    state.delta_x += dx as i32;
    state.delta_y -= dy as i32; // Y invertido no PS/2

    // Atualizar posição absoluta com clamping
    state.x = (state.x + dx as i32).clamp(0, state.screen_width - 1);
    state.y = (state.y - dy as i32).clamp(0, state.screen_height - 1);

    // Atualizar botões
    state.buttons = buttons & 0x07; // Apenas 3 botões
}

// ============================================================================
// IMPLEMENTAÇÕES DE SYSCALLS
// ============================================================================

/// SYS_MOUSE_READ: Lê estado do mouse
///
/// Args:
///   - arg1: ponteiro para MouseState (userspace)
///
/// Retorna: 0 em sucesso
pub fn sys_mouse_read(out_ptr: *mut MouseState) -> SysResult<usize> {
    if out_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    let mut internal = MOUSE_STATE.lock();

    let user_state = MouseState {
        x: internal.x,
        y: internal.y,
        delta_x: internal.delta_x,
        delta_y: internal.delta_y,
        buttons: internal.buttons,
        _pad: [0; 3],
    };

    // Resetar deltas após leitura
    internal.delta_x = 0;
    internal.delta_y = 0;

    // Copiar para userspace
    unsafe {
        core::ptr::write_volatile(out_ptr, user_state);
    }

    Ok(0)
}

/// SYS_KEYBOARD_READ: Lê eventos de teclado pendentes
///
/// Args:
///   - arg1: ponteiro para array de KeyEvent (userspace)
///   - arg2: tamanho máximo do array
///
/// Retorna: número de eventos lidos
pub fn sys_keyboard_read(out_ptr: *mut KeyEvent, max_events: usize) -> SysResult<usize> {
    if out_ptr.is_null() || max_events == 0 {
        return Err(SysError::BadAddress);
    }

    // Por enquanto, polling simples do driver de teclado
    let mut events_read = 0;

    while events_read < max_events {
        if let Some(scancode) = crate::drivers::input::keyboard::read_scancode() {
            let pressed = (scancode & 0x80) == 0;
            let code = scancode & 0x7F;

            let event = KeyEvent {
                scancode: code,
                pressed,
                _pad: [0; 2],
            };

            unsafe {
                core::ptr::write_volatile(out_ptr.add(events_read), event);
            }
            events_read += 1;
        } else {
            break; // Sem mais eventos
        }
    }

    Ok(events_read)
}

// ============================================================================
// WRAPPERS PARA DISPATCH TABLE
// ============================================================================

pub fn sys_mouse_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut MouseState;
    sys_mouse_read(out_ptr)
}

pub fn sys_keyboard_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut KeyEvent;
    let max_events = args.arg2;
    sys_keyboard_read(out_ptr, max_events)
}
