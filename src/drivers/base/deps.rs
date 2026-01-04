//! # Grafo de Dependências (Dependency Graph)
//!
//! Este módulo gerencia as **dependências entre drivers** para garantir
//! que a inicialização ocorra na ordem correta.
//!
//! ## Problema:
//! - Driver NVMe depende de PCI estar inicializado
//! - Driver USB Storage depende de xHCI estar pronto
//! - Driver de teclado USB depende de USB HID estar carregado
//!
//! ## Solução:
//! 1. Constrói grafo dirigido de dependências
//! 2. Faz ordenação topológica
//! 3. Inicializa na ordem correta
//!
//! ## Tipos de Dependência:
//! - **Hard**: Driver não funciona sem a dependência (erro)
//! - **Soft**: Driver pode funcionar, mas com funcionalidade reduzida (warning)
//!
//! ## STUB:
//! A implementação completa requer metadados de dependências nos drivers.
//! Por enquanto, fornece estruturas e funções básicas.

use super::device::DeviceId;
use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

// =============================================================================
// TIPOS DE DEPENDÊNCIA
// =============================================================================

/// Severidade da dependência.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    /// Dependência obrigatória.
    /// Driver falha se dependência não estiver satisfeita.
    Hard,

    /// Dependência opcional.
    /// Driver funciona com recursos reduzidos sem ela.
    Soft,
}

// =============================================================================
// NÓ DO GRAFO
// =============================================================================

/// Representa um nó (driver) no grafo de dependências.
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Nome do driver.
    pub name: &'static str,

    /// Lista de dependências (nome do driver de que depende).
    pub depends_on: Vec<(&'static str, DependencyType)>,

    /// Prioridade base (menor = inicializa primeiro se sem dependências).
    pub priority: u8,

    /// Flag marcando se já foi inicializado.
    pub initialized: bool,
}

// =============================================================================
// GRAFO DE DEPENDÊNCIAS
// =============================================================================

/// Gerenciador do grafo de dependências.
struct DependencyGraph {
    /// Nós do grafo.
    nodes: Vec<DependencyNode>,

    /// Flag de inicialização.
    initialized: bool,
}

static DEP_GRAPH: Spinlock<DependencyGraph> = Spinlock::new(DependencyGraph {
    nodes: Vec::new(),
    initialized: false,
});

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o sistema de dependências.
pub fn init() {
    crate::kinfo!("(Deps) Inicializando grafo de dependências...");

    let mut graph = DEP_GRAPH.lock();
    graph.initialized = true;

    // Registra dependências conhecidas (hard-coded por enquanto)
    register_builtin_dependencies(&mut graph);

    crate::kinfo!("(Deps) Sistema pronto com", graph.nodes.len(), "nós");
}

/// Registra dependências built-in conhecidas.
fn register_builtin_dependencies(graph: &mut DependencyGraph) {
    // PCI é a raiz de muita coisa
    graph.nodes.push(DependencyNode {
        name: "pci",
        depends_on: vec![],
        priority: 1, // Muito cedo
        initialized: false,
    });

    // AHCI depende de PCI
    graph.nodes.push(DependencyNode {
        name: "ahci",
        depends_on: vec![("pci", DependencyType::Hard)],
        priority: 10,
        initialized: false,
    });

    // NVMe depende de PCI
    graph.nodes.push(DependencyNode {
        name: "nvme",
        depends_on: vec![("pci", DependencyType::Hard)],
        priority: 10,
        initialized: false,
    });

    // xHCI (USB 3.0) depende de PCI
    graph.nodes.push(DependencyNode {
        name: "xhci",
        depends_on: vec![("pci", DependencyType::Hard)],
        priority: 10,
        initialized: false,
    });

    // USB HID depende de xHCI
    graph.nodes.push(DependencyNode {
        name: "usb-hid",
        depends_on: vec![("xhci", DependencyType::Hard)],
        priority: 20,
        initialized: false,
    });

    // USB Storage depende de xHCI
    graph.nodes.push(DependencyNode {
        name: "usb-storage",
        depends_on: vec![("xhci", DependencyType::Hard)],
        priority: 20,
        initialized: false,
    });

    // VirtIO depende de PCI (transport)
    graph.nodes.push(DependencyNode {
        name: "virtio",
        depends_on: vec![("pci", DependencyType::Hard)],
        priority: 10,
        initialized: false,
    });

    // VirtIO-Blk depende de VirtIO
    graph.nodes.push(DependencyNode {
        name: "virtio-blk",
        depends_on: vec![("virtio", DependencyType::Hard)],
        priority: 15,
        initialized: false,
    });

    // VirtIO-Net depende de VirtIO
    graph.nodes.push(DependencyNode {
        name: "virtio-net",
        depends_on: vec![("virtio", DependencyType::Hard)],
        priority: 15,
        initialized: false,
    });
}

