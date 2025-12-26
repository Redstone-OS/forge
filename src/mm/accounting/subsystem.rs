//! # Subsistemas de Memória
//!
//! Define os subsistemas conhecidos e gerencia o contexto atual.

use core::sync::atomic::{AtomicU8, Ordering};

// =============================================================================
// DEFINIÇÃO DE SUBSISTEMAS
// =============================================================================

/// Identificadores de subsistemas do kernel
///
/// Cada subsistema tem suas próprias estatísticas e quota.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Subsystem {
    /// Código central do kernel
    Kernel = 0,
    /// Scheduler e tasks
    Scheduler = 1,
    /// Sistema de arquivos virtual
    VFS = 2,
    /// Stack de rede
    Network = 3,
    /// Drivers de dispositivo
    Drivers = 4,
    /// Processos de usuário (genérico)
    UserProcess = 5,
    /// IPC (pipes, sockets, etc)
    IPC = 6,
    /// Gerenciador de memória (meta)
    MemoryManager = 7,
    /// Subsistema gráfico
    Graphics = 8,
    /// Áudio
    Audio = 9,
    /// USB
    USB = 10,
    /// Storage (NVMe, AHCI)
    Storage = 11,
    /// Criptografia
    Crypto = 12,
    /// Temporário / teste
    Test = 253,
    /// Debug
    Debug = 254,
    /// Desconhecido
    Unknown = 255,
}

impl Subsystem {
    /// Converte de u8
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Kernel,
            1 => Self::Scheduler,
            2 => Self::VFS,
            3 => Self::Network,
            4 => Self::Drivers,
            5 => Self::UserProcess,
            6 => Self::IPC,
            7 => Self::MemoryManager,
            8 => Self::Graphics,
            9 => Self::Audio,
            10 => Self::USB,
            11 => Self::Storage,
            12 => Self::Crypto,
            253 => Self::Test,
            254 => Self::Debug,
            _ => Self::Unknown,
        }
    }

    /// Nome legível
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kernel => "Kernel",
            Self::Scheduler => "Scheduler",
            Self::VFS => "VFS",
            Self::Network => "Network",
            Self::Drivers => "Drivers",
            Self::UserProcess => "UserProcess",
            Self::IPC => "IPC",
            Self::MemoryManager => "MemoryManager",
            Self::Graphics => "Graphics",
            Self::Audio => "Audio",
            Self::USB => "USB",
            Self::Storage => "Storage",
            Self::Crypto => "Crypto",
            Self::Test => "Test",
            Self::Debug => "Debug",
            Self::Unknown => "Unknown",
        }
    }

    /// Quota padrão em bytes (0 = sem limite)
    pub fn default_quota(&self) -> usize {
        match self {
            Self::Kernel => 0,                   // Sem limite
            Self::Scheduler => 64 * 1024 * 1024, // 64 MB
            Self::VFS => 128 * 1024 * 1024,      // 128 MB
            Self::Network => 256 * 1024 * 1024,  // 256 MB
            Self::Drivers => 128 * 1024 * 1024,  // 128 MB
            Self::UserProcess => 0,              // Gerenciado por processo
            Self::Test => 16 * 1024 * 1024,      // 16 MB
            _ => 0,
        }
    }

    /// Todos os subsistemas conhecidos
    pub fn all() -> &'static [Self] {
        &[
            Self::Kernel,
            Self::Scheduler,
            Self::VFS,
            Self::Network,
            Self::Drivers,
            Self::UserProcess,
            Self::IPC,
            Self::MemoryManager,
            Self::Graphics,
            Self::Audio,
            Self::USB,
            Self::Storage,
            Self::Crypto,
        ]
    }
}

// =============================================================================
// CONTEXTO ATUAL
// =============================================================================

/// Subsistema atual (thread-local simulado)
///
/// TODO: Quando tivermos per-CPU storage ou TLS, usar isso.
/// Por enquanto, mantemos um valor global.
static CURRENT_SUBSYSTEM: AtomicU8 = AtomicU8::new(Subsystem::Kernel as u8);

/// Obtém o subsistema atual
#[inline]
pub fn get_current_subsystem() -> Subsystem {
    Subsystem::from_u8(CURRENT_SUBSYSTEM.load(Ordering::Relaxed))
}

/// Define o subsistema atual
///
/// Deve ser chamado ao entrar em código de um subsistema específico.
#[inline]
pub fn set_current_subsystem(subsys: Subsystem) {
    CURRENT_SUBSYSTEM.store(subsys as u8, Ordering::Relaxed);
}

/// RAII guard para subsistema
///
/// Restaura o subsistema anterior quando sai de escopo.
pub struct SubsystemGuard {
    previous: Subsystem,
}

impl SubsystemGuard {
    /// Entra em um subsistema, guardando o anterior
    pub fn enter(subsys: Subsystem) -> Self {
        let previous = get_current_subsystem();
        set_current_subsystem(subsys);
        Self { previous }
    }
}

impl Drop for SubsystemGuard {
    fn drop(&mut self) {
        set_current_subsystem(self.previous);
    }
}

/// Macro para executar código em contexto de subsistema
///
/// ```rust
/// with_subsystem!(Subsystem::Network, {
///     // Código executado com subsistema Network
/// });
/// ```
#[macro_export]
macro_rules! with_subsystem {
    ($subsys:expr, $code:block) => {{
        let _guard = $crate::mm::accounting::subsystem::SubsystemGuard::enter($subsys);
        $code
    }};
}
