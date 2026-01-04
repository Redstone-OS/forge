//! # Timestamp Counter (TSC)
//!
//! Driver para leitura do contador de ciclos da CPU.

/// Lê o TSC nativo da CPU.
pub fn read_tsc() -> u64 {
    // STUB: Usar instrução rdtsc
    // unsafe { core::arch::x86_64::_rdtsc() }
    0
}

/// Calibra o TSC usando o PIT ou HPET para determinar a frequência da CPU.
pub fn calibrate() {
    // TODO:
    // 1. Ler T1 do TSC
    // 2. Esperar 10ms usando PIT/HPET
    // 3. Ler T2 do TSC
    // 4. Calcular delta e estimar frequência
    crate::kinfo!("(System/TSC) Calibração do contador de ciclos (stub).");
}
