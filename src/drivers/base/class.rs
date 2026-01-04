//! # Classes Funcionais (Class Layer)
//!
//! Este módulo agrupa dispositivos por **função** em vez de **conexão física**.
//! É a abstração usada pelos subsistemas de alto nível (VFS, GUI, Audio Stack).
//!
//! ## Por que classes?
//! Um teclado pode estar conectado via:
//! - PS/2 (legacy)
//! - USB (moderno)
//! - Bluetooth (wireless)
//!
//! Para o gerenciador de janelas, não importa HOW o teclado está conectado,
//! apenas que ele é um dispositivo de **Input**.
//!
//! ## Exemplos de Classes:
//! - **Storage**: Discos NVMe, SATA, USB, VirtIO → todos são "block devices"
//! - **Input**: Teclados, mouses, touchpads → todos geram InputEvents
//! - **Display**: GPUs, framebuffers → todos mostram pixels na tela
//! - **Network**: Ethernet, WiFi, VirtIO → todos enviam/recebem pacotes

use super::device::{Device, DeviceId};
use super::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// TIPOS DE CLASSE
// =============================================================================

/// Categorias funcionais de dispositivos.
///
/// Cada classe agrupa dispositivos que oferecem funcionalidade similar,
/// independente de como estão conectados fisicamente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassType {
    /// Saída de vídeo (Framebuffers, GPUs).
    /// Subsistema gráfico usa esta classe.
    Display,

    /// Dispositivos de entrada (Teclado, Mouse, Touchpad).
    /// Gerenciador de janelas usa esta classe.
    Input,

    /// Armazenamento de dados (Block devices).
    /// VFS usa esta classe para montar filesystems.
    Storage,

    /// Conectividade de rede (Ethernet, WiFi, Bluetooth).
    /// Stack de rede usa esta classe.
    Network,

    /// Dispositivos de áudio (Placas de som, CODECs).
    /// Subsistema de áudio usa esta classe.
    Sound,

    /// Dispositivos críticos do sistema (Timer, IRQ Controller, DMA).
    /// Kernel core usa esta classe.
    System,

    /// Comunicação serial (UART, COM, USB-Serial).
    /// Usado para debug e dispositivos industriais.
    Serial,

    /// Dispositivos que não se encaixam em classes específicas.
    Generic,
}

impl ClassType {
    /// Retorna nome legível da classe.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Display => "Display",
            Self::Input => "Input",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Sound => "Sound",
            Self::System => "System",
            Self::Serial => "Serial",
            Self::Generic => "Generic",
        }
    }
}

// =============================================================================
// TRAIT DE CLASSE
// =============================================================================

/// Interface que toda classe funcional deve implementar.
///
/// As classes concretas (Ex: StorageClass, InputClass) implementam esta trait
/// para permitir que o sistema encontre dispositivos por função.
pub trait Class: Send + Sync {
    /// Retorna nome descritivo da classe.
    fn name(&self) -> &'static str;

    /// Retorna o tipo funcional.
    fn class_type(&self) -> ClassType;

    /// Retorna todos os dispositivos registrados nesta classe.
    fn get_devices(&self) -> Vec<Arc<Spinlock<Device>>>;

    /// Notifica que um novo dispositivo foi adicionado.
    fn on_device_added(&self, dev: Arc<Spinlock<Device>>);

    /// Notifica que um dispositivo foi removido.
    fn on_device_removed(&self, dev_id: DeviceId);
}

// =============================================================================
// REGISTRO DE CLASSES
// =============================================================================

/// Lista global de classes registradas.
static CLASS_REGISTRY: Spinlock<Vec<Arc<dyn Class>>> = Spinlock::new(Vec::new());

/// Registra uma nova classe funcional.
///
/// Chamado durante inicialização pelos subsistemas (block, input, display, etc).
pub fn register(class: Arc<dyn Class>) {
    let name = class.name();
    crate::kinfo!("(Class) Registrando classe funcional:", name);
    CLASS_REGISTRY.lock().push(class);
}

