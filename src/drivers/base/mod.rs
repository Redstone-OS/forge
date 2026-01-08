//! # Redstone Drive System (RDS) - Módulo Central
//!
//! Este é o coração do sistema de gerenciamento de hardware do kernel Forge.
//! O RDS atua como **síndico** do condomínio de drivers, definindo regras,
//! gerenciando recursos e garantindo que nenhum inquilino (driver) cause
//! problemas para os demais.
//!
//! ## Responsabilidades Principais:
//! - **Orquestração**: Gerencia ciclo de vida de dispositivos e drivers
//! - **Pareamento**: Associa drivers a dispositivos compatíveis automaticamente
//! - **Hierarquia**: Mantém árvore topológica do hardware (pai-filho)
//! - **Recuperação**: Coordena subsistemas de monitoramento e recuperação de falhas
//! - **Isolamento**: Garante que falha em um driver não derrube o sistema
//!
//! ## Filosofia:
//! > "Os drivers não são donos da casa, são hóspedes. A base é o síndico."
//!
//! ## Fluxo de Funcionamento:
//! 1. `Bus` escaneia hardware e registra `Device`s
//! 2. Módulos de driver registram `Driver`s
//! 3. `DriverManager` faz matching e promove dispositivo para `Ready`
//! 4. Se falhar, `RecoveryManager` tenta recuperar ou fazer fallback

// =============================================================================
// SUBMÓDULOS DO RDS BASE
// =============================================================================
// Cada submódulo tem responsabilidade única e bem definida

/// Trait fundamental que todo driver deve implementar
pub mod driver;

/// Representação de um dispositivo de hardware no kernel
pub mod device;

/// Interface para barramentos de hardware (PCI, USB, etc)
pub mod bus;

/// Classes funcionais que agrupam dispositivos por função (Storage, Network, etc)
pub mod class;

/// Sistema de eventos para hotplug e notificações
pub mod events;

/// Monitor de saúde de dispositivos (watchdog de software)
pub mod monitor;

/// Gerenciador de recuperação de falhas
pub mod recovery;

/// Gerenciamento de energia (D-States)
pub mod power;

/// Gerenciador de recursos físicos (IRQ, MMIO, Ports)
pub mod resource;

/// Parâmetros configuráveis em runtime
pub mod parameters;

/// Zona de memória dedicada para drivers (DRIVER ZONE)
pub mod memory;

/// Pool centralizado de buffers DMA
pub mod dma;

/// Contexto persistente de driver (sobrevive a reloads)
pub mod context;

/// Sistema de probing assíncrono
pub mod async_probe;

/// Grafo de dependências entre drivers
pub mod deps;

/// Sistema de fallback progressivo
pub mod fallback;

/// Telemetria e diagnósticos do RDS
pub mod telemetry;

// =============================================================================
// IMPORTS
// =============================================================================

use self::context::DriverContext;
use self::device::{Device, DeviceId, DeviceState};
use self::driver::{Driver, DriverError};
use self::recovery::RecoveryManager;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES GLOBAIS DO RDS
// =============================================================================

/// Versão da ABI do RDS - usado para compatibilidade de módulos externos
pub const RDS_ABI_VERSION: u32 = 0x00_01_00_00; // v1.0.0

/// Versão mínima de ABI suportada
pub const RDS_ABI_MIN: u32 = 0x00_01_00_00;

/// Máximo de dispositivos que o sistema pode gerenciar
/// Limite alto para sistemas massivos, mas evita alocação infinita
pub const MAX_DEVICES: usize = 4096;

/// Máximo de drivers registrados simultaneamente
pub const MAX_DRIVERS: usize = 256;

// =============================================================================
// ESTRUTURA PRINCIPAL: DRIVER MANAGER
// =============================================================================

/// O gerenciador central de todo o hardware do sistema.
///
/// Esta é a "prefeitura" do sistema de drivers. Ela:
/// - Mantém registro de todos os dispositivos conhecidos
/// - Mantém registro de todos os drivers disponíveis
/// - Faz o matching entre drivers e dispositivos
/// - Coordena recuperação de falhas
/// - Gerencia contextos persistentes
pub struct DriverManager {
    /// Lista de todos os dispositivos conhecidos pelo kernel
    /// Cada dispositivo é protegido por Spinlock para acesso concorrente seguro
    devices: Vec<Arc<Spinlock<Device>>>,

