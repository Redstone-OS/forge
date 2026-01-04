//! # Container
//!
//! Sandbox completo com namespaces e limites de recursos.

use super::namespace::NamespaceSet;

/// ID de container.
pub type ContainerId = u64;

/// Limites de recursos de um container.
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    /// Número máximo de processos.
    pub max_processes: u32,
    /// Memória máxima em bytes.
    pub max_memory: usize,
    /// Porcentagem máxima de CPU (0-100).
    pub max_cpu_percent: u8,
    /// Número máximo de arquivos abertos.
    pub max_files: u32,
    /// Número máximo de capabilities.
    pub max_caps: u32,
}

impl ResourceLimits {
    /// Limites padrão.
    pub const fn default_limits() -> Self {
        Self {
            max_processes: 256,
            max_memory: 256 * 1024 * 1024, // 256 MB
            max_cpu_percent: 100,
            max_files: 1024,
            max_caps: 256,
        }
    }

    /// Sem limites (para init/kernel).
    pub const fn unlimited() -> Self {
        Self {
            max_processes: u32::MAX,
            max_memory: usize::MAX,
            max_cpu_percent: 100,
            max_files: u32::MAX,
            max_caps: u32::MAX,
        }
    }

    /// Limites restritivos (para módulos não-confiáveis).
    pub const fn restricted() -> Self {
        Self {
            max_processes: 16,
            max_memory: 16 * 1024 * 1024, // 16 MB
            max_cpu_percent: 10,
            max_files: 64,
            max_caps: 32,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::default_limits()
    }
}

/// Container (sandbox completo).
///
/// Agrupa namespaces, capabilities e limites de recursos.
#[derive(Debug, Clone)]
pub struct Container {
    /// ID único.
    pub id: ContainerId,

    /// Namespaces do container.
    pub namespaces: NamespaceSet,

    /// Limites de recursos.
    pub limits: ResourceLimits,

    /// Container está ativo.
    pub active: bool,

    /// Número de processos dentro.
    pub process_count: u32,

    /// Memória usada.
    pub memory_used: usize,
}

impl Container {
    /// Container init (id 0, sem limites).
    pub const INIT_ID: ContainerId = 0;

    /// Cria container init.
    pub fn init() -> Self {
        Self {
            id: Self::INIT_ID,
            namespaces: NamespaceSet::init(),
            limits: ResourceLimits::unlimited(),
            active: true,
            process_count: 0,
            memory_used: 0,
        }
    }

    /// Cria novo container.
    pub fn new(id: ContainerId, limits: ResourceLimits) -> Self {
        Self {
            id,
            namespaces: NamespaceSet::init(),
            limits,
            active: true,
            process_count: 0,
            memory_used: 0,
        }
    }

    /// Cria container filho (herda namespaces do pai).
    pub fn fork(id: ContainerId, parent: &Container, limits: ResourceLimits) -> Self {
        Self {
            id,
            namespaces: NamespaceSet::inherit_from(&parent.namespaces),
            limits,
            active: true,
            process_count: 0,
            memory_used: 0,
        }
    }

    /// Verifica se é o container init.
    pub fn is_init(&self) -> bool {
        self.id == Self::INIT_ID
    }

    /// Verifica se pode criar mais processos.
    pub fn can_create_process(&self) -> bool {
        self.process_count < self.limits.max_processes
    }

    /// Verifica se pode alocar memória.
    pub fn can_allocate(&self, size: usize) -> bool {
        self.memory_used.saturating_add(size) <= self.limits.max_memory
    }

    /// Registra uso de memória.
    pub fn track_memory(&mut self, delta: isize) {
        if delta > 0 {
            self.memory_used = self.memory_used.saturating_add(delta as usize);
        } else {
            self.memory_used = self.memory_used.saturating_sub((-delta) as usize);
        }
    }

    /// Registra criação de processo.
    pub fn add_process(&mut self) -> bool {
        if self.can_create_process() {
            self.process_count += 1;
            true
        } else {
            false
        }
    }

    /// Registra término de processo.
    pub fn remove_process(&mut self) {
        self.process_count = self.process_count.saturating_sub(1);
    }
}
