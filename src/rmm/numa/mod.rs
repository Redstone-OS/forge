//! # NUMA Support
//!
//! Suporte para Non-Uniform Memory Access (NUMA).
//!
//! ## Conceitos
//!
//! Em sistemas NUMA, CPUs são organizadas em "nós" (nodes), onde cada nó
//! tem memória local. Acesso à memória local é mais rápido que acesso
//! à memória de outros nós.
//!
//! ```text
//! ┌────────────────────────────────────────────────────────┐
//! │                      Sistema NUMA                      │
//! ├────────────────────────────────────────────────────────┤
//! │                                                        │
//! │  ┌─────────────────────┐      ┌─────────────────────┐  │
//! │  │       Node 0        │      │       Node 1        │  │
//! │  │  ┌───────┬───────┐  │      │  ┌───────┬───────┐  │  │
//! │  │  │ CPU 0 │ CPU 1 │  │      │  │ CPU 2 │ CPU 3 │  │  │
//! │  │  └───────┴───────┘  │      │  └───────┴───────┘  │  │
//! │  │         │           │      │         │           │  │
//! │  │    ┌────▼────┐      │      │    ┌────▼────┐      │  │
//! │  │    │ Memory  │      │◄────►│    │ Memory  │      │  │
//! │  │    │  16 GB  │      │      │    │  16 GB  │      │  │
//! │  │    └─────────┘      │      │    └─────────┘      │  │
//! │  └─────────────────────┘      └─────────────────────┘  │
//! │              │                          │              │
//! │              └────────────┬─────────────┘              │
//! │                           │                            │
//! │                      Interconnect                      │
//! │                      (QPI/UPI/HT)                      │
//! └────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Políticas
//!
//! - **Local**: Aloca do nó local da CPU (padrão)
//! - **Interleave**: Distribui páginas entre nós (bandwidth)
//! - **Preferred**: Prefere um nó, fallback para outros
//! - **Bind**: Força alocação de um nó específico
//!
//! ## Detecção
//!
//! NUMA é detectado via ACPI SRAT/SLIT tables.

pub mod policy;
pub mod topology;

pub use policy::{NumaPolicy, NumaPolicyType};
pub use topology::{NumaNode, NumaTopology};

use crate::sync::Spinlock;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Topologia NUMA global
static NUMA_TOPOLOGY: Spinlock<Option<NumaTopology>> = Spinlock::new(None);

/// Flag indicando se sistema é NUMA
static IS_NUMA: AtomicBool = AtomicBool::new(false);

/// Número de nós NUMA
static NUM_NODES: AtomicU32 = AtomicU32::new(1);

/// ID do nó atual (per-CPU seria melhor, mas simplificamos)
static CURRENT_NODE: AtomicU32 = AtomicU32::new(0);

// =============================================================================
// Inicialização
// =============================================================================

/// Inicializa subsistema NUMA
///
/// Detecta topologia via ACPI e configura políticas.
pub fn init() {
    crate::kinfo!("(NUMA) Inicializando...");

    // Tenta detectar topologia
    if let Some(topology) = detect_topology() {
        let num_nodes = topology.node_count();

        if num_nodes > 1 {
            IS_NUMA.store(true, Ordering::Release);
            NUM_NODES.store(num_nodes as u32, Ordering::Release);

            crate::kinfo!(
                "(NUMA) Detectados {} nós, {} CPUs totais",
                num_nodes,
                topology.total_cpus()
            );

            // Log memória por nó
            for node in topology.nodes() {
                crate::kinfo!(
                    "(NUMA)   Node {}: {} MB RAM, {} CPUs",
                    node.id,
                    node.memory_size / (1024 * 1024),
                    node.cpu_count
                );
            }

            *NUMA_TOPOLOGY.lock() = Some(topology);
        } else {
            crate::kinfo!("(NUMA) Sistema UMA (único nó)");
        }
    } else {
        crate::kinfo!("(NUMA) Topologia não detectada, assumindo UMA");
    }
}

