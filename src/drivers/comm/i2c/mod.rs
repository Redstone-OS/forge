//! # I2C Bus Driver
//!
//! Driver para o barramento **I2C (Inter-Integrated Circuit)** - um
//! barramento serial de baixa velocidade usado para comunicação com
//! periféricos simples.
//!
//! ## Características:
//! - Two-wire: SDA (data) + SCL (clock)
//! - Multi-master, multi-slave
//! - Velocidades: 100kHz (standard), 400kHz (fast), 1MHz (fast+), 3.4MHz (high-speed)
//! - Endereçamento: 7-bit ou 10-bit
//!
//! ## Dispositivos Típicos:
//! - EEPROMs
//! - Sensores (temperatura, acelerômetro, giroscópio)
//! - RTCs (Real Time Clocks)
//! - Touchpads (HID-over-I2C)
//! - Audio codecs
//!
//! ## Controllers em PCs:
//! - Intel PCH (Platform Controller Hub)
//! - AMD FCH (Fusion Controller Hub)
//! - Embedded controllers
//!
//! ## STUB:
//! Este driver não está implementado. I2C em PCs geralmente requer
//! acesso via SMBus (System Management Bus) que é uma variante do I2C.

use crate::sync::Spinlock;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Velocidades I2C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cSpeed {
    /// 100 kHz.
    Standard,
    /// 400 kHz.
    Fast,
    /// 1 MHz.
    FastPlus,
    /// 3.4 MHz.
    HighSpeed,
}

/// Resultado de operações I2C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cError {
    /// Dispositivo não respondeu (NACK).
    Nack,
    /// Timeout.
    Timeout,
    /// Arbitração perdida.
    ArbitrationLost,
    /// Bus error.
    BusError,
    /// Não implementado.
    NotImplemented,
}

// =============================================================================
// TRAIT I2C
// =============================================================================

/// Interface para controllers I2C.
pub trait I2cController: Send + Sync {
    /// Escreve dados para um dispositivo.
    fn write(&self, addr: u8, data: &[u8]) -> Result<(), I2cError>;

    /// Lê dados de um dispositivo.
    fn read(&self, addr: u8, buffer: &mut [u8]) -> Result<(), I2cError>;

    /// Escreve e então lê (write-then-read).
    fn write_read(
        &self,
        addr: u8,
        write_data: &[u8],
        read_buffer: &mut [u8],
    ) -> Result<(), I2cError>;

    /// Define velocidade do bus.
    fn set_speed(&self, speed: I2cSpeed);
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o subsistema I2C.
pub fn init() {
    crate::kinfo!("(I2C) Inicializando subsistema I2C...");

    // TODO: Detectar controllers I2C via ACPI ou PCI
    crate::kwarn!("(I2C) Driver não implementado");

    *INITIALIZED.lock() = true;
}

/// Desliga o subsistema I2C.
pub fn shutdown() {
    crate::kinfo!("(I2C) Shutdown");
}

/// Verifica se I2C está disponível.
pub fn is_available() -> bool {
    false // Sempre false até implementação
}