    /// Drivers registrados e disponíveis para uso
    /// Arc permite compartilhamento entre múltiplos dispositivos
    drivers: Vec<Arc<dyn Driver>>,

    /// Gerenciador de recuperação de falhas
    /// Responsável por tentar recuperar dispositivos problemáticos
    recovery: RecoveryManager,

    /// Pool de contextos persistentes
    /// Sobrevive a reloads de drivers
    contexts: Vec<DriverContext>,

    /// Contador de IDs de dispositivos
    /// Garante IDs únicos para cada dispositivo descoberto
    next_device_id: u64,

    /// Flag indicando se o RDS foi inicializado
    initialized: bool,
}

// =============================================================================
// INSTÂNCIA GLOBAL (SINGLETON)
// =============================================================================

/// Instância global do DriverManager
/// Protegida por Spinlock para acesso thread-safe
static DRIVER_MANAGER: Spinlock<DriverManager> = Spinlock::new(DriverManager {
    devices: Vec::new(),
    drivers: Vec::new(),
    recovery: RecoveryManager::new(),
    contexts: Vec::new(),
    next_device_id: 1, // IDs começam em 1, 0 é reservado para "inválido"
    initialized: false,
});

// =============================================================================
// FUNÇÕES PÚBLICAS DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o Redstone Drive System.
///
/// Esta função DEVE ser chamada durante o boot do kernel, antes de
/// qualquer tentativa de registrar drivers ou dispositivos.
///
/// ## Ordem de inicialização:
/// 1. Inicializa subsistema de memória (Driver Zone)
/// 2. Inicializa pool de DMA
/// 3. Inicializa monitor de saúde
/// 4. Inicializa sistema de eventos
/// 5. Inicializa telemetria
pub fn init() {
    crate::kinfo!("(RDS) Inicializando Redstone Drive System v1.0...");

    // Subsistema de memória dedicada para drivers
    memory::init();

    // Pool centralizado de DMA
    dma::init();

    // Monitor de saúde (watchdog)
    monitor::init();

    // Sistema de eventos (pub/sub)
    events::init();

    // Telemetria e diagnósticos
    telemetry::init();

    // Grafo de dependências
    deps::init();

    // Sistema de fallback
    fallback::init();

    // Marca como inicializado
    DRIVER_MANAGER.lock().initialized = true;

    crate::kinfo!("(RDS) Sistema inicializado com sucesso!");
}

/// Verifica se o RDS foi inicializado.
///
/// Útil para debug e para garantir ordem correta de operações.
pub fn is_initialized() -> bool {
    DRIVER_MANAGER.lock().initialized
}

// =============================================================================
// FUNÇÕES PÚBLICAS DE REGISTRO
// =============================================================================

/// Registra um novo driver no sistema.
///
/// Após o registro, o RDS tentará parear o driver com dispositivos
/// que estão aguardando um driver compatível.
///
/// ## Parâmetros:
/// - `driver`: Arc contendo a implementação do driver
///
/// ## Fluxo:
/// 1. Adiciona driver à lista global
/// 2. Percorre dispositivos sem driver
/// 3. Chama `probe()` para testar compatibilidade
/// 4. Se compatível, anexa driver ao dispositivo
pub fn register_driver(driver: Arc<dyn Driver>) {
    let driver_name = driver.name();
    // crate::kinfo!("(RDS) Registrando driver:", driver_name);

    let mut mgr = DRIVER_MANAGER.lock();

    // Verifica limite de drivers
    if mgr.drivers.len() >= MAX_DRIVERS {
        crate::kerror!("(RDS) Limite de drivers atingido! Ignorando:", driver_name);
        return;
    }

    // Adiciona à lista
    mgr.drivers.push(driver.clone());

    // Tenta parear com dispositivos órfãos (sem driver)
    for dev_arc in mgr.devices.iter() {
        let mut dev = dev_arc.lock();

        // Só tenta parear se dispositivo não tem driver
        if dev.driver.is_some() {
            continue;
        }

        // Tenta o probe - se sucesso, anexa
        match driver.probe(&mut dev) {
            Ok(()) => {
                dev.driver = Some(driver.clone());
                dev.set_state(DeviceState::Ready);
                crate::kinfo!(
                    "(RDS) Driver pareado:",
                    driver_name,
                    "->",
                    dev.name_as_str()
                );

                // Notifica classes funcionais
                class::notify_device_added(dev_arc.clone());

                // Emite evento de dispositivo adicionado
                events::emit(events::DeviceEvent::Added(dev.id));

                // Registra na telemetria
                telemetry::record_driver_bound(dev.id, driver_name);
            }
            Err(DriverError::NotSupported) => {
                // Normal - driver não suporta este dispositivo
            }
            Err(e) => {
                // Erro real durante probe
                crate::kwarn!("(RDS) Probe falhou para", driver_name, "erro:", e as u8);
            }
        }
    }
}

