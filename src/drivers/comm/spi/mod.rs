//! # SPI Bus Driver
//!
//! Driver para o barramento **SPI (Serial Peripheral Interface)** - um
//! barramento síncrono de alta velocidade para periféricos.
//!
//! ## Características:
//! - Four-wire: MOSI, MISO, SCLK, CS
//! - Full duplex
//! - Master/Slave
//! - Velocidades: até 100+ MHz
//! - Sem endereçamento (usa CS para selecionar dispositivo)
//!
//! ## Dispositivos Típicos:
//! - Flash chips (NOR, NAND)
//! - SD cards (modo SPI)
//! - Displays
//! - ADCs/DACs
//! - Sensors
//!
//! ## Em PCs:
//! SPI é mais comum em sistemas embarcados. Em PCs, é usado
//! internamente para flash de firmware (SPI flash para BIOS).
//!
//! ## STUB:
//! Este driver não está implementado.

use crate::sync::Spinlock;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Modos de clock SPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    /// CPOL=0, CPHA=0.
    Mode0,
    /// CPOL=0, CPHA=1.
    Mode1,
    /// CPOL=1, CPHA=0.
    Mode2,
    /// CPOL=1, CPHA=1.
    Mode3,
}

/// Erros SPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiError {
    /// Timeout.
    Timeout,
    /// Overrun.
    Overrun,
    /// Não implementado.
    NotImplemented,
}

// =============================================================================
// TRAIT SPI
// =============================================================================

/// Interface para controllers SPI.
pub trait SpiController: Send + Sync {
    /// Transfere dados (full duplex).
    fn transfer(&self, tx: &[u8], rx: &mut [u8]) -> Result<(), SpiError>;

    /// Escreve dados.
    fn write(&self, data: &[u8]) -> Result<(), SpiError>;

    /// Lê dados.
    fn read(&self, buffer: &mut [u8]) -> Result<(), SpiError>;

    /// Define velocidade do clock.
    fn set_speed(&self, hz: u32);

    /// Define modo de clock.
    fn set_mode(&self, mode: SpiMode);
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o subsistema SPI.
pub fn init() {
    crate::kinfo!("(SPI) Inicializando subsistema SPI...");

    crate::kwarn!("(SPI) Driver não implementado");

    *INITIALIZED.lock() = true;
}

/// Desliga o subsistema SPI.
pub fn shutdown() {
    crate::kinfo!("(SPI) Shutdown");
}

/// Verifica se SPI está disponível.
pub fn is_available() -> bool {
    false
}
