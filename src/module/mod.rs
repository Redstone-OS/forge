//! # Kernel Module System
//!
//! Sistema de carregamento seguro de módulos de kernel (drivers privilegiados).
//!
//! ## Filosofia: "Convidado com Crachá"
//!
//! Módulos rodam em Ring 0 mas são **supervisionados**. Eles não podem:
//! - Acessar page tables globais
//! - Fazer DMA sem IOMMU
//! - Modificar syscall table do kernel
//! - Carregar outros módulos
//! - Acessar memória de outros processos
//!
//! ## Componentes
//!
//! | Módulo | Responsabilidade |
//! |--------|------------------|
//! | `supervisor` | Gerencia ciclo de vida dos módulos |
//! | `loader` | Carrega ELF de módulos |
//! | `verifier` | Verifica assinaturas |
//! | `capability` | Capabilities específicas de módulos |
//! | `sandbox` | Isolamento e proteção |
//! | `watchdog` | Monitora saúde dos módulos |
//! | `abi` | Interface estável para módulos |
//!
//! ## Uso
//!
//! ```ignore
//! // Carregar módulo (apenas kernel pode fazer isso)
//! let module_id = module::load("/drivers/nvidia.ko")?;
//!
//! // Módulo solicita capability
//! let cap = module::request_capability(ModuleCapType::GpuControl)?;
//!
//! // Se módulo falhar, kernel faz fallback
//! module::on_fault(module_id, |_| FallbackAction::UseBuiltIn);
//! ```

pub mod abi;
pub mod capability;
pub mod loader;
pub mod sandbox;
pub mod supervisor;
pub mod verifier;
pub mod watchdog;

// Re-exports
pub use abi::{ModuleAbi, ModuleInfo};
pub use capability::{ModuleCapType, ModuleCapability};
pub use loader::ModuleLoader;
pub use sandbox::ModuleSandbox;
pub use supervisor::{LoadedModule, ModuleId, ModuleSupervisor, SUPERVISOR};
pub use verifier::SignatureVerifier;
pub use watchdog::{HealthStatus, ModuleWatchdog};

/// Inicializa o sistema de módulos.
///
/// Deve ser chamado após MM e antes do PID1.
pub fn init() {
    crate::kinfo!("(Module) Inicializando sistema de módulos...");

    // Inicializar supervisor
    SUPERVISOR.lock().init();

    crate::kdebug!("(Module) Supervisor inicializado");
    if has_iommu() {
        crate::kdebug!("(Module) IOMMU: Disponivel");
    } else {
        crate::kdebug!("(Module) IOMMU: Indisponivel");
    }
}

/// Verifica se IOMMU está disponível no sistema.
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

/// Carrega um módulo do sistema de arquivos.
///
/// # Arguments
/// * `path` - Caminho do módulo (ex: "/drivers/nvidia.ko")
///
/// # Returns
/// * `Ok(ModuleId)` - ID do módulo carregado
/// * `Err(ModuleError)` - Se falhou
pub fn load(path: &str) -> Result<ModuleId, ModuleError> {
    SUPERVISOR.lock().load_module(path)
}

/// Descarrega um módulo.
pub fn unload(id: ModuleId) -> Result<(), ModuleError> {
    SUPERVISOR.lock().unload_module(id)
}

/// Lista módulos carregados.
pub fn list_modules() -> alloc::vec::Vec<ModuleId> {
    SUPERVISOR.lock().list_modules()
}

/// Erros do sistema de módulos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleError {
    /// Módulo não encontrado
    NotFound,
    /// Assinatura inválida
    InvalidSignature,
    /// Módulo mal-formado
    InvalidFormat,
    /// IOMMU necessário mas não disponível
    IommuRequired,
    /// Capability negada
    CapabilityDenied,
    /// Módulo já carregado
    AlreadyLoaded,
    /// Limite de módulos atingido
    LimitReached,
    /// Timeout na inicialização
    InitTimeout,
    /// Falha interna
    InternalError,
    /// Módulo banido (muitas falhas)
    Banned,
}

#[cfg(feature = "self_test")]
pub mod test;
