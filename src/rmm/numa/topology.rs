//! # NUMA Topology
//!
//! Representação da topologia NUMA do sistema.
//!
//! ## Estrutura
//!
//! A topologia é descoberta via ACPI:
//! - **SRAT (System Resource Affinity Table)**: CPU e memória por nó
//! - **SLIT (System Locality Information Table)**: Distâncias entre nós

use alloc::vec::Vec;

// =============================================================================
// NumaNode
// =============================================================================

/// Representa um nó NUMA
#[derive(Debug, Clone)]
pub struct NumaNode {
    /// ID do nó (0-based)
    pub id: u32,
    /// Endereço físico inicial da memória local
    pub memory_start: u64,
    /// Tamanho da memória local em bytes
    pub memory_size: u64,
    /// Bitmask de CPUs neste nó
    pub cpu_mask: u64,
    /// Número de CPUs neste nó
    pub cpu_count: u32,
    /// Distâncias para outros nós (índice = node_id)
    pub distance_to: Vec<u32>,
}

impl NumaNode {
    /// Cria nó vazio
    pub fn new(id: u32) -> Self {
        Self {
            id,
            memory_start: 0,
            memory_size: 0,
            cpu_mask: 0,
            cpu_count: 0,
            distance_to: Vec::new(),
        }
    }

    /// Verifica se CPU pertence a este nó
    #[inline]
    pub fn has_cpu(&self, cpu_id: u32) -> bool {
        if cpu_id >= 64 {
            return false;
        }
        (self.cpu_mask & (1 << cpu_id)) != 0
    }

    /// Adiciona CPU a este nó
    pub fn add_cpu(&mut self, cpu_id: u32) {
        if cpu_id < 64 {
            self.cpu_mask |= 1 << cpu_id;
            self.cpu_count += 1;
        }
    }

    /// Verifica se endereço físico está na memória deste nó
    #[inline]
    pub fn contains_addr(&self, phys: u64) -> bool {
        phys >= self.memory_start && phys < self.memory_start + self.memory_size
    }

    /// Retorna fim da memória
    #[inline]
    pub fn memory_end(&self) -> u64 {
        self.memory_start + self.memory_size
    }

    /// Distância para outro nó
    pub fn distance_to_node(&self, other_id: u32) -> u32 {
        if other_id as usize >= self.distance_to.len() {
            return if other_id == self.id { 10 } else { 20 };
        }
        self.distance_to[other_id as usize]
    }

    /// Itera sobre CPUs deste nó
    pub fn cpus(&self) -> impl Iterator<Item = u32> + '_ {
        (0..64u32).filter(move |&cpu| self.has_cpu(cpu))
    }
}

// =============================================================================
// NumaTopology
// =============================================================================

/// Topologia NUMA do sistema
#[derive(Debug, Clone)]
pub struct NumaTopology {
    /// Nós NUMA
    nodes: Vec<NumaNode>,
    /// Matriz de distâncias (redundante mas otimiza lookup)
    distance_matrix: Vec<Vec<u32>>,
    /// Número total de CPUs
    total_cpus: u32,
    /// Memória total em bytes
    total_memory: u64,
}

