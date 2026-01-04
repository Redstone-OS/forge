//! # VirtIO Console Driver
//!
//! Driver para **VirtIO Console** - console serial paravirtualizado
//! para QEMU, KVM e outros hypervisors.
//!
//! ## Características:
//! - Alta performance (sem emulação de hardware)
//! - Múltiplas portas de console
//! - Suporte a resize de terminal
//! - Controle de fluxo
//!
//! ## Vs Serial:
//! O VirtIO Console é mais eficiente que emular um UART 16550,
//! mas requer suporte do guest OS. O serial tradicional funciona
//! sem drivers especiais.
//!
//! ## Uso:
//! - Console principal do guest
//! - Debug output
//! - Comunicação host-guest
//!
//! ## STUB:
//! Este driver não está implementado. O serial tradicional (UART)
//! é suficiente para debug e é mais simples de manter.

use crate::sync::Spinlock;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Tipo de dispositivo VirtIO Console (Device ID 3).
pub const VIRTIO_CONSOLE_DEVICE_ID: u16 = 3;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o driver VirtIO Console.
pub fn init() {
    crate::kinfo!("(VirtIO Console) Inicializando driver...");

    // Busca dispositivo VirtIO Console
    // TODO: Integrar com VirtIO bus

    crate::kwarn!("(VirtIO Console) Driver não implementado (usando UART)");

    *INITIALIZED.lock() = true;
}

/// Desliga o driver.
pub fn shutdown() {
    crate::kinfo!("(VirtIO Console) Shutdown");
}

/// Verifica se VirtIO Console está disponível.
pub fn is_available() -> bool {
    false
}

/// Escreve para o console VirtIO.
///
/// ## STUB:
/// Não implementado.
pub fn write(_data: &[u8]) -> Result<usize, ()> {
    Err(())
}

/// Lê do console VirtIO.
///
/// ## STUB:
/// Não implementado.
pub fn read(_buffer: &mut [u8]) -> Result<usize, ()> {
    Err(())
}