/// Busca todos os dispositivos de uma determinada classe.
///
/// Útil para o VFS encontrar todos os discos, por exemplo.
pub fn get_devices_by_type(class_type: ClassType) -> Vec<Arc<Spinlock<Device>>> {
    let mut all_devices = Vec::new();
    let classes = CLASS_REGISTRY.lock();

    for class in classes.iter() {
        if class.class_type() == class_type {
            all_devices.extend(class.get_devices());
        }
    }

    all_devices
}

/// Notifica as classes sobre um novo dispositivo.
///
/// Chamado pelo DriverManager quando um dispositivo é pareado com driver.
pub fn notify_device_added(dev: Arc<Spinlock<Device>>) {
    let dev_type = dev.lock().device_type;
    let classes = CLASS_REGISTRY.lock();

    for class in classes.iter() {
        if is_compatible(class.class_type(), dev_type) {
            class.on_device_added(dev.clone());
        }
    }
}

/// Notifica as classes sobre remoção de dispositivo.
pub fn notify_device_removed(dev_id: DeviceId, dev_type: DeviceType) {
    let classes = CLASS_REGISTRY.lock();

    for class in classes.iter() {
        if is_compatible(class.class_type(), dev_type) {
            class.on_device_removed(dev_id);
        }
    }
}

/// Retorna lista de todas as classes registradas.
pub fn get_all() -> Vec<Arc<dyn Class>> {
    CLASS_REGISTRY.lock().clone()
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Mapeia tipo de dispositivo para classe funcional.
///
/// Um dispositivo pode ser compatível com múltiplas classes.
fn is_compatible(class_type: ClassType, dev_type: DeviceType) -> bool {
    match (class_type, dev_type) {
        (ClassType::Storage, DeviceType::Storage) => true,
        (ClassType::Input, DeviceType::Input) => true,
        (ClassType::Display, DeviceType::Display) => true,
        (ClassType::Network, DeviceType::Network) => true,
        (ClassType::Sound, DeviceType::Audio) => true,
        (ClassType::Serial, DeviceType::Serial) => true,
        (ClassType::System, DeviceType::Timer) => true,
        (ClassType::System, DeviceType::Controller) => true,
        (ClassType::System, DeviceType::Bus) => true,
        (ClassType::Generic, DeviceType::Generic) => true,
        _ => false,
    }
}

/// Converte DeviceType para ClassType correspondente.
pub fn device_type_to_class(dev_type: DeviceType) -> ClassType {
    match dev_type {
        DeviceType::Storage => ClassType::Storage,
        DeviceType::Input => ClassType::Input,
        DeviceType::Display => ClassType::Display,
        DeviceType::Network => ClassType::Network,
        DeviceType::Audio => ClassType::Sound,
        DeviceType::Serial => ClassType::Serial,
        DeviceType::Timer | DeviceType::Controller | DeviceType::Bus => ClassType::System,
        _ => ClassType::Generic,
    }
}

// =============================================================================
// IMPLEMENTAÇÕES DE CLASSE GENÉRICA (STUB)
// =============================================================================

/// Classe genérica para dispositivos sem classe específica.
///
/// Funciona como fallback para dispositivos que não se encaixam
/// em nenhuma categoria.
pub struct GenericClass {
    devices: Spinlock<Vec<Arc<Spinlock<Device>>>>,
}

impl GenericClass {
    pub fn new() -> Self {
        Self {
            devices: Spinlock::new(Vec::new()),
        }
    }
}

impl Class for GenericClass {
    fn name(&self) -> &'static str {
        "Generic Devices"
    }

    fn class_type(&self) -> ClassType {
        ClassType::Generic
    }

    fn get_devices(&self) -> Vec<Arc<Spinlock<Device>>> {
        self.devices.lock().clone()
    }

    fn on_device_added(&self, dev: Arc<Spinlock<Device>>) {
        crate::kinfo!(
            "(Class) Dispositivo genérico adicionado:",
            dev.lock().name_as_str()
        );
        self.devices.lock().push(dev);
    }

    fn on_device_removed(&self, dev_id: DeviceId) {
        self.devices.lock().retain(|d| d.lock().id != dev_id);
    }
}