/// Registra um novo nó no grafo.
pub fn register(node: DependencyNode) {
    let mut graph = DEP_GRAPH.lock();

    if graph.nodes.iter().any(|n| n.name == node.name) {
        crate::kwarn!("(Deps) Nó já existe:", node.name);
        return;
    }

    crate::kinfo!("(Deps) Nó registrado:", node.name);
    graph.nodes.push(node);
}

/// Retorna a ordem de inicialização correta (ordenação topológica).
///
/// ## STUB:
/// Implementação simplificada. Uma real usaria algoritmo de Kahn ou DFS.
pub fn get_init_order() -> Vec<&'static str> {
    crate::kwarn!("(Deps) get_init_order() usando ordenação por prioridade simples");

    let graph = DEP_GRAPH.lock();
    let mut order: Vec<_> = graph.nodes.iter().collect();

    // Ordena por prioridade (menor primeiro)
    order.sort_by_key(|n| n.priority);

    order.iter().map(|n| n.name).collect()
}

/// Verifica se todas as dependências de um driver estão satisfeitas.
pub fn check_dependencies(driver_name: &str) -> Result<(), Vec<&'static str>> {
    let graph = DEP_GRAPH.lock();

    let node = match graph.nodes.iter().find(|n| n.name == driver_name) {
        Some(n) => n,
        None => return Ok(()), // Não tem dependências registradas
    };

    let mut missing = Vec::new();

    for (dep_name, dep_type) in &node.depends_on {
        let dep_node = graph.nodes.iter().find(|n| n.name == *dep_name);

        let satisfied = dep_node.map(|n| n.initialized).unwrap_or(false);

        if !satisfied {
            match dep_type {
                DependencyType::Hard => {
                    crate::kerror!("(Deps) Dependência HARD faltando:", *dep_name);
                    missing.push(*dep_name);
                }
                DependencyType::Soft => {
                    crate::kwarn!("(Deps) Dependência soft faltando:", *dep_name);
                    // Soft não adiciona a missing
                }
            }
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

/// Marca um driver como inicializado.
pub fn mark_initialized(driver_name: &str) {
    let mut graph = DEP_GRAPH.lock();

    if let Some(node) = graph.nodes.iter_mut().find(|n| n.name == driver_name) {
        node.initialized = true;
        crate::kinfo!("(Deps) Marcado como inicializado:", driver_name);
    }
}

/// Retorna nós que dependem de um driver específico.
pub fn get_dependents(driver_name: &str) -> Vec<&'static str> {
    let graph = DEP_GRAPH.lock();

    graph
        .nodes
        .iter()
        .filter(|n| n.depends_on.iter().any(|(d, _)| *d == driver_name))
        .map(|n| n.name)
        .collect()
}

/// Imprime o grafo de dependências para debug.
pub fn print_graph() {
    let graph = DEP_GRAPH.lock();

    crate::kinfo!("=== Dependency Graph ===");
    for node in graph.nodes.iter() {
        let deps: Vec<_> = node.depends_on.iter().map(|(n, _)| *n).collect();
        let deps_str = if deps.is_empty() {
            "(none)".into()
        } else {
            deps.join(", ")
        };
        crate::kinfo!("  ", node.name, " -> [", "]"); // TODO: Format deps properly
    }
    crate::kinfo!("========================");
}
