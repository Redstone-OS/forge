//! # Syscalls de Input (Mouse/Teclado)
//!
//! Permite que userspace leia eventos de mouse e teclado.

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// ============================================================================
// TIPOS COMPARTILHADOS (ABI estável)
// ============================================================================

/// Estado do mouse para userspace
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MouseStateABI {
    /// Posição X absoluta
    pub x: i32,
    /// Posição Y absoluta
    pub y: i32,
    /// Movimento delta X
    pub delta_x: i32,
    /// Movimento delta Y
    pub delta_y: i32,
    /// Botões
    pub buttons: u8,
    /// Padding
    pub _pad: [u8; 3],
}

/// Evento de teclado para userspace
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct KeyEventABI {
    /// Scancode
    pub scancode: u8,
    /// Pressionado?
    pub pressed: bool,
    /// Padding
    pub _pad: [u8; 2],
}

// ============================================================================
// IMPLEMENTAÇÕES DE SYSCALLS
// ============================================================================

/// SYS_MOUSE_READ: Lê estado do mouse
pub fn sys_mouse_read(out_ptr: *mut MouseStateABI) -> SysResult<usize> {
    if out_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    // Obter estado do driver
    let driver_state = crate::drivers::input::mouse::get_state();

    let user_state = MouseStateABI {
        x: driver_state.x,
        y: driver_state.y,
        delta_x: driver_state.delta_x,
        delta_y: driver_state.delta_y,
        buttons: driver_state.buttons,
        _pad: [0; 3],
    };

    // Copiar para userspace
    unsafe {
        core::ptr::write_volatile(out_ptr, user_state);
    }

    Ok(0)
}

/// SYS_KEYBOARD_READ: Lê eventos de teclado pendentes
pub fn sys_keyboard_read(out_ptr: *mut KeyEventABI, max_events: usize) -> SysResult<usize> {
    if out_ptr.is_null() || max_events == 0 {
        return Err(SysError::BadAddress);
    }

    let mut events_read = 0;

    while events_read < max_events {
        if let Some(scancode) = crate::drivers::input::keyboard::pop_scancode() {
            let pressed = (scancode & 0x80) == 0;
            let code = scancode & 0x7F;

            let event = KeyEventABI {
                scancode: code,
                pressed,
                _pad: [0; 2],
            };

            unsafe {
                core::ptr::write_volatile(out_ptr.add(events_read), event);
            }
            events_read += 1;
        } else {
            break;
        }
    }

    Ok(events_read)
}

// ============================================================================
// WRAPPERS PARA DISPATCH TABLE
// ============================================================================

pub fn sys_mouse_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut MouseStateABI;
    sys_mouse_read(out_ptr)
}

pub fn sys_keyboard_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut KeyEventABI;
    let max_events = args.arg2;
    sys_keyboard_read(out_ptr, max_events)
}

/// Configura limites do mouse (chamado pelo kernel/compositor via syscall ou internal)
/// Por enquanto mantemos internal wrapper se necessário
pub fn set_mouse_bounds(width: i32, height: i32) {
    crate::drivers::input::mouse::set_resolution(width, height);
}
