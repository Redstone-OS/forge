//! # NUMA Policies
//!
//! Políticas de alocação de memória para sistemas NUMA.
//!
//! ## Tipos de Política
//!
//! - **Local**: Aloca do nó da CPU executando (padrão e mais rápido)
//! - **Preferred**: Prefere um nó, mas aceita outros se necessário
//! - **Interleave**: Distribui páginas round-robin entre nós
//! - **Bind**: Força alocação de nó específico, falha se não houver memória
//!
//! ## Uso
//!
//! ```rust
//! // Política local (padrão)
//! let policy = NumaPolicy::local();
//!
//! // Preferir nó 1
//! let policy = NumaPolicy::preferred(1);
//!
//! // Interleave entre nós 0 e 1
//! let policy = NumaPolicy::interleave(&[0, 1]);
//!
//! // Bind estrito ao nó 0
//! let policy = NumaPolicy::bind(0);
//! ```

use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

// =============================================================================
// NumaPolicyType
// =============================================================================

/// Tipo de política NUMA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPolicyType {
    /// Aloca do nó local (padrão)
    Local,
    /// Prefere um nó, fallback para outros
    Preferred,
    /// Distribui páginas entre nós (bandwidth)
    Interleave,
    /// Força alocação de nó específico
    Bind,
}

impl Default for NumaPolicyType {
    fn default() -> Self {
        Self::Local
    }
}

// =============================================================================
// NumaPolicy
// =============================================================================

/// Política de alocação NUMA
#[derive(Debug)]
pub struct NumaPolicy {
    /// Tipo de política
    pub policy_type: NumaPolicyType,
    /// Nó preferido (para Preferred e Bind)
    pub preferred_node: Option<u32>,
    /// Nós para interleave
    pub interleave_nodes: Vec<u32>,
    /// Contador para round-robin em interleave
    interleave_counter: AtomicU32,
    /// Modo estrito (falha se não conseguir do nó desejado)
    pub strict: bool,
}

impl Clone for NumaPolicy {
    fn clone(&self) -> Self {
        Self {
            policy_type: self.policy_type,
            preferred_node: self.preferred_node,
            interleave_nodes: self.interleave_nodes.clone(),
            interleave_counter: AtomicU32::new(self.interleave_counter.load(Ordering::Relaxed)),
            strict: self.strict,
        }
    }
}

impl NumaPolicy {
    /// Cria política local (padrão)
    pub fn local() -> Self {
        Self {
            policy_type: NumaPolicyType::Local,
            preferred_node: None,
            interleave_nodes: Vec::new(),
            interleave_counter: AtomicU32::new(0),
            strict: false,
        }
    }

    /// Cria política de preferência
    pub fn preferred(node_id: u32) -> Self {
        Self {
            policy_type: NumaPolicyType::Preferred,
            preferred_node: Some(node_id),
            interleave_nodes: Vec::new(),
            interleave_counter: AtomicU32::new(0),
            strict: false,
        }
    }

    /// Cria política interleave
    pub fn interleave(nodes: &[u32]) -> Self {
        Self {
            policy_type: NumaPolicyType::Interleave,
            preferred_node: None,
            interleave_nodes: nodes.to_vec(),
            interleave_counter: AtomicU32::new(0),
            strict: false,
        }
    }

    /// Cria política bind (estrita)
    pub fn bind(node_id: u32) -> Self {
        Self {
            policy_type: NumaPolicyType::Bind,
            preferred_node: Some(node_id),
            interleave_nodes: Vec::new(),
            interleave_counter: AtomicU32::new(0),
            strict: true,
        }
    }

    /// Define se política é estrita
    pub fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    /// Seleciona próximo nó para alocação
    ///
    /// # Arguments
    ///
    /// * `current_node` - Nó atual da CPU
    ///
    /// # Returns
    ///
    /// ID do nó onde alocar
    pub fn select_node(&self, current_node: u32) -> u32 {
        match self.policy_type {
            NumaPolicyType::Local => current_node,

            NumaPolicyType::Preferred => self.preferred_node.unwrap_or(current_node),

            NumaPolicyType::Interleave => {
                if self.interleave_nodes.is_empty() {
                    return current_node;
                }

                let idx = self.interleave_counter.fetch_add(1, Ordering::Relaxed);
                let node_idx = idx as usize % self.interleave_nodes.len();
                self.interleave_nodes[node_idx]
            }

            NumaPolicyType::Bind => self.preferred_node.unwrap_or(0),
        }
    }