/// Registra um novo dispositivo de hardware no sistema.
///
/// Geralmente chamado por drivers de barramento (PCI, USB, etc) quando
/// descobrem novo hardware durante o scan.
///
/// ## Parâmetros:
/// - `dev`: Estrutura Device preenchida pelo bus
///
/// ## Retorno:
/// - Arc<Spinlock<Device>> para referência ao dispositivo registrado
///
/// ## Fluxo:
/// 1. Atribui ID único ao dispositivo
/// 2. Adiciona à lista global
/// 3. Tenta encontrar driver compatível
/// 4. Se encontrar, faz binding
/// 5. Se não, dispositivo fica aguardando
pub fn register_device(mut dev: Device) -> Arc<Spinlock<Device>> {
    let mut mgr = DRIVER_MANAGER.lock();

    // Atribui ID único
    dev.id = DeviceId(mgr.next_device_id);
    mgr.next_device_id += 1;

    // Cria Arc protegido
    let dev_arc = Arc::new(Spinlock::new(dev));

    // Obscurece o mut dev original para evitar uso acidental
    let dev_locked = dev_arc.lock();
    let dev_name = dev_locked.name_as_str();
    let dev_id = dev_locked.id;

    crate::kinfo!("(RDS) Registrando dispositivo:", dev_name, "ID:", dev_id.0);

    // Verifica limite
    if mgr.devices.len() >= MAX_DEVICES {
        crate::kerror!("(RDS) Limite de dispositivos atingido!");
        // Retorna dispositivo mesmo assim, mas em estado de erro
        // Precisamos liberar o lock antes de retornar
        drop(dev_locked);
        dev_arc.lock().set_state(DeviceState::Dead);
        return dev_arc;
    }

    mgr.devices.push(dev_arc.clone());

    // Cria contexto persistente para este dispositivo
    let ctx = DriverContext::new(dev_id);
    mgr.contexts.push(ctx);

    // Libera o lock inicial
    drop(dev_locked);

    // Tenta encontrar driver compatível
    let mut bound_driver: Option<Arc<dyn Driver>> = None;

    for driver in mgr.drivers.iter() {
        let mut d = dev_arc.lock();

        match driver.probe(&mut d) {
            Ok(()) => {
                d.driver = Some(driver.clone());
                d.set_state(DeviceState::Ready);
                bound_driver = Some(driver.clone());
                break;
            }
            Err(DriverError::NotSupported) => {
                // Esperado - tenta próximo driver
            }
            Err(e) => {
                crate::kwarn!("(RDS) Probe falhou:", driver.name(), "erro:", e as u8);
            }
        }
    }

    // Notifica sistema sobre resultado
    if let Some(drv) = bound_driver {
        crate::kinfo!("(RDS) Dispositivo pareado com:", drv.name());
        class::notify_device_added(dev_arc.clone());
        events::emit(events::DeviceEvent::Added(dev_id));
        telemetry::record_device_added(dev_id);
    } else {
        let dev = dev_arc.lock();
        let dev_name = dev.name_as_str();
        crate::kwarn!("(RDS) Nenhum driver compatível para:", dev_name);
        telemetry::record_device_orphan(dev_id);
    }

    dev_arc
}

// =============================================================================
// FUNÇÕES PÚBLICAS DE CONSULTA
// =============================================================================

/// Busca um dispositivo pelo seu ID único.
///
/// ## Parâmetros:
/// - `id`: DeviceId do dispositivo procurado
///
/// ## Retorno:
/// - Some(Arc<Spinlock<Device>>) se encontrado
/// - None se não existe dispositivo com este ID
pub fn get_device(id: DeviceId) -> Option<Arc<Spinlock<Device>>> {
    DRIVER_MANAGER
        .lock()
        .devices
        .iter()
        .find(|d| d.lock().id == id)
        .cloned()
}

/// Retorna lista de todos os dispositivos registrados.
///
/// Útil para debug e para o sistema de telemetria.
pub fn get_all_devices() -> Vec<Arc<Spinlock<Device>>> {
    DRIVER_MANAGER.lock().devices.clone()
}

