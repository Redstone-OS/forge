//! # Sistema de Fallback Progressivo
//!
//! Este módulo implementa a lógica de **fallback progressivo** do RDS.
//! Quando um driver falha e não pode ser recuperado, o sistema tenta
//! usar drivers alternativos em ordem de preferência.
//!
//! ## Filosofia:
//! > "Tente se virar com o que tiver disponível. Algo funcionando
//! >  é melhor que nada funcionando."
//!
//! ## Ordem de Fallback por Categoria:
//!
//! ### Display:
//! 1. nvidia (proprietário) → 2. intel/amd → 3. vesa → 4. framebuffer → 5. texto
//!
//! ### Storage:
//! 1. nvme → 2. ahci → 3. outro disco → 4. ramdisk (emergência)
//!
//! ### Network:
//! 1. driver específico → 2. virtio → 3. loopback only
//!
//! ## STUB:
//! A implementação completa requer o sistema de matching de drivers.
//! Por enquanto, apenas estrutura e logs.

use super::device::{Device, DeviceId, DeviceState};
use super::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTRUTURA DE FALLBACK
// =============================================================================

/// Entrada na cadeia de fallback.
///
/// Define um driver alternativo e sua prioridade.
#[derive(Debug, Clone)]
pub struct FallbackEntry {
    /// Nome do driver.
    pub driver_name: &'static str,

    /// Prioridade (menor = mais preferido).
    pub priority: u8,

    /// Se true, este é o último recurso.
    pub is_last_resort: bool,

    /// Descrição para logs.
    pub description: &'static str,
}

/// Cadeia de fallback para um tipo de dispositivo.
#[derive(Debug, Clone)]
pub struct FallbackChain {
    /// Tipo de dispositivo.
    pub device_type: DeviceType,

    /// Lista ordenada de drivers fallback.
    pub entries: Vec<FallbackEntry>,
}

// =============================================================================
// CADEIAS DE FALLBACK PADRÃO
// =============================================================================

/// Retorna a cadeia de fallback para Display.
fn display_fallback_chain() -> FallbackChain {
    FallbackChain {
        device_type: DeviceType::Display,
        entries: vec![
            FallbackEntry {
                driver_name: "nvidia-gpu",
                priority: 1,
                is_last_resort: false,
                description: "NVIDIA proprietário (melhor performance)",
            },
            FallbackEntry {
                driver_name: "intel-gpu",
                priority: 2,
                is_last_resort: false,
                description: "Intel HD Graphics",
            },
            FallbackEntry {
                driver_name: "amd-gpu",
                priority: 2,
                is_last_resort: false,
                description: "AMD/ATI Graphics",
            },
            FallbackEntry {
                driver_name: "vesa-generic",
                priority: 3,
                is_last_resort: false,
                description: "VESA genérico (compatibilidade)",
            },
            FallbackEntry {
                driver_name: "bochs-vga",
                priority: 3,
                is_last_resort: false,
                description: "Bochs/QEMU VGA",
            },
            FallbackEntry {
                driver_name: "framebuffer-simple",
                priority: 4,
                is_last_resort: false,
                description: "Framebuffer simples (GOP/LFB)",
            },
            FallbackEntry {
                driver_name: "text-mode",
                priority: 5,
                is_last_resort: true,
                description: "Modo texto 80x25 (último recurso)",
            },
        ],
    }
}

/// Retorna a cadeia de fallback para Storage.
fn storage_fallback_chain() -> FallbackChain {
    FallbackChain {
        device_type: DeviceType::Storage,
        entries: vec![
            FallbackEntry {
                driver_name: "nvme-driver",
                priority: 1,
                is_last_resort: false,
                description: "NVMe SSD (máxima performance)",
            },
            FallbackEntry {
                driver_name: "ahci-driver",
                priority: 2,
                is_last_resort: false,
                description: "AHCI/SATA",
            },
            FallbackEntry {
                driver_name: "virtio-blk",
                priority: 2,
                is_last_resort: false,
                description: "VirtIO Block (VMs)",
            },
            FallbackEntry {
                driver_name: "ata-driver",
                priority: 3,
                is_last_resort: false,
                description: "ATA/IDE legado",
            },
            FallbackEntry {
                driver_name: "usb-mass-storage",
                priority: 4,
                is_last_resort: false,
                description: "USB Mass Storage",
            },
            FallbackEntry {
                driver_name: "ramdisk",
                priority: 5,
                is_last_resort: true,
                description: "RAM Disk (dados em memória!)",
            },
        ],
    }
}