impl NumaTopology {
    /// Cria topologia vazia
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            distance_matrix: Vec::new(),
            total_cpus: 0,
            total_memory: 0,
        }
    }

    /// Adiciona nó à topologia
    pub fn add_node(&mut self, node: NumaNode) {
        self.total_cpus += node.cpu_count;
        self.total_memory += node.memory_size;
        self.nodes.push(node);

        // Atualiza matriz de distâncias
        self.rebuild_distance_matrix();
    }

    /// Reconstrói matriz de distâncias
    fn rebuild_distance_matrix(&mut self) {
        let n = self.nodes.len();
        self.distance_matrix = vec![vec![20; n]; n];

        for node in &self.nodes {
            let i = node.id as usize;
            if i < n {
                for (j, &dist) in node.distance_to.iter().enumerate() {
                    if j < n {
                        self.distance_matrix[i][j] = dist;
                    }
                }
            }
        }
    }

    /// Número de nós
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total de CPUs
    #[inline]
    pub fn total_cpus(&self) -> u32 {
        self.total_cpus
    }

    /// Memória total
    #[inline]
    pub fn total_memory(&self) -> u64 {
        self.total_memory
    }

    /// Retorna nó por ID
    pub fn get_node(&self, node_id: u32) -> Option<&NumaNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    /// Retorna nó mutável por ID
    pub fn get_node_mut(&mut self, node_id: u32) -> Option<&mut NumaNode> {
        self.nodes.iter_mut().find(|n| n.id == node_id)
    }

    /// Itera sobre todos os nós
    pub fn nodes(&self) -> impl Iterator<Item = &NumaNode> {
        self.nodes.iter()
    }

    /// Retorna nó que contém endereço físico
    pub fn node_for_addr(&self, phys: u64) -> u32 {
        for node in &self.nodes {
            if node.contains_addr(phys) {
                return node.id;
            }
        }
        0 // Default: nó 0
    }

    /// Retorna nó que contém CPU
    pub fn node_for_cpu(&self, cpu_id: u32) -> u32 {
        for node in &self.nodes {
            if node.has_cpu(cpu_id) {
                return node.id;
            }
        }
        0 // Default: nó 0
    }

    /// Distância entre dois nós
    pub fn distance(&self, from: u32, to: u32) -> u32 {
        let from_idx = from as usize;
        let to_idx = to as usize;

        if from_idx < self.distance_matrix.len() && to_idx < self.distance_matrix[from_idx].len() {
            return self.distance_matrix[from_idx][to_idx];
        }

        if from == to {
            10
        } else {
            20
        }
    }

    /// Retorna nós ordenados por distância de um nó
    pub fn nodes_by_distance(&self, from_node: u32) -> Vec<u32> {
        let mut node_ids: Vec<u32> = self.nodes.iter().map(|n| n.id).collect();

        node_ids.sort_by_key(|&node_id| self.distance(from_node, node_id));

        node_ids
    }

    /// Define memória de um nó
    pub fn set_node_memory(&mut self, node_id: u32, start: u64, size: u64) {
        // Atualiza total
        if let Some(node) = self.get_node(node_id) {
            self.total_memory -= node.memory_size;
        }

        if let Some(node) = self.get_node_mut(node_id) {
            node.memory_start = start;
            node.memory_size = size;
        }

        self.total_memory += size;
    }

    /// Adiciona CPU a um nó
    pub fn add_cpu_to_node(&mut self, node_id: u32, cpu_id: u32) {
        if let Some(node) = self.get_node_mut(node_id) {
            if !node.has_cpu(cpu_id) {
                node.add_cpu(cpu_id);
                self.total_cpus += 1;
            }
        }
    }

    /// Define distância entre dois nós
    pub fn set_distance(&mut self, from: u32, to: u32, distance: u32) {
        // Atualiza no nó
        if let Some(node) = self.get_node_mut(from) {
            let to_idx = to as usize;
            while node.distance_to.len() <= to_idx {
                node.distance_to.push(20);
            }
            node.distance_to[to_idx] = distance;
        }

        // Atualiza matriz
        let from_idx = from as usize;
        let to_idx = to as usize;
        if from_idx < self.distance_matrix.len() {
            while self.distance_matrix[from_idx].len() <= to_idx {
                self.distance_matrix[from_idx].push(20);
            }
            self.distance_matrix[from_idx][to_idx] = distance;
        }
    }

    /// Nó com mais memória livre (para balanceamento)
    pub fn node_with_most_free_memory(&self) -> u32 {
        // TODO: Integrar com stats reais por nó
        self.nodes
            .iter()
            .max_by_key(|n| n.memory_size)
            .map_or(0, |n| n.id)
    }

    /// Verifica se topologia é válida
    pub fn is_valid(&self) -> bool {
        !self.nodes.is_empty() && self.total_memory > 0
    }
}

impl Default for NumaTopology {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Parsing ACPI (stubs)
// =============================================================================

/// Informação de afinidade de memória (do SRAT)
#[derive(Debug, Clone)]
pub struct MemoryAffinity {
    pub base_addr: u64,
    pub length: u64,
    pub proximity_domain: u32,
    pub enabled: bool,
    pub hot_pluggable: bool,
}

/// Informação de afinidade de CPU (do SRAT)
#[derive(Debug, Clone)]
pub struct CpuAffinity {
    pub apic_id: u32,
    pub proximity_domain: u32,
    pub enabled: bool,
}

/// Parseia SRAT table
///
/// TODO: Implementar parsing real de ACPI
pub fn parse_srat(_srat_addr: u64) -> Option<(Vec<CpuAffinity>, Vec<MemoryAffinity>)> {
    // Stub - seria implementado com parsing ACPI
    None
}

/// Parseia SLIT table
///
/// TODO: Implementar parsing real de ACPI
pub fn parse_slit(_slit_addr: u64, _node_count: usize) -> Option<Vec<Vec<u32>>> {
    // Stub - seria implementado com parsing ACPI
    None
}

/// Constrói topologia a partir de ACPI tables
pub fn build_from_acpi(srat_addr: Option<u64>, slit_addr: Option<u64>) -> Option<NumaTopology> {
    let mut topology = NumaTopology::new();

    // Parse SRAT
    if let Some(addr) = srat_addr {
        if let Some((cpus, memories)) = parse_srat(addr) {
            // Cria nós únicos
            let mut node_ids: Vec<u32> = cpus.iter().map(|c| c.proximity_domain).collect();
            node_ids.extend(memories.iter().map(|m| m.proximity_domain));
            node_ids.sort();
            node_ids.dedup();

            for &id in &node_ids {
                topology.add_node(NumaNode::new(id));
            }

            // Adiciona CPUs
            for cpu in cpus {
                if cpu.enabled {
                    topology.add_cpu_to_node(cpu.proximity_domain, cpu.apic_id);
                }
            }

            // Adiciona memória
            for mem in memories {
                if mem.enabled {
                    topology.set_node_memory(mem.proximity_domain, mem.base_addr, mem.length);
                }
            }
        }
    }

    // Parse SLIT (distâncias)
    if let Some(addr) = slit_addr {
        if let Some(distances) = parse_slit(addr, topology.node_count()) {
            for (from, row) in distances.iter().enumerate() {
                for (to, &dist) in row.iter().enumerate() {
                    topology.set_distance(from as u32, to as u32, dist);
                }
            }
        }
    }

    if topology.is_valid() {
        Some(topology)
    } else {
        None
    }
}
