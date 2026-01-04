//! # WiFi Drivers
//!
//! Este módulo conterá drivers para adaptadores WiFi (802.11).
//!
//! ## Complexidade:
//! WiFi é significativamente mais complexo que Ethernet:
//! - Gerenciamento de BSS/IBSS
//! - Scanning de redes
//! - Autenticação (WPA2, etc)
//! - Criptografia (AES, etc)
//! - Roaming
//! - Power management
//!
//! ## Subsistemas Necessários:
//! - cfg80211 (configuração wireless)
//! - mac80211 (MAC layer)
//! - Supplicant (WPA)
//!
//! ## Drivers Futuros:
//! - Intel iwlwifi
//! - Realtek RTL8xxx
//! - Qualcomm/Atheros ath9k/ath10k
//!
//! ## STUB:
//! WiFi será implementado em fases posteriores.
//! Este módulo define apenas a estrutura básica.

use super::traits::*;
use crate::sync::Spinlock;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa subsistema WiFi.
pub fn init() {
    crate::kinfo!("(WiFi) Inicializando subsistema WiFi...");
    crate::kwarn!("(WiFi) WiFi não implementado nesta versão");

    *INITIALIZED.lock() = true;
}

/// Desliga subsistema WiFi.
pub fn shutdown() {
    crate::kinfo!("(WiFi) Shutdown");
}

/// Verifica se WiFi está disponível.
pub fn is_available() -> bool {
    false // Sempre false por enquanto
}

/// Escaneia redes WiFi disponíveis.
///
/// ## STUB:
/// Sempre retorna lista vazia.
pub fn scan_networks() -> alloc::vec::Vec<WifiNetwork> {
    crate::kwarn!("(WiFi) scan_networks() não implementado");
    alloc::vec::Vec::new()
}
