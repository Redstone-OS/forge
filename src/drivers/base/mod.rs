//! # Redstone Driver Model (RDM) - Central Orchestrator
//!
//! O `drivers::base` é a espinha dorsal do gerenciamento de hardware do RedstoneOS.
//! Este módulo implementa o **DriverManager**, que atua como o supervisor global de todos
//! os periféricos e drivers do sistema.
//!
//! ## Responsabilidades do Core:
//! - **Orquestração**: Gerencia o ciclo de vida de dispositivos e drivers.
//! - **Pareamento (Binding)**: Automatiza a associação de drivers a dispositivos compatíveis.
//! - **Hierarquia**: Mantém a árvore topológica do hardware (relação pai-filho).
//! - **Supervisão**: Coordena os subsistemas de monitoramento, recursos e recuperação.
//!
//! ## Fluxo de Funcionamento:
//! 1. Barramentos (`Bus`) escaneiam o hardware e registram `Device`s.
//! 2. Módulos de driver registram `Driver`s.
//! 3. O `DriverManager` realiza o matching e promove o dispositivo ao estado `Ready`.

pub mod bus;
pub mod class;
pub mod device;
pub mod driver;
pub mod events;
pub mod monitor;
pub mod parameters;
pub mod power;
pub mod recovery;
pub mod resource;

use self::device::{Device, DeviceId, DeviceState};
use self::driver::Driver;
use self::recovery::RecoveryManager;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Gerenciador Global de Hardware e Drivers
pub struct DriverManager {
    /// Lista de todos os dispositivos conhecidos pelo kernel
    devices: Vec<Arc<Spinlock<Device>>>,
    /// Drivers registrados e disponíveis para uso
    drivers: Vec<Arc<dyn Driver>>,
    /// Gerenciador de recuperação de falhas
    recovery: RecoveryManager,
}

static DRIVER_MANAGER: Spinlock<DriverManager> = Spinlock::new(DriverManager {
    devices: Vec::new(),
    drivers: Vec::new(),
    recovery: RecoveryManager::new(),
});

/// Inicializa o subsistema de drivers base.
pub fn init() {
    crate::kinfo!("(RDM) Inicializando Redstone Driver Model...");
    monitor::init();
    // Outras inicializações de subsistemas base podem ser adicionadas aqui
}

/// Registra um novo driver no sistema.
/// Após o registro, o kernel tentará pareá-lo com dispositivos ociosos.
pub fn register_driver(driver: Arc<dyn Driver>) {
    crate::kinfo!("(RDM) Novo driver registrado:", driver.name());
    let mut mgr = DRIVER_MANAGER.lock();
    mgr.drivers.push(driver.clone());

    // Tentativa imediata de parear com dispositivos que estão sem driver
    for dev_arc in mgr.devices.iter() {
        let mut dev = dev_arc.lock();
        if dev.driver.is_none() {
            if driver.probe(&mut dev).is_ok() {
                dev.driver = Some(driver.clone());
                dev.state = DeviceState::Ready;
                crate::kinfo!("(RDM) Driver pareado com sucesso:", driver.name());

                // Notifica as classes sobre o novo dispositivo operacional
                class::notify_device_added(dev_arc.clone());
                events::emit(events::DeviceEvent::Added(dev.id));
            }
        }
    }
}

/// Registra um novo dispositivo no sistema.
/// Geralmente chamado por um driver de barramento (PCI/USB/etc).
pub fn register_device(dev: Device) -> Arc<Spinlock<Device>> {
    let dev_id = dev.id;
    let dev_arc = Arc::new(Spinlock::new(dev));

    let mut mgr = DRIVER_MANAGER.lock();
    mgr.devices.push(dev_arc.clone());

    // Tenta encontrar um driver compatível já registrado
    let mut found_driver = None;
    for driver in mgr.drivers.iter() {
        let mut d = dev_arc.lock();
        if driver.probe(&mut d).is_ok() {
            found_driver = Some(driver.clone());
            d.driver = Some(driver.clone());
            d.state = DeviceState::Ready;
            break;
        }
    }

    if let Some(drv) = found_driver {
        crate::kinfo!("(RDM) Dispositivo registrado e pareado com:", drv.name());
        class::notify_device_added(dev_arc.clone());
        events::emit(events::DeviceEvent::Added(dev_id));
    } else {
        crate::kwarn!("(RDM) Dispositivo registrado, mas nenhum driver compatível encontrado.");
    }

    dev_arc
}

/// Reporta uma falha crítica em um dispositivo.
/// Aciona o sistema de recuperação para tentar restaurar o hardware.
pub fn report_failure(id: DeviceId, err: driver::DriverError) {
    let mgr = DRIVER_MANAGER.lock();
    if let Some(dev_arc) = mgr.devices.iter().find(|d| d.lock().id == id) {
        mgr.recovery.handle_failure(dev_arc.clone(), err);
    }
}

/// Retorna um dispositivo pelo seu ID.
pub fn get_device(id: DeviceId) -> Option<Arc<Spinlock<Device>>> {
    DRIVER_MANAGER
        .lock()
        .devices
        .iter()
        .find(|d| d.lock().id == id)
        .cloned()
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Árvore de Dispositivos Dinâmica:
//    - Implementar a visualização da árvore completa (DUMP) para depuração
//      via comando de shell.
//
// 2. Carregamento de Drivers Sob Demanda:
//    - Integrar com o sistema de arquivos para carregar binários de drivers
//      apenas quando o hardware correspondente for detectado (redução de memória).
//
// 3. Suporte a Nomes Persistentes:
//    - Garantir que um disco sempre seja "sda" mesmo se trocado de porta USB,
//      baseado em UUIDs de hardware.
//
// 4. Gestão de Dependências:
//    - Garantir que se o driver do Barramento PCI for descarregado, todos os
//      drivers de placas conectadas sejam removidos primeiro.
//
// 5. Interface Sysfs (/devices):
//    - Exportar toda a estrutura do DriverManager para um sistema de arquivos
//      virtual acessível pelo usuário.
