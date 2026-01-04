//! # xHCI Global Functions
//!
//! Funções globais auxiliares do xHCI.

use super::*;

/// Retorna o primeiro controller xHCI disponível.
pub fn get_controller() -> Option<Arc<XhciController>> {
    get_primary_controller()
}

/// Verifica se xHCI está disponível.
pub fn is_available() -> bool {
    *INITIALIZED.lock() && controller_count() > 0
}

/// Escaneia portas de todos os controllers.
pub fn scan_ports() {
    crate::kinfo!("(xHCI) Escaneando portas...");

    let controllers = XHCI_CONTROLLERS.lock();

    for ctrl in controllers.iter() {
        let port_count = ctrl.port_count();
        crate::kinfo!("(xHCI) Controller com", port_count, "portas");

        // TODO: Para cada porta, verificar status e enumerar dispositivos
    }
}
