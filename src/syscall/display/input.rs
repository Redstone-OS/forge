//! # Input Syscalls
//!
//! Syscalls para acesso ao teclado e mouse.

use crate::drivers::input::{keyboard, mouse};
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// ============================================================================
// SYS_MOUSE_READ (0x48)
// ============================================================================

/// Estado do mouse para userspace.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct UserMouseState {
    pub x: i32,
    pub y: i32,
    pub delta_x: i32,
    pub delta_y: i32,
    pub buttons: u8,
    pub _pad: [u8; 3],
}

/// Lê estado atual do mouse.
pub fn sys_mouse_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut UserMouseState;

    if out_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    let state = mouse::get_state();

    let user_state = UserMouseState {
        x: state.x,
        y: state.y,
        delta_x: state.delta_x,
        delta_y: state.delta_y,
        buttons: state.buttons,
        _pad: [0; 3],
    };

    unsafe {
        core::ptr::write_volatile(out_ptr, user_state);
    }

    Ok(0)
}

// ============================================================================
// SYS_KEYBOARD_READ (0x49)
// ============================================================================

/// Evento de tecla para userspace.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct UserKeyEvent {
    pub scancode: u8,
    pub pressed: bool,
    pub _pad: [u8; 6],
}

/// Lê eventos de teclado.
/// Retorna número de eventos lidos.
pub fn sys_keyboard_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut UserKeyEvent;
    let max_events = args.arg2;

    if out_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    if max_events == 0 {
        return Ok(0);
    }

    let mut count = 0;
    let max = max_events.min(32); // Limitar para evitar overflow

    // TODO: Remover após debug
    if max > 0 {
        crate::kdebug!("(Syscall) Keyboard read request. Max:", max as u64);
    }

    for i in 0..max {
        if let Some(scancode) = keyboard::pop_scancode() {
            // Bit 7 = 1 significa key released (0x80)
            let pressed = (scancode & 0x80) == 0;
            let actual_scancode = scancode & 0x7F;

            unsafe {
                let user_event = UserKeyEvent {
                    scancode: actual_scancode,
                    pressed,
                    _pad: [0; 6],
                };
                core::ptr::write_volatile(out_ptr.add(i), user_event);
            }
            count += 1;
        } else {
            break;
        }
    }

    Ok(count)
}
