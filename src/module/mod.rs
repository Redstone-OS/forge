//! # Kernel Module System
//!
//! Carregamento seguro de módulos dinâmicos (drivers).
//!
//! ## Filosofia: "Convidado com Crachá"
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                 KERNEL CORE (Ring 0)                │
//! │        MM │ Scheduler │ IPC │ Security              │
//! │                 🔒 ZONA SAGRADA 🔒                  │
//! │           Módulos NÃO acessam diretamente           │
//! └─────────────────────────────────────────────────────┘
//!                         ↑
//!                  Capability Tokens
//!                         ↑
//! ┌─────────────────────────────────────────────────────┐
//! │              MODULE SUPERVISOR                      │
//! │   Loader │ Verifier │ Sandbox │ Watchdog            │
//! │          Único ponto de entrada                     │
//! └─────────────────────────────────────────────────────┘
//!                         ↑
//!                    Module ABI
//!                         ↑
//! ┌───────────┐ ┌───────────┐ ┌───────────┐
//! │ nvidia.ko │ │ e1000.ko  │ │ nvme.ko   │
//! └───────────┘ └───────────┘ └───────────┘
//! ```
//!
//! ## Fluxo de Carga
//!
//! 1. Verificar assinatura (Ed25519)
//! 2. Análise estática (símbolos permitidos)
//! 3. Alocação de memória (RX/RW separados)
//! 4. Concessão de capabilities
//! 5. Inicialização supervisionada (timeout)
//! 6. Monitoramento por watchdog

// =============================================================================
// MODULES
// =============================================================================

/// Interface binária estável para módulos
pub mod abi;

/// Capabilities específicas de módulos
pub mod capability;

/// Carregador ELF
pub mod loader;

/// Sandbox e isolamento
pub mod sandbox;

/// Supervisor de ciclo de vida
pub mod supervisor;

/// Verificação de assinatura
pub mod verifier;

/// Watchdog de saúde
pub mod watchdog;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use abi::{ModuleAbi, ModuleInfo};
pub use capability::{ModuleCapType, ModuleCapability};
pub use loader::ModuleLoader;
pub use sandbox::ModuleSandbox;
pub use supervisor::{LoadedModule, ModuleId, ModuleSupervisor, SUPERVISOR};
pub use verifier::SignatureVerifier;
pub use watchdog::{HealthStatus, ModuleWatchdog};

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Erros do sistema de módulos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleError {
    /// Módulo não encontrado
    NotFound,
    /// Assinatura inválida
    InvalidSignature,
    /// Formato inválido
    InvalidFormat,
    /// IOMMU necessário mas indisponível
    IommuRequired,
    /// Capability negada
    CapabilityDenied,
    /// Já carregado
    AlreadyLoaded,
    /// Limite atingido
    LimitReached,
    /// Timeout na inicialização
    InitTimeout,
    /// Erro interno
    InternalError,
    /// Módulo banido
    Banned,
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Inicializa o sistema de módulos
pub fn init() {
    crate::kinfo!("(Module) Inicializando supervisor...");
    SUPERVISOR.lock().init();
    crate::kinfo!("(Module) Sistema de módulos inicializado");
}

/// Carrega um módulo
pub fn load(path: &str) -> Result<ModuleId, ModuleError> {
    SUPERVISOR.lock().load_module(path)
}

/// Descarrega um módulo
pub fn unload(id: ModuleId) -> Result<(), ModuleError> {
    SUPERVISOR.lock().unload_module(id)
}

/// Lista módulos carregados
pub fn list() -> alloc::vec::Vec<ModuleId> {
    SUPERVISOR.lock().list_modules()
}

/// Verifica se IOMMU está disponível
pub fn has_iommu() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        crate::arch::x86_64::iommu::is_available()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
