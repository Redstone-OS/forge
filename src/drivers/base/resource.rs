//! # Gerenciador de Recursos Físicos (Resource Manager)
//!
//! Atua como o "árbitro" dos recursos de hardware do computador.
//! Garante que dois drivers não tentem acessar simultaneamente o mesmo canal de comunicação,
//! o que causaria instabilidade, corrupção de dados ou travamentos parciais.
//!
//! ## Recursos Gerenciados:
//! - **I/O Ports**: Portas de comunicação herdadas (ex: 0x3F8 para Serial).
//! - **MMIO (Memory Mapped I/O)**: Endereços de memória física mapeados para hardware.
//! - **IRQs (Interrupt Requests)**: Linhas de sinalização de eventos de hardware.
//! - **DMA Channels**: Canais de acesso direto à memória.
//!
//! O `ResourceManager` mantém um registro de "quem é dono de quê", permitindo a auditoria
//! do sistema e a prevenção de conflitos durante o boot e o hotplug.

use super::device::DeviceId;
use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec::Vec;

/// Tipos de recursos físicos disponíveis na plataforma
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Porta I/O (Arquitetura x86)
    IoPort { port: u16, len: u16 },
    /// Região de Memória Mapeada (MMIO)
    Mmio { start: u64, len: u64 },
    /// Linha de Interrupção
    Irq { line: u8 },
    /// Canal de DMA legado
    DmaChannel { channel: u8 },
}

/// Registro de alocação de um recurso
pub struct ResourceRecord {
    pub res_type: ResourceType,
    pub name: &'static str,
    pub owner: Option<DeviceId>,
}

pub struct ResourceManager {
    allocated: Vec<ResourceRecord>,
}

static RESOURCE_MANAGER: Spinlock<ResourceManager> = Spinlock::new(ResourceManager {
    allocated: Vec::new(),
});

impl ResourceManager {
    /// Tenta solicitar um recurso para um dispositivo.
    /// Retorna Ok(()) se o recurso estiver livre e foi reservado.
    pub fn request(
        &mut self,
        res_type: ResourceType,
        name: &'static str,
        owner: Option<DeviceId>,
    ) -> bool {
        // Verificar se há conflito com recursos existentes
        for existing in &self.allocated {
            if self.conflicts(&existing.res_type, &res_type) {
                crate::kerror!("(Resource) CONFLITO DETECTADO:", name);
                return false;
            }
        }

        self.allocated.push(ResourceRecord {
            res_type,
            name,
            owner,
        });
        true
    }

    /// Libera um recurso baseado no tipo e endereço.
    pub fn release(&mut self, res_type: ResourceType) {
        self.allocated.retain(|r| r.res_type != res_type);
    }

    /// Lógica de detecção de sobreposição de intervalos e colisões de IRQs.
    fn conflicts(&self, a: &ResourceType, b: &ResourceType) -> bool {
        match (a, b) {
            (
                ResourceType::IoPort { port: p1, len: l1 },
                ResourceType::IoPort { port: p2, len: l2 },
            ) => p1 < &(p2 + l2) && p2 < &(p1 + l1),
            (
                ResourceType::Mmio {
                    start: s1,
                    len: len1,
                },
                ResourceType::Mmio {
                    start: s2,
                    len: len2,
                },
            ) => s1 < &(s2 + len2) && s2 < &(s1 + len1),
            (ResourceType::Irq { line: i1 }, ResourceType::Irq { line: i2 }) => i1 == i2,
            (
                ResourceType::DmaChannel { channel: c1 },
                ResourceType::DmaChannel { channel: c2 },
            ) => c1 == c2,
            _ => false,
        }
    }
}

/// Interface pública para solicitar recursos (Thread-Safe)
pub fn request_resource(res: ResourceType, name: &'static str, owner: DeviceId) -> bool {
    RESOURCE_MANAGER.lock().request(res, name, Some(owner))
}

/// Interface pública para liberar recursos (Thread-Safe)
pub fn release_resource(res: ResourceType) {
    RESOURCE_MANAGER.lock().release(res);
}

/// Retorna uma lista de todos os recursos alocados (para ferramentas de debug/VFS)
pub fn list_resources() -> Vec<String> {
    let mgr = RESOURCE_MANAGER.lock();
    mgr.allocated
        .iter()
        .map(|r| format!("{}: {:?}", r.name, r.res_type))
        .collect()
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Árvore de Recursos (Hierarquia):
//    - Implementar um modelo onde recursos podem ter sub-recursos (ex: Um BAR
//      PCI de 1GB que é subdividido em várias regiões menores por drivers).
//
// 2. Alocação Dinâmica (Resource Balancing):
//    - Permitir que o kernel escolha automaticamente uma faixa de memória ou
//      porta livre se o driver não exigir um endereço fixo.
//
// 3. Integração com ACPI (_CRS):
//    - Ler as tabelas ACPI para pré-alocar recursos críticos da placa-mãe e
//      evitar que drivers tentem usá-los.
//
// 4. Suporte a Multi-Bridge:
//    - Lidar com barramentos que traduzem janelas de endereços (ex: Endereço
//      no barramento PCI mapeado para um endereço diferente na CPU).
//
// 5. Interface /devices/resources:
//    - Exportar a lista de recursos para o VFS, permitindo visualizar o mapa
//      de hardware do sistema de forma amigável (estilo /proc/ioports).
