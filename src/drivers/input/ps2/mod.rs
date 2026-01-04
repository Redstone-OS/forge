//! # PS/2 Controller Driver (8042)
//!
//! Implementação dos drivers para o **controlador PS/2 8042** - o
//! controlador legado de teclado e mouse em PCs x86.
//!
//! ## Arquitetura:
//! ```text
//! +------------------+
//! |   IRQ 1 (KB)     |
//! +--------+---------+
//!          |
//! +--------v---------+
//! |   8042 Controller|
//! |   Port 0x60/0x64 |
//! +--------+---------+
//!          |
//! +--------v---------+
//! |   IRQ 12 (Mouse) |
//! +------------------+
//! ```
//!
//! ## Dispositivos:
//! - **Porta 1**: Teclado (IRQ 1)
//! - **Porta 2**: Mouse (IRQ 12)
//!
//! ## Portas I/O:
//! - **0x60**: Data port (leitura/escrita)
//! - **0x64**: Command/Status port
//!
//! ## Integração RDS:
//! Este driver implementa a trait `Driver` e pode ser registrado
//! no DriverManager.

pub mod io; // Operações I/O baixo nível
pub mod keyboard; // Driver de teclado
pub mod mouse; // Driver de mouse
pub mod ports; // Constantes de portas

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::sync::Spinlock;
use alloc::sync::Arc;

// Re-exports para compatibilidade
pub use keyboard::{handle_irq as keyboard_irq, pop_scancode, read_scancode};
pub use mouse::{
    get_state as mouse_get_state, handle_irq as mouse_irq, read_packet, set_resolution,
    MousePacket, MouseState,
};

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// DRIVER PS/2
// =============================================================================

/// Driver PS/2 para RDS.
pub struct Ps2Driver;

impl Driver for Ps2Driver {
    fn name(&self) -> &'static str {
        "PS/2 Legacy Controller (8042)"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Input
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(PS/2) Inicializando controlador 8042...");

        // Limpa buffer inicial
        io::flush_output();

        // Testa controlador
        if !io::self_test() {
            crate::kerror!("(PS/2) Falha no self-test do 8042");
            return Err(DriverError::HardwareError);
        }

        // Inicializa teclado (porta 1)
        keyboard::init();
        crate::kinfo!("(PS/2) Teclado inicializado (IRQ 1)");

        // Inicializa mouse (porta 2)
        if mouse::init() {
            crate::kinfo!("(PS/2) Mouse inicializado (IRQ 12)");
        } else {
            crate::kwarn!("(PS/2) Mouse não detectado");
        }

        dev.set_state(DeviceState::Running);
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.set_state(DeviceState::Disconnected);
        Ok(())
    }

    fn suspend(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // PS/2 não suporta suspend real
        Ok(())
    }

    fn resume(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // Re-inicializa após resume
        io::flush_output();
        keyboard::init();
        mouse::init();
        Ok(())
    }
}

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o subsistema PS/2.
pub fn init() {
    crate::kinfo!("(PS/2) Inicializando...");

    // Limpa buffer
    io::flush_output();

    // Inicializa dispositivos
    keyboard::init();
    mouse::init();

    // Registra driver no RDS
    crate::drivers::base::register_driver(Arc::new(Ps2Driver) as Arc<dyn Driver>);

    *INITIALIZED.lock() = true;

    crate::kinfo!("(PS/2) Controlador inicializado");
}

/// Desliga o subsistema PS/2.
pub fn shutdown() {
    crate::kinfo!("(PS/2) Shutdown");
    // Desabilita interrupções
    io::disable_interrupts();
}

/// Inicializa apenas o teclado (legacy API).
pub fn init_keyboard() {
    io::flush_output();
    keyboard::init();
}

/// Inicializa apenas o mouse (legacy API).
pub fn init_mouse() {
    mouse::init();
}

/// Verifica se PS/2 está inicializado.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock()
}
