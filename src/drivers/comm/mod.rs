//! # Communication Subsystem
//!
//! Este módulo agrupa drivers de **comunicação serial e byte-a-byte**
//! entre o processador e periféricos.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Aplicações                 │
//! ├─────────────────────────────────────────┤
//! │          Comm Subsystem                 │
//! ├────────┬────────┬────────┬──────────────┤
//! │ Serial │  I2C   │  SPI   │   Parallel   │
//! │ (UART) │        │        │   (LPT)      │
//! ├────────┴────────┴────────┴──────────────┤
//! │              Platform Bus               │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Drivers:
//!
//! ### Críticos (Fase 1):
//! - **Serial (UART)**: COM1-COM4, usado para debug e logging
//!   - ⚠️ **CRÍTICO**: Inicializado MUITO CEDO no boot
//!   - Usado pelo sistema de log antes de qualquer outro driver
//!
//! ### Secundários (Fases Posteriores):
//! - **I2C**: Barramento para sensores, EEPROMs, etc
//! - **SPI**: Barramento síncrono de alta velocidade
//! - **Parallel**: Interface legada (LPT)
//! - **VirtIO Console**: Serial paravirtualizado
//!
//! ## Notas de Inicialização:
//! O driver serial é especial - ele é inicializado pelo kernel ANTES
//! do DriverManager e antes de muitos outros subsistemas. Isso permite
//! que logs apareçam desde os primeiros momentos do boot.

pub mod i2c; // Inter-Integrated Circuit (stub)
pub mod parallel; // Porta paralela/LPT (stub)
pub mod serial; // UART - CRÍTICO (debug)
pub mod spi; // Serial Peripheral Interface (stub)
pub mod virtio; // VirtIO Console (stub)

// Re-exports do serial (mais usado)
pub use serial::{force_flush, init as init_serial, write_byte, write_hex, write_str};

use crate::sync::Spinlock;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa todos os drivers de comunicação.
///
/// ## NOTA:
/// O driver serial já deve ter sido inicializado pelo kernel antes
/// desta função ser chamada. Aqui apenas inicializamos os outros.
pub fn init() {
    crate::kinfo!("(Comm) Inicializando drivers de comunicação...");

    // Serial já foi inicializado early pelo kernel
    // serial::init(); // NÃO chamar aqui!

    // Inicializa outros drivers
    virtio::init();
    i2c::init();
    spi::init();
    parallel::init();

    *INITIALIZED.lock() = true;

    crate::kinfo!("(Comm) Subsistema inicializado");
}

/// Desliga drivers de comunicação.
pub fn shutdown() {
    crate::kinfo!("(Comm) Shutdown");

    parallel::shutdown();
    spi::shutdown();
    i2c::shutdown();
    virtio::shutdown();

    // Serial fica ativo até o último momento (para logs de shutdown)
}

/// Verifica se está inicializado.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock()
}