/// Retorna a cadeia de fallback para Network.
fn network_fallback_chain() -> FallbackChain {
    FallbackChain {
        device_type: DeviceType::Network,
        entries: vec![
            FallbackEntry {
                driver_name: "intel-e1000",
                priority: 1,
                is_last_resort: false,
                description: "Intel e1000/e1000e",
            },
            FallbackEntry {
                driver_name: "realtek-rtl8139",
                priority: 2,
                is_last_resort: false,
                description: "Realtek RTL8139/8169",
            },
            FallbackEntry {
                driver_name: "virtio-net",
                priority: 2,
                is_last_resort: false,
                description: "VirtIO Network",
            },
            FallbackEntry {
                driver_name: "loopback",
                priority: 5,
                is_last_resort: true,
                description: "Loopback only (sem rede externa)",
            },
        ],
    }
}

// =============================================================================
// REGISTRO DE FALLBACK
// =============================================================================

/// Lista global de cadeias de fallback.
static FALLBACK_CHAINS: Spinlock<Vec<FallbackChain>> = Spinlock::new(Vec::new());

/// Inicializa o sistema de fallback.
pub fn init() {
    crate::kinfo!("(Fallback) Inicializando sistema de fallback...");

    let mut chains = FALLBACK_CHAINS.lock();

    // Registra cadeias padrão
    chains.push(display_fallback_chain());
    chains.push(storage_fallback_chain());
    chains.push(network_fallback_chain());

    crate::kinfo!("(Fallback) Cadeias registradas:", chains.len());
}

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Tenta aplicar fallback para um dispositivo.
///
/// Percorre a cadeia de fallback tentando cada driver alternativo
/// até encontrar um que funcione.
///
/// ## STUB:
/// Atualmente apenas loga a tentativa. Implementação real requer
/// integração com o sistema de matching de drivers.
pub fn try_fallback(dev_arc: Arc<Spinlock<Device>>, dev_type: DeviceType) {
    crate::kinfo!("(Fallback) Iniciando busca de driver alternativo...");

    let chains = FALLBACK_CHAINS.lock();

    // Busca cadeia para este tipo
    let chain = match chains.iter().find(|c| c.device_type == dev_type) {
        Some(c) => c,
        None => {
            crate::kwarn!("(Fallback) Sem cadeia de fallback para:", dev_type.as_str());
            return;
        }
    };

    crate::kinfo!(
        "(Fallback) Cadeia encontrada com",
        chain.entries.len(),
        "opções"
    );

    // Tenta cada driver na ordem de prioridade
    for entry in chain.entries.iter() {
        crate::kinfo!(
            "(Fallback) Tentando:",
            entry.driver_name,
            "-",
            entry.description
        );

        // TODO: Implementar tentativa real de carregar o driver
        // 1. Buscar driver pelo nome no DriverManager
        // 2. Tentar probe()
        // 3. Se sucesso, retornar
        // 4. Se falha, continuar para próximo

        if entry.is_last_resort {
            crate::kwarn!("(Fallback) Tentando último recurso:", entry.driver_name);
        }

        // Por enquanto, apenas simula falha
        crate::kwarn!("(Fallback) try_fallback() não totalmente implementado");
    }

    crate::kerror!(
        "(Fallback) Todas as opções falharam para tipo:",
        dev_type.as_str()
    );

    // Marca dispositivo como morto se todas falharam
    let mut dev = dev_arc.lock();
    dev.set_state(DeviceState::Dead);
}

/// Registra uma cadeia de fallback customizada.
pub fn register_chain(chain: FallbackChain) {
    crate::kinfo!(
        "(Fallback) Registrando cadeia para:",
        chain.device_type.as_str()
    );

    let mut chains = FALLBACK_CHAINS.lock();

    // Remove cadeia existente para este tipo (se houver)
    chains.retain(|c| c.device_type != chain.device_type);

    // Adiciona nova
    chains.push(chain);
}

/// Retorna a cadeia de fallback para um tipo de dispositivo.
pub fn get_chain(dev_type: DeviceType) -> Option<FallbackChain> {
    FALLBACK_CHAINS
        .lock()
        .iter()
        .find(|c| c.device_type == dev_type)
        .cloned()
}

/// Retorna o próximo driver na cadeia após o atual.
///
/// Útil para saber qual seria o fallback se o driver atual falhar.
pub fn get_next_fallback(dev_type: DeviceType, current_driver: &str) -> Option<&'static str> {
    let chains = FALLBACK_CHAINS.lock();

    let chain = chains.iter().find(|c| c.device_type == dev_type)?;

    // Encontra posição do driver atual
    let current_pos = chain
        .entries
        .iter()
        .position(|e| e.driver_name == current_driver)?;

    // Retorna o próximo
    chain.entries.get(current_pos + 1).map(|e| e.driver_name)
}