/// Retorna lista de todos os drivers registrados.
pub fn get_all_drivers() -> Vec<Arc<dyn Driver>> {
    DRIVER_MANAGER.lock().drivers.clone()
}

/// Retorna quantidade de dispositivos registrados.
pub fn device_count() -> usize {
    DRIVER_MANAGER.lock().devices.len()
}

/// Retorna quantidade de drivers registrados.
pub fn driver_count() -> usize {
    DRIVER_MANAGER.lock().drivers.len()
}

// =============================================================================
// FUNÇÕES PÚBLICAS DE RECUPERAÇÃO
// =============================================================================

/// Reporta uma falha em um dispositivo ao sistema de recuperação.
///
/// Esta função deve ser chamada quando um driver detecta erro crítico.
/// O RecoveryManager decidirá a melhor estratégia (retry, rebind, reset, isolate).
///
/// ## Parâmetros:
/// - `id`: ID do dispositivo com problema
/// - `err`: Tipo do erro ocorrido
pub fn report_failure(id: DeviceId, err: DriverError) {
    crate::kerror!("(RDS) Falha reportada para dispositivo ID:", id.0);

    let mgr = DRIVER_MANAGER.lock();

    if let Some(dev_arc) = mgr.devices.iter().find(|d| d.lock().id == id) {
        // Delega para o RecoveryManager
        mgr.recovery.handle_failure(dev_arc.clone(), err);

        // Registra na telemetria
        telemetry::record_failure(id, err);
    } else {
        crate::kerror!("(RDS) Dispositivo não encontrado para recuperação:", id.0);
    }
}

/// Solicita hot-reload de um driver específico.
///
/// ## STUB - Não implementado ainda
///
/// Esta funcionalidade permitirá atualizar drivers sem reiniciar o sistema.
pub fn request_driver_reload(driver_name: &str) -> Result<(), DriverError> {
    crate::kwarn!("(RDS) request_driver_reload() ainda não implementado!");
    crate::kwarn!("(RDS) Driver solicitado:", driver_name);

    // TODO: Implementar hot-reload
    // 1. Pausar operações em andamento
    // 2. Salvar estado no DriverContext
    // 3. Chamar remove() no driver antigo
    // 4. Carregar novo driver
    // 5. Chamar probe() no novo driver
    // 6. Restaurar estado do DriverContext
    // 7. Resumir operações

    Err(DriverError::NotSupported)
}

// =============================================================================
// FUNÇÕES PÚBLICAS DE SHUTDOWN
// =============================================================================

/// Desliga todos os dispositivos de forma ordenada.
///
/// Chamado durante o shutdown do sistema. Percorre a árvore de dispositivos
/// em ordem reversa (folhas primeiro) para garantir que dependências
/// sejam respeitadas.
pub fn shutdown_all() {
    crate::kinfo!("(RDS) Iniciando shutdown de todos os dispositivos...");

    let mgr = DRIVER_MANAGER.lock();

    // TODO: Ordenar por dependências (folhas primeiro)
    // Por enquanto, faz em ordem reversa simples

    for dev_arc in mgr.devices.iter().rev() {
        let mut dev = dev_arc.lock();

        if let Some(driver) = dev.driver.clone() {
            crate::kinfo!("(RDS) Desligando:", dev.name_as_str());

            // Chama shutdown do driver
            driver.shutdown(&mut dev);

            // Atualiza estado
            dev.set_state(DeviceState::Disconnected);
        }
    }

    // Desliga barramentos
    bus::shutdown_all();

    crate::kinfo!("(RDS) Shutdown completo!");
}

// =============================================================================
// FUNÇÕES INTERNAS
// =============================================================================

// TODO: Revisar no futuro
#[allow(unused)]
/// Gera próximo ID de dispositivo (uso interno).
fn next_id() -> DeviceId {
    let mut mgr = DRIVER_MANAGER.lock();
    let id = DeviceId(mgr.next_device_id);
    mgr.next_device_id += 1;
    id
}

// TODO: Revisar no futuro
#[allow(unused)]
/// Busca contexto persistente de um dispositivo (uso interno).
pub(crate) fn get_context(_id: DeviceId) -> Option<&'static DriverContext> {
    // TODO: Implementar busca no pool de contextos
    crate::kwarn!("(RDS) get_context() ainda não totalmente implementado");
    None
}
