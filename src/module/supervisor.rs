//! # Module Supervisor
//!
//! Gerencia o ciclo de vida dos módulos de kernel.
//!
//! ## Responsabilidades
//! - Carregar/descarregar módulos
//! - Alocar recursos (páginas, capabilities)
//! - Monitorar saúde via watchdog
//! - Gerenciar fallbacks
use super::{ModuleError, ModuleLoader, ModuleSandbox, ModuleWatchdog, SignatureVerifier};
use crate::security::Capability;
use crate::sync::Mutex;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// ID único de um módulo carregado
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u64);

impl ModuleId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// Estado de saúde do módulo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Módulo está carregando
    Loading,
    /// Módulo está ativo e saudável
    Active,
    /// Módulo está com problemas (healthcheck falhou)
    Degraded,
    /// Módulo está sendo descarregado
    Unloading,
    /// Módulo falhou e foi descarregado
    Failed,
    /// Módulo banido (muitas falhas)
    Banned,
}

/// Ação de fallback quando módulo falha
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackAction {
    /// Usar driver builtin do kernel
    UseBuiltIn,
    /// Desabilitar funcionalidade
    Disable,
    /// Tentar recarregar o módulo
    Reload,
    /// Kernel panic (para drivers críticos como storage)
    Panic,
}

/// Configuração de limites de recursos para módulos
#[derive(Debug, Clone)]
pub struct ModuleLimits {
    /// Máximo de páginas de código (RX)
    pub max_code_pages: usize,
    /// Máximo de páginas de dados (RW)
    pub max_data_pages: usize,
    /// Máximo de capabilities simultâneas
    pub max_capabilities: usize,
    /// Máximo de IRQs que pode registrar
    pub max_irqs: usize,
    /// Timeout de inicialização em ms
    pub init_timeout_ms: u64,
    /// Máximo de falhas antes de ban
    pub max_faults: u32,
}

impl Default for ModuleLimits {
    fn default() -> Self {
        Self {
            max_code_pages: 256,  // 1MB código
            max_data_pages: 1024, // 4MB dados
            max_capabilities: 64,
            max_irqs: 4,
            init_timeout_ms: 5000, // 5 segundos
            max_faults: 3,
        }
    }
}

/// Módulo carregado no kernel
pub struct LoadedModule {
    /// ID único
    pub id: ModuleId,
    /// Nome do módulo
    pub name: String,
    /// Estado atual
    pub state: ModuleState,
    /// Páginas de código (endereços físicos)
    pub code_pages: Vec<u64>,
    /// Páginas de dados (endereços físicos)
    pub data_pages: Vec<u64>,
    /// Capabilities concedidas
    pub capabilities: Vec<Capability>,
    /// Contador de falhas
    pub fault_count: u32,
    /// Ação de fallback configurada
    pub fallback: FallbackAction,
    /// Limites de recursos
    pub limits: ModuleLimits,
    /// Entry point do módulo
    pub entry_point: u64,
    /// Função de cleanup
    pub exit_fn: Option<u64>,
}

impl LoadedModule {
    /// Cria um novo módulo com estado inicial
    pub fn new(id: ModuleId, name: String) -> Self {
        Self {
            id,
            name,
            state: ModuleState::Loading,
            code_pages: Vec::new(),
            data_pages: Vec::new(),
            capabilities: Vec::new(),
            fault_count: 0,
            fallback: FallbackAction::Disable,
            limits: ModuleLimits::default(),
            entry_point: 0,
            exit_fn: None,
        }
    }

    /// Incrementa contador de falhas e retorna true se deve banir
    pub fn record_fault(&mut self) -> bool {
        self.fault_count += 1;
        self.fault_count >= self.limits.max_faults
    }
}

/// Supervisor central de módulos
pub struct ModuleSupervisor {
    /// Módulos carregados (ID -> Module)
    modules: BTreeMap<ModuleId, LoadedModule>,
    /// Próximo ID a ser atribuído
    next_id: u64,
    /// Loader de módulos
    loader: ModuleLoader,
    /// Verificador de assinaturas
    verifier: SignatureVerifier,
    /// Sandbox de isolamento
    pub sandbox: ModuleSandbox,
    /// Watchdog de monitoramento
    pub watchdog: ModuleWatchdog,
    /// Limites padrão
    default_limits: ModuleLimits,
    /// Módulos banidos (hash do nome)
    banned: Vec<u64>,
    /// Sistema inicializado
    initialized: bool,
}

