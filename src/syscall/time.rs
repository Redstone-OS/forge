//! # Time Management Syscalls
//!
//! Controle de tempo e relógios.
//!
//! ## 🎯 Propósito
//! - **Timekeeping:** Saber "que horas são" (`clock_get`).
//! - **Timer:** Esperar "N milissegundos" (`sleep`).
//!
//! ## 🏗️ Arquitetura
//! - **Tick-Based:** Baseado em interrupção de timer (lapic/pit) rodando a ~100Hz (10ms).
//! - **Monotonicity:** `CLOCK_MONOTONIC` nunca volta no tempo, ideal para medir duração.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Busy Wait:** `sys_sleep` atualmente faz um loop `yield`. Isso gasta CPU inutilmente (polling). Precisa de fila de espera com *deadline*.
//! - **Low Resolution:** 10ms é muito lento para multimídia ou jogos. O sistema precisa de High Resolution Timers (HPET/TSC).
//!
//! ## 🛠️ TODOs
//! - [ ] **TODO: (Performance)** Implementar **Timer Queue** (Wheel ou Heap) para acordar threads apenas quando necessário (Tickless Kernel).
//! - [ ] **TODO: (Hardware)** Implementar driver **HPET/APIC Timer** calibravel para precisão de µs.
//!
use super::abi::{ClockId, TimeSpec};
use super::error::{SysError, SysResult};

/// Ticks desde o boot (contador global simples).
static mut BOOT_TICKS: u64 = 0;

/// Incrementa contador de ticks (chamado pelo timer).
pub fn increment_ticks() {
    unsafe {
        BOOT_TICKS += 1;
    }
}

/// Obtém ticks atuais.
pub fn get_ticks() -> u64 {
    unsafe { BOOT_TICKS }
}

/// Obtém tempo do sistema.
///
/// # Syscall
/// `SYS_CLOCK_GET (0x50)` - Args: (clock_id, out_ptr)
///
/// # Argumentos
/// - `clock_id`: Tipo de relógio (ClockId)
/// - `out_ptr`: Ponteiro para TimeSpec
///
/// # Clocks Suportados
/// - Monotonic: Ticks desde boot
/// - Realtime: (não implementado ainda)
pub fn sys_clock_get(clock_id: usize, out_ptr: usize) -> SysResult<usize> {
    if out_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    // Validar alinhamento
    if out_ptr % core::mem::align_of::<TimeSpec>() != 0 {
        return Err(SysError::BadAlignment);
    }

    let clock = match clock_id as u32 {
        0 => ClockId::Realtime,
        1 => ClockId::Monotonic,
        2 => ClockId::ProcessCpu,
        3 => ClockId::ThreadCpu,
        _ => return Err(SysError::InvalidArgument),
    };

    // Calcular tempo
    let timespec = match clock {
        ClockId::Monotonic => {
            // Assumindo timer de 100Hz (10ms por tick)
            let ticks = get_ticks();
            let ms = ticks * 10;
            TimeSpec::from_millis(ms)
        }
        ClockId::Realtime => {
            // TODO: Implementar RTC
            crate::kwarn!("[Syscall] clock_get: REALTIME não implementado");
            TimeSpec::zero()
        }
        _ => {
            crate::kwarn!("[Syscall] clock_get: clock {:?} não implementado", clock);
            TimeSpec::zero()
        }
    };

    // Escrever resultado
    // TODO: Validar que ponteiro pertence ao userspace
    unsafe {
        let ptr = out_ptr as *mut TimeSpec;
        *ptr = timespec;
    }

    Ok(0)
}

/// Dorme por N milissegundos.
///
/// # Syscall
/// `SYS_SLEEP (0x51)` - Args: (ms)
///
/// # Retorno
/// Milissegundos restantes se interrompido, 0 se completou
pub fn sys_sleep(ms: usize) -> SysResult<usize> {
    if ms == 0 {
        return Ok(0);
    }

    // TODO: Implementar sleep real com timer queue
    // Por enquanto: busy-wait com yields

    let start = get_ticks();
    let ticks_to_wait = (ms as u64 + 9) / 10; // Arredondar para cima

    loop {
        let elapsed = get_ticks() - start;
        if elapsed >= ticks_to_wait {
            break;
        }

        // Yield para não travar CPU
        let _ = super::process::sys_yield();
    }

    Ok(0)
}

/// Obtém tempo monotônico em ticks.
///
/// # Syscall
/// `SYS_MONOTONIC (0x52)` - Args: nenhum
///
/// # Retorno
/// Ticks desde boot
pub fn sys_monotonic() -> SysResult<usize> {
    Ok(get_ticks() as usize)
}
