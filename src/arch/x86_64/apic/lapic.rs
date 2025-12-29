#![allow(dead_code)]
/// Arquivo: x86_64/apic/lapic.rs
///
/// Propósito: Driver para o Local APIC (LAPIC).
/// Cada core da CPU possui seu próprio LAPIC.
/// Funções principais:
/// - Receber interrupções do I/O APIC.
/// - Gerar interrupções locais (Timer, Thermal, Performance).
/// - Gerar IPIs (Inter-Processor Interrupts) para comunicar com outros cores.
/// - Enviar sinal de End of Interrupt (EOI).
///
/// Detalhes de Implementação:
/// - Usa MSR `IA32_APIC_BASE` para habilitar globalmente.
/// - Usa MMIO (padrão 0xFEE00000) para acesso aos registradores de controle.
/// - Configura Spurious Interrupt Vector (SVR) para habilitar recepção de interrupções.

/// Controlador Local APIC
use crate::arch::x86_64::cpu::Cpu;

// --- Registradores e Constantes ---
const IA32_APIC_BASE_MSR: u32 = 0x1B;
const LAPIC_BASE_ADDR: u64 = 0xFEE00000; // Endereço físico padrão

// Offsets MMIO
const REG_ID: usize = 0x020;
const REG_VER: usize = 0x030;
const REG_EOI: usize = 0x0B0;
const REG_SVR: usize = 0x0F0; // Spurious Interrupt Vector
const REG_ESR: usize = 0x280; // Error Status Register
const REG_LVT_TIMER: usize = 0x320;
const REG_TICR: usize = 0x380; // Timer Initial Count
const REG_TCCR: usize = 0x390; // Timer Current Count
const REG_TDCR: usize = 0x3E0; // Timer Divide Config

// Bits e Flags
const APIC_ENABLE_BIT: u64 = 1 << 11; // MSR Enable
const SVR_SOFT_ENABLE: u32 = 1 << 8; // Software Enable no registro SVR

/// Inicializa o Local APIC do core atual.
///
/// # Safety
///
/// - Deve ser chamado em Ring 0.
/// - Assuma que endereço 0xFEE00000 está mapeado (identity map ou similar) nas page tables.
/// - Não é thread-safe se chamado concorrentemente no mesmo core (o que não deve acontecer).
pub unsafe fn init() {
    // 1. Habilitar LAPIC globalmente via MSR
    let msr_info = Cpu::read_msr(IA32_APIC_BASE_MSR);
    if (msr_info & APIC_ENABLE_BIT) == 0 {
        Cpu::write_msr(IA32_APIC_BASE_MSR, msr_info | APIC_ENABLE_BIT);
    }

    // 2. Definir Spurious Interrupt Vector e Habilitar Software (Bit 8)
    // Vetor 0xFF (255) geralmente usado para Spurious
    write(REG_SVR, SVR_SOFT_ENABLE | 0xFF);

    // 3. Mascarar LVT Timer inicialmente (até configurarmos o timer)
    // Bit 16 = Masked
    write(REG_LVT_TIMER, 1 << 16);

    // 4. Limpar Error Status Register (precisa escrever 2x em hardware antigo, 1x em novos)
    write(REG_ESR, 0);
    write(REG_ESR, 0);

    // 5. Sinalizar EOI para limpar estado pendente anterior (sanity check)
    write(REG_EOI, 0);
}

/// Envia o sinal de End of Interrupt (EOI) para o LAPIC.
///
/// Deve ser chamado ao final de todo handler de interrupção externa (exceto exceções e NMI).
#[inline]
pub unsafe fn eoi() {
    write(REG_EOI, 0);
}

/// Lê o ID do LAPIC atual (Core ID físico).
///
/// O ID está nos bits 24-31 do registrador ID.
#[inline]
pub fn id() -> u32 {
    unsafe { read(REG_ID) >> 24 }
}

// --- Helpers de Acesso MMIO (Privados) ---

#[inline]
unsafe fn read(offset: usize) -> u32 {
    let ptr = (LAPIC_BASE_ADDR as *const u32).add(offset / 4);
    core::ptr::read_volatile(ptr)
}

#[inline]
unsafe fn write(offset: usize, value: u32) {
    let ptr = (LAPIC_BASE_ADDR as *mut u32).add(offset / 4);
    core::ptr::write_volatile(ptr, value);
}
