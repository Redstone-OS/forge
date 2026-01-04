//! # Classes Funcionais (Class Layer)
//!
//! Agrupa dispositivos por sua "função" em vez de sua "conexão".
//! É a abstração usada pelos subsistemas de alto nível do kernel, como VFS e GUI.
//!
//! ## Por que usar Classes?
//! - Um drive de disco pode estar no barramento USB, PCI (NVMe) ou IDE.
//! - O subsistema de arquivos (VFS) não quer saber *como* o disco está conectado,
//!   apenas que ele é um membro da classe `Storage`.
//!
//! ## Exemplos:
//! - `Input`: Consolida teclados PS/2 e USB para o gerenciador de janelas.
//! - `Display`: Consolida drivers de vídeo para o subsistema gráfico.

use super::device::Device;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Categorias funcionais de dispositivos no RedstoneOS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassType {
    Display, // Saída de vídeo (Framebuffers, GPUs)
    Input,   // Teclado, mouse, touchpad, joystick
    Storage, // Armazenamento de dados (Drives de Bloco)
    Network, // Conectividade (Ethernet, WiFi, Bluetooth)
    Sound,   // Áudio (Placas de som, Codecs)
    System,  // Dispositivos críticos do sistema (Timer, Interrupt Ctrl, DMA Ctrl)
    Serial,  // Comunicação serial (UART, COM, USB-Serial)
    Generic, // Dispositivos que não se encaixam em classes específicas
}

/// Interface fundamental para uma classe de hardware.
/// Cada subsistema (ex: USB Storage) pode implementar uma classe funcional.
pub trait Class: Send + Sync {
    /// Nome descritivo da classe (ex: "Mass Storage Devices")
    fn name(&self) -> &'static str;

    /// Tipo funcional da classe
    fn class_type(&self) -> ClassType;

    /// Retorna todos os dispositivos atualmente registrados nesta classe funcional.
    fn get_devices(&self) -> Vec<Arc<Spinlock<Device>>>;

    /// Notifica a classe que um novo dispositivo compatível foi adicionado
    fn on_device_added(&self, _dev: Arc<Spinlock<Device>>) {}

    /// Notifica a classe que um dispositivo foi removido
    fn on_device_removed(&self, _dev_id: super::device::DeviceId) {}
}

// =============================================================================
// REGISTRO DE CLASSES (CLASS REGISTRY)
// =============================================================================

static CLASS_REGISTRY: Spinlock<Vec<Arc<dyn Class>>> = Spinlock::new(Vec::new());

/// Registra uma nova classe funcional no sistema.
/// Geralmente chamado durante a inicialização de subsistemas (ex: block::init()).
pub fn register(class: Arc<dyn Class>) {
    crate::kinfo!("(Class) Registrando classe funcional:", class.name());
    CLASS_REGISTRY.lock().push(class);
}

/// Busca todos os dispositivos de uma determinada classe em todos os registros.
/// Útil para o VFS encontrar todos os discos disponíveis.
pub fn get_devices_by_type(class_type: ClassType) -> Vec<Arc<Spinlock<Device>>> {
    let mut all_devices = Vec::new();
    let classes = CLASS_REGISTRY.lock();

    for class in classes.iter() {
        if class.class_type() == class_type {
            all_devices.append(&mut class.get_devices());
        }
    }

    all_devices
}

/// Notifica as classes relevantes sobre um novo dispositivo (Broadcast de Hotplug)
pub fn notify_device_added(dev: Arc<Spinlock<Device>>) {
    let dev_type = dev.lock().device_type;
    let classes = CLASS_REGISTRY.lock();

    for class in classes.iter() {
        // Se a classe for compatível com o tipo do dispositivo, notifica
        if is_class_compatible(class.class_type(), dev_type) {
            class.on_device_added(dev.clone());
        }
    }
}

/// Mapeia o tipo de dispositivo para a classe funcional correspondente
fn is_class_compatible(class: ClassType, dev: super::driver::DeviceType) -> bool {
    use super::driver::DeviceType as DT;
    match (class, dev) {
        (ClassType::Storage, DT::Storage) => true,
        (ClassType::Input, DT::Input) => true,
        (ClassType::Display, DT::Display) => true,
        (ClassType::Network, DT::Network) => true,
        (ClassType::Serial, DT::Serial) => true,
        (ClassType::System, DT::Timer) => true,
        (ClassType::System, DT::Bus) => true,
        _ => false,
    }
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Criação Automática de Nós em /devices:
//    - Integrar com o VFS para que qualquer dispositivo registrado em uma classe
//      funcional (ex: Storage) apareça automaticamente como /devices/sdX ou /devices/fbX.
//
// 2. Abstração de Interface (Ops):
//    - Definir uma trait de "Operations" para cada classe (ex: BlockOps para discos,
//      NetOps para rede). Isso permitiria que o kernel usasse os dispositivos via
//      classes sem conhecer o driver específico.
//
// 3. Priorização de Dispositivos:
//    - Se houver dois teclados (Input), permitir que o sistema defina qual é a
//      "instância primária" da classe.
//
// 4. Agrupamento de Estatísticas:
//    - Coletar métricas de performance por classe (ex: "Uso total de banda na classe Network").
//
// 5. Suporte a Sensores e Atuadores:
//    - Adicionar classes para I2C/SPI sensors, PWM, GPIOs, etc.