/// Detecta topologia NUMA via ACPI
fn detect_topology() -> Option<NumaTopology> {
    // TODO: Parse ACPI SRAT/SLIT tables

    // Por enquanto, assume UMA
    let mut topology = NumaTopology::new();

    // Adiciona nó único com toda a memória
    topology.add_node(NumaNode {
        id: 0,
        memory_start: 0,
        memory_size: 0,               // Será preenchido depois
        cpu_mask: 0xFFFFFFFF,         // Todas as CPUs
        cpu_count: 1,                 // Placeholder
        distance_to: alloc::vec![10], // Distância para si mesmo
    });

    Some(topology)
}

// =============================================================================
// API Pública
// =============================================================================

/// Verifica se sistema é NUMA
#[inline]
pub fn is_numa() -> bool {
    IS_NUMA.load(Ordering::Acquire)
}

/// Número de nós NUMA
#[inline]
pub fn node_count() -> usize {
    NUM_NODES.load(Ordering::Relaxed) as usize
}

/// Retorna ID do nó atual
#[inline]
pub fn current_node() -> u32 {
    CURRENT_NODE.load(Ordering::Relaxed)
}

/// Define nó atual (chamado pelo scheduler ao migrar)
pub fn set_current_node(node_id: u32) {
    CURRENT_NODE.store(node_id, Ordering::Relaxed);
}

/// Retorna nó que contém endereço físico
pub fn node_for_addr(phys_addr: u64) -> u32 {
    if let Some(ref topology) = *NUMA_TOPOLOGY.lock() {
        return topology.node_for_addr(phys_addr);
    }
    0 // Default: nó 0
}

/// Retorna nó preferido para CPU
pub fn node_for_cpu(cpu_id: u32) -> u32 {
    if let Some(ref topology) = *NUMA_TOPOLOGY.lock() {
        return topology.node_for_cpu(cpu_id);
    }
    0
}

/// Retorna distância entre dois nós
///
/// 10 = local, valores maiores = mais longe
pub fn distance(from_node: u32, to_node: u32) -> u32 {
    if from_node == to_node {
        return 10; // Local
    }

    if let Some(ref topology) = *NUMA_TOPOLOGY.lock() {
        return topology.distance(from_node, to_node);
    }

    20 // Default: remoto
}

/// Retorna lista de nós ordenados por distância
pub fn nodes_by_distance(from_node: u32) -> alloc::vec::Vec<u32> {
    if let Some(ref topology) = *NUMA_TOPOLOGY.lock() {
        return topology.nodes_by_distance(from_node);
    }

    // Fallback: apenas nó 0
    alloc::vec![0]
}

/// Retorna memória total no nó
pub fn node_memory_size(node_id: u32) -> u64 {
    if let Some(ref topology) = *NUMA_TOPOLOGY.lock() {
        if let Some(node) = topology.get_node(node_id) {
            return node.memory_size;
        }
    }
    0
}

/// Retorna memória livre no nó (placeholder)
pub fn node_free_memory(node_id: u32) -> u64 {
    // TODO: Integrar com phys stats por nó
    node_memory_size(node_id) / 2 // Placeholder
}

/// Atualiza tamanho de memória do nó (chamado durante init)
pub fn set_node_memory(node_id: u32, start: u64, size: u64) {
    if let Some(ref mut topology) = *NUMA_TOPOLOGY.lock() {
        topology.set_node_memory(node_id, start, size);
    }
}

// =============================================================================
// Helpers para Alocador
// =============================================================================

/// Sugere nó para alocação com política específica
pub fn select_node(policy: NumaPolicyType, preferred: Option<u32>) -> u32 {
    match policy {
        NumaPolicyType::Local => current_node(),

        NumaPolicyType::Preferred => preferred.unwrap_or_else(current_node),

        NumaPolicyType::Interleave => {
            // Round-robin entre nós
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            count % NUM_NODES.load(Ordering::Relaxed)
        }

        NumaPolicyType::Bind => preferred.unwrap_or(0),
    }
}

/// Verifica se memória deve ser migrada para nó local
pub fn should_migrate(current_node_id: u32, target_node_id: u32) -> bool {
    if !is_numa() {
        return false;
    }

    // Migra se distância for grande
    let dist = distance(current_node_id, target_node_id);
    dist > 20 // Threshold arbitrário
}
