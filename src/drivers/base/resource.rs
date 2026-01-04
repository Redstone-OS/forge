//! # Gerenciador de Recursos Físicos (Resource Manager)
//!
//! Este módulo é o **árbitro** dos recursos de hardware. Garante que
//! dois drivers não tentem usar o mesmo recurso simultaneamente.
//!
//! ## Recursos Gerenciados:
//! - **I/O Ports**: Portas de comunicação x86 (ex: 0x3F8 para Serial)
//! - **MMIO**: Regiões de memória mapeada para hardware
//! - **IRQs**: Linhas de interrupção
//! - **DMA Channels**: Canais de acesso direto à memória (legado)
//!
//! ## Filosofia:
//! > "Drivers pedem, a Base decide. Conflitos são detectados ANTES
//! >  de causar instabilidade."
//!
//! ## Uso:
//! 1. Driver solicita recursos durante probe()
//! 2. ResourceManager verifica conflitos
//! 3. Se livre, recurso é reservado
//! 4. Se conflito, probe() falha com ResourceConflict
//! 5. Recursos são liberados automaticamente em remove()

use super::device::DeviceId;
use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// TIPOS DE RECURSO
// =============================================================================

/// Tipos de recursos físicos que podem ser solicitados.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Porta de I/O (arquitetura x86).
    /// Intervalo: [port, port + len - 1]
    IoPort { port: u16, len: u16 },

    /// Região de memória mapeada (MMIO).
    /// Usado para registradores de hardware.
    Mmio { start: u64, len: u64 },

    /// Linha de interrupção de hardware.
    Irq { line: u8 },

    /// Canal de DMA legado (controller 8237).
    /// Usado apenas para hardware muito antigo.
    DmaChannel { channel: u8 },

    /// Vetor MSI/MSI-X.
    /// Para interrupções baseadas em mensagem.
    Msi { vector: u16 },
}

impl ResourceType {
    /// Retorna descrição legível do recurso.
    pub fn description(&self) -> &'static str {
        match self {
            Self::IoPort { .. } => "I/O Port",
            Self::Mmio { .. } => "MMIO Region",
            Self::Irq { .. } => "IRQ Line",
            Self::DmaChannel { .. } => "DMA Channel",
            Self::Msi { .. } => "MSI Vector",
        }
    }
}

// =============================================================================
// REGISTRO DE ALOCAÇÃO
// =============================================================================

/// Registro de um recurso alocado.
#[derive(Debug, Clone)]
pub struct ResourceRecord {
    /// Tipo e parâmetros do recurso.
    pub res_type: ResourceType,

    /// Nome descritivo (para logs).
    pub name: &'static str,

    /// ID do dispositivo que possui o recurso.
    pub owner: Option<DeviceId>,

    /// Flag indicando se está em uso.
    pub in_use: bool,
}

// =============================================================================
// GERENCIADOR DE RECURSOS
// =============================================================================

/// Gerenciador central de recursos.
pub struct ResourceManager {
    /// Lista de recursos alocados.
    allocated: Vec<ResourceRecord>,

    /// Flag de inicialização.
    initialized: bool,
}

/// Instância global do ResourceManager.
static RESOURCE_MANAGER: Spinlock<ResourceManager> = Spinlock::new(ResourceManager {
    allocated: Vec::new(),
    initialized: false,
});

impl ResourceManager {
    /// Verifica se dois recursos conflitam.
    fn conflicts(&self, a: &ResourceType, b: &ResourceType) -> bool {
        match (a, b) {
            // I/O Ports: conflito se intervalos se sobrepõem
            (
                ResourceType::IoPort { port: p1, len: l1 },
                ResourceType::IoPort { port: p2, len: l2 },
            ) => {
                let end1 = *p1 as u32 + *l1 as u32;
                let end2 = *p2 as u32 + *l2 as u32;
                (*p1 as u32) < end2 && (*p2 as u32) < end1
            }

            // MMIO: conflito se regiões se sobrepõem
            (
                ResourceType::Mmio { start: s1, len: l1 },
                ResourceType::Mmio { start: s2, len: l2 },
            ) => {
                let end1 = s1 + l1;
                let end2 = s2 + l2;
                *s1 < end2 && *s2 < end1
            }

            // IRQ: conflito se mesmo número (exceto se compartilhável)
            (ResourceType::Irq { line: i1 }, ResourceType::Irq { line: i2 }) => i1 == i2,

            // DMA: conflito se mesmo canal
            (
                ResourceType::DmaChannel { channel: c1 },
                ResourceType::DmaChannel { channel: c2 },
            ) => c1 == c2,

            // MSI: conflito se mesmo vetor
            (ResourceType::Msi { vector: v1 }, ResourceType::Msi { vector: v2 }) => v1 == v2,

            // Tipos diferentes nunca conflitam
            _ => false,
        }
    }
}

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o gerenciador de recursos.
pub fn init() {
    crate::kinfo!("(Resource) Inicializando Resource Manager...");

    let mut mgr = RESOURCE_MANAGER.lock();
    mgr.initialized = true;

    crate::kinfo!("(Resource) Sistema pronto");
}

