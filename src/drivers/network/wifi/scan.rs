//! # Wi-Fi Scanning and SSID Management
//!
//! Gerenciamento de busca por redes sem fio e armazenamento de SSIDs encontrados.

use alloc::string::String;
use alloc::vec::Vec;

/// Informações de uma rede sem fio encontrada
pub struct ScanResult {
    pub ssid: String,
    pub bssid: [u8; 6],
    pub rssi: i8,
    pub channel: u8,
    pub security: SecurityType,
}

pub enum SecurityType {
    Open,
    Wep,
    Wpa2,
    Wpa3,
}

/// STUB: Inicia um scan de redes
pub fn start_scan() {
    // TODO:
    // 1. Enviar comando de scan para o hardware
    // 2. Iterar sobre os canais (1-13 em 2.4GHz)
    // 3. Capturar Beacon frames
    crate::kinfo!("(Net/Wifi) Iniciando busca por redes...");
}

/// STUB: Retorna a lista de redes encontradas
pub fn get_results() -> Vec<ScanResult> {
    Vec::new()
}
