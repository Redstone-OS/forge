//! # Local APIC Driver
//!
//! Driver para o Local APIC (LAPIC).
//! Cada core da CPU possui seu próprio LAPIC.
//!
//! ## Responsabilidades
//!
//! - Receber interrupções do I/O APIC
//! - Gerar interrupções locais (Timer, Thermal, Performance)
//! - Gerar IPIs (Inter-Processor Interrupts) para comunicar com outros cores
//! - Enviar sinal de End of Interrupt (EOI)
//! - Enviar INIT/SIPI para acordar APs
//!
//! ## Registradores
//!
//! ```text
//! LAPIC MMIO (0xFEE00000)
//! ┌────────┬───────────────────────────────────┐
//! │ Offset │ Registrador                       │
//! ├────────┼───────────────────────────────────┤
//! │ 0x020  │ ID - Local APIC ID                │
//! │ 0x030  │ VER - Version                     │
//! │ 0x0B0  │ EOI - End of Interrupt            │
//! │ 0x0F0  │ SVR - Spurious Vector Register    │
//! │ 0x300  │ ICR Low - Interrupt Command       │
//! │ 0x310  │ ICR High - Destination            │
//! │ 0x320  │ LVT Timer                         │
//! │ 0x380  │ Timer Initial Count               │
//! └────────┴───────────────────────────────────┘
//! ```

#![allow(dead_code)]

use crate::arch::x86_64::cpu::Cpu;

// =============================================================================
// Constantes e Registradores
// =============================================================================

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const LAPIC_BASE_ADDR: u64 = 0xFEE00000;

// Offsets MMIO
const REG_ID: usize = 0x020;
const REG_VER: usize = 0x030;
const REG_EOI: usize = 0x0B0;
const REG_SVR: usize = 0x0F0;
const REG_ESR: usize = 0x280;
const REG_ICR_LOW: usize = 0x300;
const REG_ICR_HIGH: usize = 0x310;
const REG_LVT_TIMER: usize = 0x320;
const REG_TICR: usize = 0x380;
const REG_TCCR: usize = 0x390;
const REG_TDCR: usize = 0x3E0;

// Bits e Flags
const APIC_ENABLE_BIT: u64 = 1 << 11;
const SVR_SOFT_ENABLE: u32 = 1 << 8;

// ICR Delivery Modes
const DELIVERY_FIXED: u32 = 0b000 << 8;
const DELIVERY_INIT: u32 = 0b101 << 8;
const DELIVERY_STARTUP: u32 = 0b110 << 8;

// ICR Level/Trigger
const LEVEL_ASSERT: u32 = 1 << 14;
const LEVEL_DEASSERT: u32 = 0 << 14;

// ICR Delivery Status
const DELIVERY_STATUS_PENDING: u32 = 1 << 12;

// =============================================================================
// Inicialização
// =============================================================================

/// Inicializa o Local APIC do core atual.
///
/// # Safety
///
/// - Deve ser chamado em Ring 0
/// - O endereço 0xFEE00000 deve estar mapeado nas page tables
pub unsafe fn init() {
    // 1. Habilitar LAPIC globalmente via MSR
    let msr_info = Cpu::read_msr(IA32_APIC_BASE_MSR);
    if (msr_info & APIC_ENABLE_BIT) == 0 {
        Cpu::write_msr(IA32_APIC_BASE_MSR, msr_info | APIC_ENABLE_BIT);
    }

    // 2. Definir Spurious Interrupt Vector e Habilitar Software
    write(REG_SVR, SVR_SOFT_ENABLE | 0xFF);

    // 3. Mascarar LVT Timer inicialmente
    write(REG_LVT_TIMER, 1 << 16);

    // 4. Limpar Error Status Register
    write(REG_ESR, 0);
    write(REG_ESR, 0);

    // 5. Sinalizar EOI para limpar estado pendente
    write(REG_EOI, 0);
}

// =============================================================================
// EOI e ID
// =============================================================================

/// Envia o sinal de End of Interrupt (EOI) para o LAPIC.
#[inline]
pub unsafe fn eoi() {
    write(REG_EOI, 0);
}

/// Lê o ID do LAPIC atual (APIC ID nos bits 24-31).
#[inline]
pub fn id() -> u32 {
    unsafe { read(REG_ID) >> 24 }
}

// =============================================================================
// IPIs para SMP Bringup
// =============================================================================

/// Envia um INIT IPI para o AP especificado.
///
/// Isso reseta o AP e o coloca em estado de espera por SIPI.
///
/// # Safety
///
/// - Apenas o BSP deve chamar esta função
/// - O APIC ID deve ser válido
pub unsafe fn send_init_ipi(apic_id: u32) {
    // Esperar ICR estar livre
    wait_icr_idle();

    // Configurar destino (APIC ID nos bits 24-31)
    write(REG_ICR_HIGH, apic_id << 24);

    // Enviar INIT IPI (Assert)
    write(REG_ICR_LOW, DELIVERY_INIT | LEVEL_ASSERT);

    // Pequeno delay
    spin_delay(1000);

    // Enviar INIT IPI (De-assert)
    wait_icr_idle();
    write(REG_ICR_LOW, DELIVERY_INIT | LEVEL_DEASSERT);

    wait_icr_idle();
}

/// Envia um Startup IPI (SIPI) para o AP especificado.
///
/// O vetor indica a página de 4KB onde o AP deve começar:
/// endereço_físico = vector * 0x1000
///
/// # Safety
///
/// - Apenas o BSP deve chamar
/// - O vetor deve apontar para código de inicialização válido
pub unsafe fn send_sipi(apic_id: u32, vector: u8) {
    // Esperar ICR estar livre
    wait_icr_idle();

    // Configurar destino
    write(REG_ICR_HIGH, apic_id << 24);

    // Enviar SIPI com vetor
    write(REG_ICR_LOW, DELIVERY_STARTUP | (vector as u32));

    wait_icr_idle();
}

/// Envia um IPI genérico (Fixed) para outro core.
///
/// # Safety
///
/// - O vetor deve corresponder a um handler instalado na IDT
pub unsafe fn send_ipi(apic_id: u32, vector: u8) {
    wait_icr_idle();
    write(REG_ICR_HIGH, apic_id << 24);
    write(REG_ICR_LOW, DELIVERY_FIXED | (vector as u32));
    wait_icr_idle();
}

// =============================================================================
// Helpers
// =============================================================================

/// Espera o ICR estar pronto para novo comando.
#[inline]
unsafe fn wait_icr_idle() {
    while (read(REG_ICR_LOW) & DELIVERY_STATUS_PENDING) != 0 {
        core::hint::spin_loop();
    }
}

/// Delay por busy-wait (aproximado).
#[inline]
fn spin_delay(iterations: u32) {
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

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