impl ModuleSupervisor {
    /// Cria um novo supervisor
    pub const fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            next_id: 1,
            loader: ModuleLoader::new(),
            verifier: SignatureVerifier::new(),
            sandbox: ModuleSandbox::new(),
            watchdog: ModuleWatchdog::new(),
            default_limits: ModuleLimits {
                max_code_pages: 256,
                max_data_pages: 1024,
                max_capabilities: 64,
                max_irqs: 4,
                init_timeout_ms: 5000,
                max_faults: 3,
            },
            banned: Vec::new(),
            initialized: false,
        }
    }

    /// Inicializa o supervisor
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }

        // Inicializar componentes
        self.watchdog.init();
        self.sandbox.init();

        self.initialized = true;
    }

    /// Carrega um módulo do caminho especificado
    pub fn load_module(&mut self, path: &str) -> Result<ModuleId, ModuleError> {
        if !self.initialized {
            return Err(ModuleError::InternalError);
        }

        crate::kdebug!("(Module) Carregando módulo: ");
        // Nota: não podemos concatenar strings facilmente sem alloc

        // 1. Verificar se não está banido
        let path_hash = Self::hash_path(path);
        if self.banned.contains(&path_hash) {
            crate::kwarn!("(Module) Módulo banido!");
            return Err(ModuleError::Banned);
        }

        // 2. Carregar ELF do VFS
        let elf_data = self.loader.load_from_vfs(path)?;

        // 3. Verificar assinatura
        if !self.verifier.verify(&elf_data) {
            crate::kerror!("(Module) Assinatura inválida!");
            return Err(ModuleError::InvalidSignature);
        }

        // 4. Alocar ID
        let id = ModuleId::new(self.next_id);
        self.next_id += 1;

        // 5. Criar estrutura do módulo
        let name = Self::extract_name(path);
        let mut module = LoadedModule::new(id, name);
        module.limits = self.default_limits.clone();

        // 6. Parsear ELF e alocar páginas
        self.loader.parse_and_load(&elf_data, &mut module)?;

        // 7. Configurar sandbox
        self.sandbox.setup_module(&module)?;

        // 8. Registrar no watchdog
        self.watchdog.register(id);

        // 9. Chamar init do módulo (supervisionado)
        self.call_module_init(&mut module)?;

        // 10. Marcar como ativo
        module.state = ModuleState::Active;

        // 11. Armazenar
        self.modules.insert(id, module);

        crate::kinfo!("(Module) Módulo carregado com sucesso, ID=", id.as_u64());

        Ok(id)
    }

    /// Descarrega um módulo
    pub fn unload_module(&mut self, id: ModuleId) -> Result<(), ModuleError> {
        // Remover módulo primeiro para evitar problemas de borrow
        let mut module = self.modules.remove(&id).ok_or(ModuleError::NotFound)?;

        // Marcar como descarregando
        module.state = ModuleState::Unloading;

        // Chamar exit do módulo
        if let Some(exit_fn) = module.exit_fn {
            crate::ktrace!("(Module) Chamando exit em ", exit_fn);
            // TODO: Implementar chamada supervisionada
        }

        // Remover do watchdog
        self.watchdog.unregister(id);

        // Limpar sandbox
        self.sandbox.cleanup_module(&module);

        // Revogar capabilities (placeholder)
        // TODO: Implementar revogação real

        // Desalocar páginas
        self.loader.free_pages(&mut module);

        crate::kinfo!("(Module) Módulo descarregado, ID=", id.as_u64());

        Ok(())
    }

    /// Lista todos os módulos carregados
    pub fn list_modules(&self) -> Vec<ModuleId> {
        self.modules.keys().copied().collect()
    }

    /// Obtém informações de um módulo
    pub fn get_module(&self, id: ModuleId) -> Option<&LoadedModule> {
        self.modules.get(&id)
    }

    /// Reporta uma falha de módulo
    pub fn report_fault(&mut self, id: ModuleId) {
        if let Some(module) = self.modules.get_mut(&id) {
            let should_ban = module.record_fault();

            if should_ban {
                module.state = ModuleState::Banned;
                self.banned.push(Self::hash_path(&module.name));
                crate::kerror!("(Module) Módulo banido por excesso de falhas!");
            } else {
                module.state = ModuleState::Degraded;
            }

            // Executar fallback
            match module.fallback {
                FallbackAction::UseBuiltIn => {
                    crate::kwarn!("(Module) Ativando driver builtin");
                    // TODO: ativar driver builtin
                }
                FallbackAction::Disable => {
                    crate::kwarn!("(Module) Funcionalidade desabilitada");
                }
                FallbackAction::Reload => {
                    crate::kwarn!("(Module) Tentando recarregar módulo");
                    // TODO: recarregar
                }
                FallbackAction::Panic => {
                    crate::kerror!("(Module) Falha crítica - PANIC");
                    // TODO: kernel panic controlado
                }
            }
        }
    }

    /// Define ação de fallback para um módulo
    pub fn set_fallback(&mut self, id: ModuleId, action: FallbackAction) {
        if let Some(module) = self.modules.get_mut(&id) {
            module.fallback = action;
        }
    }

    /// Define limites padrão para novos módulos
    pub fn set_default_limits(&mut self, limits: ModuleLimits) {
        self.default_limits = limits;
    }

    // --- Funções internas ---

    fn call_module_init(&self, module: &mut LoadedModule) -> Result<(), ModuleError> {
        if module.entry_point == 0 {
            return Err(ModuleError::InvalidFormat);
        }

        // TODO: Implementar chamada supervisionada com timeout
        // Por agora, apenas marcamos como sucesso
        crate::ktrace!("(Module) Chamando init em ", module.entry_point);

        Ok(())
    }
    #[allow(dead_code)]
    fn call_module_exit(&self, module: &LoadedModule) {
        if let Some(exit_fn) = module.exit_fn {
            crate::ktrace!("(Module) Chamando exit em ", exit_fn);
            // TODO: Implementar chamada supervisionada
        }
    }
    #[allow(dead_code)]
    fn revoke_capability(&self, _cap: &Capability) {
        // TODO: Implementar revogação de capability
    }

    fn hash_path(path: &str) -> u64 {
        // Hash simples para identificar módulos banidos
        let mut hash: u64 = 0;
        for byte in path.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    fn extract_name(path: &str) -> String {
        // Extrai nome do arquivo do caminho
        if let Some(pos) = path.rfind('/') {
            String::from(&path[pos + 1..])
        } else {
            String::from(path)
        }
    }
}

/// Instância global do supervisor
pub static SUPERVISOR: Mutex<ModuleSupervisor> = Mutex::new(ModuleSupervisor::new());
