//! # Clock Syscalls
//!
//! clock_get, sleep

use crate::syscall::abi::types::{ClockId, TimeSpec};
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_clock_get_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_clock_get(args.arg1 as u32, args.arg2)
}

pub fn sys_sleep_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_sleep(args.arg1 as u64)
}

pub fn sys_timer_create_wrapper(_args: &SyscallArgs) -> SysResult<usize> {
    sys_timer_create()
}

pub fn sys_timer_set_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_timer_set(args.arg1 as u32, args.arg2 as u64, args.arg3 as u64)
}

// === IMPLEMENTAÇÕES ===

/// Obtém tempo do sistema
///
/// # Args
/// - clock_id: tipo de clock (REALTIME, MONOTONIC, etc)
/// - out_ptr: ponteiro para TimeSpec de saída
///
/// # Returns
/// 0 ou erro
pub fn sys_clock_get(clock_id: u32, out_ptr: usize) -> SysResult<usize> {
    // Validar clock_id
    let clock = match clock_id {
        0 => ClockId::Realtime,
        1 => ClockId::Monotonic,
        2 => ClockId::ProcessCpu,
        3 => ClockId::ThreadCpu,
        _ => return Err(SysError::InvalidArgument),
    };

    // Obter tempo
    let time = match clock {
        ClockId::Monotonic => {
            let ticks = crate::drivers::timer::ticks();
            let freq = crate::drivers::timer::frequency() as u64;
            if freq == 0 {
                TimeSpec::zero()
            } else {
                let seconds = ticks / freq;
                let remaining_ticks = ticks % freq;
                let nanoseconds = (remaining_ticks * 1_000_000_000) / freq;
                TimeSpec {
                    seconds,
                    nanoseconds: nanoseconds as u32,
                    _pad: 0,
                }
            }
        }
        _ => {
            // TODO: Implementar outros clocks
            TimeSpec::zero()
        }
    };

    // Escrever para userspace
    // TODO: Validar ponteiro e usar copy_to_user
    if out_ptr != 0 {
        unsafe {
            let out = out_ptr as *mut TimeSpec;
            *out = time;
        }
    }

    Ok(0)
}

/// Dorme por N milissegundos
///
/// # Args
/// - ms: milissegundos a dormir
///
/// # Returns
/// Ms restantes (se interrompido) ou 0
pub fn sys_sleep(ms: u64) -> SysResult<usize> {
    if ms == 0 {
        return Ok(0);
    }

    // Usar o scheduler para colocar a task em estado dormente
    crate::sched::core::sleep_current(ms);

    Ok(0)
}

/// Cria um timer do sistema
pub fn sys_timer_create() -> SysResult<usize> {
    crate::kwarn!("(Syscall) sys_timer_create não implementado");
    Err(SysError::NotImplemented)
}

/// Configura/Inicia um timer
pub fn sys_timer_set(handle: u32, initial_ms: u64, interval_ms: u64) -> SysResult<usize> {
    let _ = (handle, initial_ms, interval_ms);
    crate::kwarn!("(Syscall) sys_timer_set não implementado");
    Err(SysError::NotImplemented)
}
