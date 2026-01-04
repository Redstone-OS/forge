//! # HID - Human Interface Device
//!
//! Este módulo implementa o parser HID universal, usado para interpretar
//! dados de dispositivos USB HID, I2C HID e Bluetooth HID.
//!
//! ## O que é HID?
//! HID é um protocolo padronizado para dispositivos de entrada que descreve
//! o formato dos dados através de um "Report Descriptor".
//!
//! ## Report Descriptor:
//! Cada dispositivo HID fornece um descriptor que descreve:
//! - Tipo de dados (botões, eixos, etc)
//! - Tamanho dos campos
//! - Range de valores
//! - Usages (o que cada campo significa)
//!
//! ## Fluxo:
//! 1. Dispositivo conecta
//! 2. Host lê Report Descriptor
//! 3. Parser interpreta descriptor
//! 4. Parser decodifica reports de entrada
//! 5. Eventos são gerados para o sistema
//!
//! ## STUB:
//! Parser básico implementado. Suporte completo pendente.

pub mod report;
pub mod types; // Tipos HID
pub mod usage; // Usage Pages e Usages // Report parser

// Re-exports
pub use report::parse_report;
pub use types::*;

use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o parser HID.
pub fn init() {
    crate::kinfo!("(HID) Inicializando parser HID...");

    *INITIALIZED.lock() = true;

    crate::kinfo!("(HID) Parser inicializado");
}

/// Desliga o parser HID.
pub fn shutdown() {
    crate::kinfo!("(HID) Shutdown");
}

/// Verifica se HID está inicializado.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock()
}
