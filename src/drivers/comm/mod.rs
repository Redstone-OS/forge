//! # Subsistema de Comunicação (Comm)
//!
//! Este módulo agrupa drivers responsáveis pela troca de dados serial ou byte-a-byte
//! entre o processador e periféricos internos/externos.
//!
//! ## Drivers Incluídos:
//! - **Serial (UART)**: Comunicação clássica assíncrona (COM1, COM2).
//! - **VirtIO Console**: Comunicação serial otimizada para virtualização.
//! - **I2C**: Barramento de baixa velocidade para sensores e chips de controle.
//! - **SPI**: Barramento síncrono de alta velocidade para periféricos simples.
//! - **Parallel**: Interface legada para impressoras e dispositivos industriais.

pub mod i2c; // Inter-Integrated Circuit
pub mod parallel;
pub mod serial; // UART e Console Serial
pub mod spi; // Serial Peripheral Interface
pub mod virtio; // Console Virtualizado // Porta LPT / Paralela

/// Inicializa todos os drivers de comunicação do sistema.
pub fn init() {
    crate::kdebug!("(Comm) Inicializando drivers de comunicação...");

    // 1. Serial (Crucial para logs de boot - já inicializado antecipadamente pelo kernel)
    // serial::init();

    // 2. Registra os drivers funcionais no RDM
    virtio::init();
    spi::init();
    i2c::init();
    parallel::init();
}