/// Solicita alocação de um recurso.
///
/// ## Parâmetros:
/// - `res`: Tipo e especificação do recurso
/// - `name`: Nome descritivo (para logs e debug)
/// - `owner`: ID do dispositivo solicitante
///
/// ## Retorno:
/// - true: Recurso alocado com sucesso
/// - false: Conflito detectado ou erro
pub fn request(res: ResourceType, name: &'static str, owner: DeviceId) -> bool {
    let mut mgr = RESOURCE_MANAGER.lock();

    // Verifica conflitos com recursos existentes
    for existing in mgr.allocated.iter() {
        if existing.in_use && mgr.conflicts(&existing.res_type, &res) {
            crate::kerror!("(Resource) CONFLITO:", name, "conflita com", existing.name);
            return false;
        }
    }

    // Sem conflito - aloca
    mgr.allocated.push(ResourceRecord {
        res_type: res,
        name,
        owner: Some(owner),
        in_use: true,
    });

    crate::kinfo!("(Resource) Alocado:", name, "para device", owner.0);

    true
}

/// Libera um recurso específico.
pub fn release(res: ResourceType) {
    let mut mgr = RESOURCE_MANAGER.lock();

    if let Some(record) = mgr
        .allocated
        .iter_mut()
        .find(|r| r.res_type == res && r.in_use)
    {
        record.in_use = false;
        crate::kinfo!("(Resource) Liberado:", record.name);
    }
}

/// Libera todos os recursos de um dispositivo.
///
/// Chamado durante remove() ou quando dispositivo é isolado.
pub fn release_all_for_device(owner: DeviceId) {
    crate::kinfo!("(Resource) Liberando recursos do device:", owner.0);

    let mut mgr = RESOURCE_MANAGER.lock();
    let mut count = 0;

    for record in mgr.allocated.iter_mut() {
        if record.owner == Some(owner) && record.in_use {
            record.in_use = false;
            count += 1;
        }
    }

    crate::kinfo!("(Resource) Liberados", count, "recursos");
}

/// Solicita uma porta de I/O.
///
/// Conveniência sobre request() genérico.
pub fn request_io_port(port: u16, len: u16, name: &'static str, owner: DeviceId) -> bool {
    request(ResourceType::IoPort { port, len }, name, owner)
}

/// Solicita uma região MMIO.
pub fn request_mmio(start: u64, len: u64, name: &'static str, owner: DeviceId) -> bool {
    request(ResourceType::Mmio { start, len }, name, owner)
}

/// Solicita uma linha de IRQ.
pub fn request_irq(line: u8, name: &'static str, owner: DeviceId) -> bool {
    request(ResourceType::Irq { line }, name, owner)
}

/// Retorna lista de todos os recursos alocados (para debug/VFS).
pub fn list_resources() -> Vec<String> {
    let mgr = RESOURCE_MANAGER.lock();

    mgr.allocated
        .iter()
        .filter(|r| r.in_use)
        .map(|r| {
            let owner_str = match r.owner {
                Some(id) => alloc::format!("device {}", id.0),
                None => String::from("system"),
            };
            alloc::format!("{}: {:?} ({})", r.name, r.res_type, owner_str)
        })
        .collect()
}

/// Verifica se um recurso específico está disponível.
pub fn is_available(res: &ResourceType) -> bool {
    let mgr = RESOURCE_MANAGER.lock();

    !mgr.allocated
        .iter()
        .any(|r| r.in_use && mgr.conflicts(&r.res_type, res))
}
