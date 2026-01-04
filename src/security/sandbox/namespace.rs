//! # Namespaces
//!
//! Isolamento de recursos do sistema.
//! Inspirado nos Linux namespaces.

/// Tipo de namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NamespaceType {
    /// Pontos de montagem do filesystem.
    Mount = 0,
    /// Espaço de PIDs (processo vê PID 1 como init).
    Pid = 1,
    /// Stack de rede isolada.
    Net = 2,
    /// Recursos IPC (semáforos, filas).
    Ipc = 3,
    /// Hostname e domínio.
    Uts = 4,
}

impl NamespaceType {
    /// Nome do tipo.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Mount => "mount",
            Self::Pid => "pid",
            Self::Net => "net",
            Self::Ipc => "ipc",
            Self::Uts => "uts",
        }
    }
}

/// ID de um namespace.
pub type NamespaceId = u64;

/// Um namespace específico.
#[derive(Debug, Clone)]
pub struct Namespace {
    /// Tipo do namespace.
    pub ns_type: NamespaceType,
    /// ID único.
    pub id: NamespaceId,
    /// Número de referências.
    ref_count: u32,
}

impl Namespace {
    /// Cria novo namespace.
    pub fn new(ns_type: NamespaceType, id: NamespaceId) -> Self {
        Self {
            ns_type,
            id,
            ref_count: 1,
        }
    }

    /// ID global (init namespace).
    pub const INIT_ID: NamespaceId = 0;

    /// Cria namespace init.
    pub fn init(ns_type: NamespaceType) -> Self {
        Self::new(ns_type, Self::INIT_ID)
    }

    /// Incrementa referência.
    pub fn add_ref(&mut self) {
        self.ref_count += 1;
    }

    /// Decrementa referência.
    pub fn release(&mut self) -> bool {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count == 0
    }
}

/// Conjunto de namespaces de um processo.
#[derive(Debug, Clone)]
pub struct NamespaceSet {
    /// Namespace de mount.
    pub mount: NamespaceId,
    /// Namespace de PID.
    pub pid: NamespaceId,
    /// Namespace de rede.
    pub net: NamespaceId,
    /// Namespace de IPC.
    pub ipc: NamespaceId,
    /// Namespace UTS.
    pub uts: NamespaceId,
}

impl NamespaceSet {
    /// Set inicial (todos no namespace 0).
    pub const fn init() -> Self {
        Self {
            mount: Namespace::INIT_ID,
            pid: Namespace::INIT_ID,
            net: Namespace::INIT_ID,
            ipc: Namespace::INIT_ID,
            uts: Namespace::INIT_ID,
        }
    }

    /// Retorna ID do namespace pelo tipo.
    pub fn get(&self, ns_type: NamespaceType) -> NamespaceId {
        match ns_type {
            NamespaceType::Mount => self.mount,
            NamespaceType::Pid => self.pid,
            NamespaceType::Net => self.net,
            NamespaceType::Ipc => self.ipc,
            NamespaceType::Uts => self.uts,
        }
    }

    /// Define namespace pelo tipo.
    pub fn set(&mut self, ns_type: NamespaceType, id: NamespaceId) {
        match ns_type {
            NamespaceType::Mount => self.mount = id,
            NamespaceType::Pid => self.pid = id,
            NamespaceType::Net => self.net = id,
            NamespaceType::Ipc => self.ipc = id,
            NamespaceType::Uts => self.uts = id,
        }
    }

    /// Herda de outro set (clone para fork).
    pub fn inherit_from(parent: &Self) -> Self {
        parent.clone()
    }

    /// Verifica se está no namespace init.
    pub fn is_init(&self) -> bool {
        self.mount == Namespace::INIT_ID
            && self.pid == Namespace::INIT_ID
            && self.net == Namespace::INIT_ID
            && self.ipc == Namespace::INIT_ID
            && self.uts == Namespace::INIT_ID
    }
}

impl Default for NamespaceSet {
    fn default() -> Self {
        Self::init()
    }
}
