//! # NUMA Support
//!
//! Suporte a Non-Uniform Memory Access (preparado, stub).

pub mod topology;

/// ID de nó NUMA
pub type NodeId = u8;

/// NodeManager gerencia memória de um nó
pub struct NodeManager {
    pub node_id: NodeId,
    // TODO: zones, stats
}

impl NodeManager {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }
}

/// Retorna nó da CPU atual
pub fn current_node() -> NodeId {
    // TODO: Ler de tabela cpu_to_node
    0
}

/// Retorna número de nós
pub fn node_count() -> usize {
    // TODO: Detectar via ACPI SRAT
    1
}
