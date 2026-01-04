//! # Timestamp Counter (TSC)
//!
//! Driver para leitura do contador de ciclos da CPU.
//! Timer de mais alta resolução disponível, mas precisa calibração.
//!
//! ## Características:
//! - Resolução de 1 ciclo de CPU (~0.3ns em CPU de 3GHz)
//! - Contador de 64 bits
//! - Pode não ser monotônico em CPUs antigas
//! - Frequência varia com P-states em CPUs antigas

// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::system::traits::*;
use crate::sync::Spinlock;

/// Estado do TSC.
static TSC_STATE: Spinlock<TscState> = Spinlock::new(TscState::new());

struct TscState {
    calibrated: bool,
    frequency_hz: u64,
    invariant: bool,
}

impl TscState {
    const fn new() -> Self {
        Self {
            calibrated: false,
            frequency_hz: 0,
            invariant: false,
        }
    }
}

/// Lê o TSC nativo da CPU.
#[inline(always)]
pub fn read_tsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let low: u32;
        let high: u32;
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nostack, nomem)
        );
        ((high as u64) << 32) | (low as u64)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        0
    }
}

/// Lê TSC serializado (mais preciso, mais lento).
#[inline(always)]
pub fn read_tsc_serialized() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let low: u32;
        let high: u32;
        core::arch::asm!(
            "rdtscp",
            out("eax") low,
            out("edx") high,
            lateout("ecx") _,
            options(nostack, nomem)
        );
        ((high as u64) << 32) | (low as u64)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        0
    }
}

/// Calibra o TSC usando o PIT ou HPET.
///
/// Deve ser chamado após o PIT estar inicializado.
pub fn calibrate() {
    crate::kinfo!("(TSC) Calibrando contador de ciclos...");

    // Verificar se TSC é invariante (não muda com P-states)
    let invariant = check_invariant_tsc();

    // TODO: Calibração real:
    // 1. Programar PIT para one-shot de 10ms
    // 2. Ler TSC antes
    // 3. Esperar PIT expirar
    // 4. Ler TSC depois
    // 5. Calcular frequência

    // Por enquanto, assumimos 3GHz como placeholder
    let estimated_freq = 3_000_000_000u64;

    let mut state = TSC_STATE.lock();
    state.frequency_hz = estimated_freq;
    state.invariant = invariant;
    state.calibrated = true;

    crate::kinfo!(
        "(TSC) Calibrado: ~{}MHz (invariante: {})",
        estimated_freq / 1_000_000,
        invariant
    );
}

/// Verifica se a CPU tem TSC invariante.
fn check_invariant_tsc() -> bool {
    // CPUID 0x80000007, EDX bit 8
    #[cfg(target_arch = "x86_64")]
    {
        let res = unsafe { core::arch::x86_64::__cpuid(0x80000000) };
        if res.eax >= 0x80000007 {
            let res = unsafe { core::arch::x86_64::__cpuid(0x80000007) };
            return (res.edx & (1 << 8)) != 0;
        }
    }
    false
}

/// Retorna frequência calibrada do TSC em Hz.
pub fn frequency() -> u64 {
    TSC_STATE.lock().frequency_hz
}

/// Verifica se o TSC está calibrado.
pub fn is_calibrated() -> bool {
    TSC_STATE.lock().calibrated
}

/// Verifica se o TSC é invariante.
pub fn is_invariant() -> bool {
    TSC_STATE.lock().invariant
}

/// Converte ciclos TSC para nanosegundos.
pub fn cycles_to_ns(cycles: u64) -> u64 {
    let freq = frequency();
    if freq == 0 {
        return 0;
    }
    (cycles * 1_000_000_000) / freq
}

/// Converte nanosegundos para ciclos TSC.
pub fn ns_to_cycles(ns: u64) -> u64 {
    let freq = frequency();
    (ns * freq) / 1_000_000_000
}
