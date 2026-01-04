//! # Parallel Port Driver (LPT)
//!
//! Driver para a **porta paralela** (LPT) - interface legada usada
//! principalmente para impressoras.
//!
//! ## Características:
//! - 8 bits de dados paralelos
//! - Portas: LPT1 (0x378), LPT2 (0x278), LPT3 (0x3BC)
//! - Modos: SPP, EPP, ECP
//!
//! ## Histórico:
//! A porta paralela foi o método principal de conexão de impressoras
//! em PCs até ser substituída por USB. Hoje é praticamente obsoleta,
//! mas ainda pode ser encontrada em hardware industrial.
//!
//! ## STUB:
//! Este driver não está implementado. A porta paralela é raramente
//! necessária em sistemas modernos.

use crate::sync::Spinlock;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Endereço base LPT1.
pub const LPT1_PORT: u16 = 0x378;

/// Endereço base LPT2.
pub const LPT2_PORT: u16 = 0x278;

/// Endereço base LPT3.
pub const LPT3_PORT: u16 = 0x3BC;

// Registradores (offset do base)
// TODO: Revisar no futuro
#[allow(unused)]
const REG_DATA: u16 = 0; // Data register
                         // TODO: Revisar no futuro
#[allow(unused)]
const REG_STATUS: u16 = 1; // Status register
                           // TODO: Revisar no futuro
#[allow(unused)]
const REG_CONTROL: u16 = 2; // Control register

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o subsistema de porta paralela.
pub fn init() {
    crate::kinfo!("(Parallel) Inicializando driver de porta paralela...");

    crate::kwarn!("(Parallel) Driver não implementado (obsoleto)");

    *INITIALIZED.lock() = true;
}

/// Desliga o subsistema.
pub fn shutdown() {
    crate::kinfo!("(Parallel) Shutdown");
}

/// Verifica se porta paralela está disponível.
pub fn is_available() -> bool {
    false
}
