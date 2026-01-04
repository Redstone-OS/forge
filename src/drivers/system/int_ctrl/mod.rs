//! # Interrupt Controller Manager
//!
//! Gerencia controladores de interrupção (PIC, APIC, IO-APIC).
//! Abstrai diferenças entre modo legacy e APIC.
//!
//! ## Hierarquia:
//! - **PIC (8259)**: Legacy, 15 IRQs (IRQ2 = cascade)
//! - **Local APIC**: Um por CPU, timers e IPIs
//! - **IO-APIC**: Roteamento de IRQs externas

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::system::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;

/// Modo de interrupção ativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptMode {
    /// PIC legado (8259).
    Pic,
    /// APIC (Local + IO-APIC).
    Apic,
    /// x2APIC (APIC estendido).
    X2Apic,
}

/// Estado global.
static INT_STATE: Spinlock<IntState> = Spinlock::new(IntState::new());

struct IntState {
    mode: InterruptMode,
    initialized: bool,
}

impl IntState {
    const fn new() -> Self {
        Self {
            mode: InterruptMode::Pic,
            initialized: false,
        }
    }
}

/// Driver do gerenciador de interrupções.
pub struct IntCtrlDriver;

impl Driver for IntCtrlDriver {
    fn name(&self) -> &'static str {
        "int-ctrl"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(IntCtrl) Detectando controladores de interrupção...");

        // TODO: Detecção real:
        // 1. Verificar CPUID para suporte a APIC
        // 2. Verificar MSR para APIC base address
        // 3. Parsear MADT/ACPI para IO-APICs
        // 4. Decidir modo (PIC vs APIC)

        crate::kinfo!("(IntCtrl) Usando modo PIC (legacy)");
        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }
}

/// Inicializa o subsistema de interrupções.
pub fn init() {
    crate::kinfo!("(IntCtrl) Inicializando controladores...");
    crate::drivers::base::register_driver(Arc::new(IntCtrlDriver) as Arc<dyn Driver>);

    let mut state = INT_STATE.lock();
    state.initialized = true;

    crate::kinfo!("(IntCtrl) Controladores prontos");
}

/// Retorna modo de interrupção atual.
pub fn current_mode() -> InterruptMode {
    INT_STATE.lock().mode
}

/// Habilita uma IRQ específica.
pub fn enable_irq(irq: u8) {
    match current_mode() {
        InterruptMode::Pic => pic_unmask(irq),
        InterruptMode::Apic | InterruptMode::X2Apic => ioapic_unmask(irq),
    }
}

/// Desabilita uma IRQ específica.
pub fn disable_irq(irq: u8) {
    match current_mode() {
        InterruptMode::Pic => pic_mask(irq),
        InterruptMode::Apic | InterruptMode::X2Apic => ioapic_mask(irq),
    }
}

/// Envia EOI (End of Interrupt).
pub fn send_eoi(irq: u8) {
    match current_mode() {
        InterruptMode::Pic => pic_eoi(irq),
        InterruptMode::Apic | InterruptMode::X2Apic => lapic_eoi(),
    }
}

// =============================================================================
// PIC (8259) Functions
// =============================================================================

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

fn pic_mask(irq: u8) {
    let (port, bit) = if irq < 8 {
        (PIC1_DATA, irq)
    } else {
        (PIC2_DATA, irq - 8)
    };

    let mask = crate::arch::x86_64::ports::inb(port);
    crate::arch::x86_64::ports::outb(port, mask | (1 << bit));
}

fn pic_unmask(irq: u8) {
    let (port, bit) = if irq < 8 {
        (PIC1_DATA, irq)
    } else {
        (PIC2_DATA, irq - 8)
    };

    let mask = crate::arch::x86_64::ports::inb(port);
    crate::arch::x86_64::ports::outb(port, mask & !(1 << bit));
}

fn pic_eoi(irq: u8) {
    use crate::arch::x86_64::ports::outb;

    if irq >= 8 {
        outb(PIC2_CMD, 0x20); // EOI para slave
    }
    outb(PIC1_CMD, 0x20); // EOI para master
}

// =============================================================================
// APIC Functions (stubs)
// =============================================================================

fn ioapic_mask(_irq: u8) {
    // TODO: Escrever no IO-APIC redirection table
}

fn ioapic_unmask(_irq: u8) {
    // TODO: Escrever no IO-APIC redirection table
}

fn lapic_eoi() {
    // TODO: Escrever no Local APIC EOI register
}
