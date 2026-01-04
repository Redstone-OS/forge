//! # Power Control Driver
//!
//! Gerencia estados de energia do sistema (Shutdown, Reboot, Sleep).
//!
//! ## Métodos de Reset:
//! 1. **ACPI**: Via FADT Reset Register
//! 2. **Keyboard**: Via 8042 controller (0x64 <- 0xFE)
//! 3. **Triple Fault**: Forçar exceção não recuperável
//!
//! ## Métodos de Shutdown:
//! 1. **ACPI S5**: Via PM1a/PM1b Control registers

use crate::drivers::base::device::Device;
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::system::traits::*;
use alloc::sync::Arc;

/// Driver de controle de energia.
pub struct PowerControlDriver;

impl Driver for PowerControlDriver {
    fn name(&self) -> &'static str {
        "pwr-ctrl"
    }
    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(PwrCtrl) Controle de energia ativo");
        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }
}

/// Inicializa o subsistema de energia.
pub fn init() {
    crate::kinfo!("(PwrCtrl) Inicializando controle de energia...");
    crate::drivers::base::register_driver(Arc::new(PowerControlDriver) as Arc<dyn Driver>);
}

/// Reinicia o sistema.
///
/// Tenta múltiplos métodos em ordem de preferência.
pub fn reboot() {
    crate::kwarn!("(PwrCtrl) Reiniciando sistema...");

    // Método 1: ACPI Reset (se disponível)
    if !try_acpi_reset() {
        // Método 2: Keyboard controller (8042)
        if !try_keyboard_reset() {
            // Método 3: Triple Fault
            triple_fault();
        }
    }
}

/// Desliga o sistema.
///
/// Requer suporte ACPI para funcionar corretamente.
pub fn shutdown() {
    crate::kwarn!("(PwrCtrl) Desligando sistema...");

    // Método 1: ACPI S5
    if !try_acpi_shutdown() {
        // Método 2: QEMU debug exit (se em VM)
        try_qemu_exit();

        // Se chegou aqui, não conseguiu desligar
        crate::kerror!("(PwrCtrl) Falha ao desligar! Halt manual.");
        halt_forever();
    }
}

/// Entra em modo sleep (S3).
pub fn suspend() {
    crate::kinfo!("(PwrCtrl) Suspendendo sistema...");
    // TODO: ACPI S3
    crate::kwarn!("(PwrCtrl) Suspend não implementado");
}

/// Hibernate (S4).
pub fn hibernate() {
    crate::kinfo!("(PwrCtrl) Hibernando sistema...");
    // TODO: ACPI S4
    crate::kwarn!("(PwrCtrl) Hibernate não implementado");
}

// =============================================================================
// Métodos de Reset
// =============================================================================

fn try_acpi_reset() -> bool {
    // TODO: Usar FADT Reset Register
    false
}

fn try_keyboard_reset() -> bool {
    use crate::arch::x86_64::ports::outb;

    crate::kinfo!("(PwrCtrl) Tentando reset via teclado...");

    // Esperar buffer vazio
    for _ in 0..10000 {
        let status = crate::arch::x86_64::ports::inb(0x64);
        if (status & 0x02) == 0 {
            break;
        }
    }

    // Enviar comando de reset
    outb(0x64, 0xFE);

    // Se chegou aqui, não funcionou
    false
}

fn triple_fault() -> ! {
    crate::kinfo!("(PwrCtrl) Forçando triple fault...");

    // Carregar IDT inválida
    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &[0u64; 2],
            options(nostack, readonly)
        );

        // Causar interrupção
        core::arch::asm!("int 3", options(nostack));
    }

    unreachable!()
}

// =============================================================================
// Métodos de Shutdown
// =============================================================================

fn try_acpi_shutdown() -> bool {
    // TODO: ACPI S5 via PM1a/PM1b Control
    false
}

fn try_qemu_exit() {
    use crate::arch::x86_64::ports::outb;

    // QEMU debug exit port (se configurado com -device isa-debug-exit)
    outb(0x501, 0x31);
}

fn halt_forever() -> ! {
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nostack));
        }
    }
}