    /// Lista de nós a tentar em ordem de preferência
    ///
    /// Usado para fallback quando primeiro nó não tem memória.
    pub fn fallback_nodes(&self, current_node: u32, total_nodes: u32) -> Vec<u32> {
        let primary = self.select_node(current_node);

        match self.policy_type {
            NumaPolicyType::Local | NumaPolicyType::Preferred => {
                // Começa no nó selecionado, depois tenta outros
                let mut nodes = vec![primary];
                for n in 0..total_nodes {
                    if n != primary {
                        nodes.push(n);
                    }
                }
                nodes
            }

            NumaPolicyType::Interleave => {
                // Todos os nós de interleave, depois outros
                let mut nodes = self.interleave_nodes.clone();
                for n in 0..total_nodes {
                    if !nodes.contains(&n) {
                        nodes.push(n);
                    }
                }
                nodes
            }

            NumaPolicyType::Bind => {
                // Bind estrito: apenas o nó especificado
                if self.strict {
                    vec![primary]
                } else {
                    // Fallback se não estrito
                    let mut nodes = vec![primary];
                    for n in 0..total_nodes {
                        if n != primary {
                            nodes.push(n);
                        }
                    }
                    nodes
                }
            }
        }
    }

    /// Verifica se alocação em nó específico é aceitável
    pub fn accepts_node(&self, node_id: u32) -> bool {
        match self.policy_type {
            NumaPolicyType::Local | NumaPolicyType::Preferred => true,

            NumaPolicyType::Interleave => {
                self.interleave_nodes.is_empty() || self.interleave_nodes.contains(&node_id)
            }

            NumaPolicyType::Bind => !self.strict || self.preferred_node == Some(node_id),
        }
    }

    /// Reseta contador de interleave
    pub fn reset_interleave(&self) {
        self.interleave_counter.store(0, Ordering::Relaxed);
    }
}

impl Default for NumaPolicy {
    fn default() -> Self {
        Self::local()
    }
}

// =============================================================================
// Process NUMA State
// =============================================================================

/// Estado NUMA de um processo
///
/// Cada processo pode ter sua própria política NUMA.
#[derive(Debug, Clone)]
pub struct ProcessNumaState {
    /// Política de alocação de memória
    pub memory_policy: NumaPolicy,
    /// Política de afinidade de CPU (quais nós podem executar)
    pub cpu_affinity: Vec<u32>,
    /// Nó home (onde processo foi criado)
    pub home_node: u32,
    /// Migração automática habilitada?
    pub auto_migrate: bool,
}

impl ProcessNumaState {
    /// Cria estado padrão
    pub fn new(home_node: u32) -> Self {
        Self {
            memory_policy: NumaPolicy::local(),
            cpu_affinity: Vec::new(), // Vazio = qualquer CPU
            home_node,
            auto_migrate: false,
        }
    }

    /// Define política de memória
    pub fn set_memory_policy(&mut self, policy: NumaPolicy) {
        self.memory_policy = policy;
    }

    /// Define afinidade de CPU
    pub fn set_cpu_affinity(&mut self, nodes: Vec<u32>) {
        self.cpu_affinity = nodes;
    }

    /// Habilita migração automática
    pub fn enable_auto_migrate(&mut self) {
        self.auto_migrate = true;
    }

    /// Pode executar em nó?
    pub fn can_run_on(&self, node_id: u32) -> bool {
        self.cpu_affinity.is_empty() || self.cpu_affinity.contains(&node_id)
    }
}

impl Default for ProcessNumaState {
    fn default() -> Self {
        Self::new(0)
    }
}

// =============================================================================
// Syscall Interface (stubs)
// =============================================================================

/// Flags para mbind syscall
#[derive(Clone, Copy)]
pub struct MbindFlags(u32);

impl MbindFlags {
    pub const NONE: Self = Self(0);
    /// Move páginas existentes
    pub const MOVE: Self = Self(1 << 0);
    /// Move todas as páginas (inclui shared)
    pub const MOVE_ALL: Self = Self(1 << 1);
    /// Modo estrito
    pub const STRICT: Self = Self(1 << 2);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// mbind syscall - define política para região de memória
///
/// Stub para futura implementação de syscall real.
pub fn sys_mbind(
    _start: u64,
    _len: usize,
    _policy: NumaPolicyType,
    _nodes: &[u32],
    _flags: MbindFlags,
) -> Result<(), i32> {
    // TODO: Implementar
    Ok(())
}

/// get_mempolicy syscall - retorna política atual
pub fn sys_get_mempolicy(_addr: Option<u64>) -> Result<(NumaPolicyType, Vec<u32>), i32> {
    // TODO: Implementar
    Ok((NumaPolicyType::Local, vec![0]))
}

/// set_mempolicy syscall - define política padrão do processo
pub fn sys_set_mempolicy(_policy: NumaPolicyType, _nodes: &[u32]) -> Result<(), i32> {
    // TODO: Implementar
    Ok(())
}

/// migrate_pages syscall - migra páginas entre nós
pub fn sys_migrate_pages(_pid: u32, _from_nodes: &[u32], _to_nodes: &[u32]) -> Result<usize, i32> {
    // TODO: Implementar
    Ok(0)
}
