//! # Camada de Barramento (Bus Layer)
//!
//! Este arquivo define a interface para **barramentos de hardware**.
//! Os barramentos são a "rodoviária" do sistema - eles:
//!
//! - **Descobrem** dispositivos conectados (enumeram o hardware)
//! - **Identificam** cada dispositivo (IDs, classes, endereços)
//! - **Facilitam** comunicação entre drivers e hardware
//! - **Gerenciam** sinais físicos (resets, power, speed)
//!
//! ## Filosofia:
//! > "O Bus conhece todas as rotas. Drivers não precisam reinventar a roda."
//!
//! Em vez de cada driver USB implementar seu próprio scan de portas,
//! o bus USB faz isso uma vez e fornece os dados a todos.
//!
//! ## Barramentos Suportados:
//! - **PCI/PCIe**: Placas de vídeo, rede, storage (NVMe, AHCI)
//! - **USB**: Periféricos hot-plug (teclado, mouse, storage)
//! - **ACPI**: Dispositivos declarados pela BIOS/UEFI
//! - **VirtIO**: Paravirtualização (QEMU, KVM)
//! - **Platform/ISA**: Dispositivos legados fixos (PIT, PS/2, Serial)

use super::device::Device;
use super::driver::DriverError;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// TIPOS DE BARRAMENTO
// =============================================================================

/// Tipos de barramento suportados pelo RDS.
///
/// Cada tipo tem características específicas:
/// - Método de endereçamento
/// - Capacidade de hot-plug
/// - Velocidade e largura de banda
/// - Recursos disponíveis (IRQ, MMIO, DMA)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    /// Dispositivos fixos da plataforma (PIT, RTC, UART legada).
    /// Não são descobertos dinamicamente - são conhecidos a priori.
    Platform,

    /// Peripheral Component Interconnect (PCI/PCIe).
    /// Principal barramento de expansão moderno.
    /// Suporta MMIO, MSI/MSI-X, DMA.
    Pci,

    /// Universal Serial Bus.
    /// Hot-plug, velocidades de 1.5Mbps a 20Gbps.
    /// Usa endpoints e transfers (control, bulk, interrupt, isochronous).
    Usb,

    /// Advanced Configuration and Power Interface.
    /// Dispositivos declarados em tabelas ACPI/DSDT.
    /// Inclui controle de energia e eventos.
    Acpi,

    /// Barramento virtual de I/O (VirtIO).
    /// Otimizado para VMs - baixa latência, alto throughput.
    /// Suporta vários tipos de dispositivos (blk, net, gpu, etc).
    Virtio,

    /// Barramento raiz do sistema (CPU → Chipset).
    /// Usado para controladores de interrupção, memória, etc.
    System,

    /// ISA/LPC - barramento legado.
    /// Portas seriais, PS/2, speaker, etc.
    Isa,
}

impl BusType {
    /// Retorna nome legível do tipo de barramento.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Platform => "Platform",
            Self::Pci => "PCI",
            Self::Usb => "USB",
            Self::Acpi => "ACPI",
            Self::Virtio => "VirtIO",
            Self::System => "System",
            Self::Isa => "ISA",
        }
    }

    /// Verifica se o barramento suporta hot-plug.
    pub fn supports_hotplug(&self) -> bool {
        matches!(self, Self::Usb | Self::Pci) // PCIe hot-plug em alguns casos
    }
}

// =============================================================================
// ENDEREÇO NO BARRAMENTO
// =============================================================================

/// Endereço abstrato de um dispositivo em um barramento.
///
/// Cada tipo de barramento usa um formato de endereçamento diferente:
/// - PCI: Bus/Device/Function (BDF)
/// - USB: Hub Address + Port
/// - MMIO: Endereço físico de memória
/// - I/O Port: Número da porta x86
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusAddress {
    /// Sem endereço específico (dispositivo virtual ou platform).
    None,

    /// Endereço PCI: Bus/Device/Function.
    /// Bus: 0-255, Device: 0-31, Function: 0-7.
    Pci {
        segment: u16, // Segmento PCIe (geralmente 0)
        bus: u8,      // Número do barramento
        device: u8,   // Número do slot
        function: u8, // Função dentro do slot
    },

    /// Endereço USB: Hub + Porta.
    /// Hierárquico - cada hub tem seu endereço.
    Usb {
        controller: u8, // ID do controlador host (xHCI instance)
        hub_addr: u8,   // Endereço do hub pai (0 = root hub)
        port: u8,       // Porta no hub
    },

    /// Memory-Mapped I/O: endereço físico base.
    Mmio {
        base: u64, // Endereço físico
        size: u64, // Tamanho da região
    },

    /// Porta de I/O x86.
    IoPort {
        port: u16,  // Número da porta
        count: u16, // Quantas portas consecutivas
    },

    /// Endereço VirtIO: transport específico.
    Virtio {
        transport: u8, // 0=MMIO, 1=PCI
        index: u8,     // Índice do dispositivo
    },
}

impl BusAddress {
    /// Formata o endereço para exibição em logs.
    pub fn format(&self) -> &'static str {
        // TODO: Implementar formatação dinâmica
        match self {
            Self::None => "none",
            Self::Pci { .. } => "pci:...",
            Self::Usb { .. } => "usb:...",
            Self::Mmio { .. } => "mmio:...",
            Self::IoPort { .. } => "io:...",
            Self::Virtio { .. } => "virtio:...",
        }
    }
}

// =============================================================================
// TRAIT PRINCIPAL: BUS
// =============================================================================

/// Trait fundamental para todos os barramentos de hardware.
///
/// Implementações desta trait são responsáveis por:
/// - Escanear o barramento e descobrir dispositivos
/// - Criar objetos Device para cada hardware encontrado
/// - Fornecer operações de reset e controle físico
/// - Gerenciar energia do barramento
pub trait Bus: Send + Sync {
    /// Retorna nome descritivo do barramento.
    ///
    /// Exemplo: "PCI Express Root Complex", "xHCI USB 3.0 Host"
    fn name(&self) -> &'static str;

    /// Retorna o tipo funcional do barramento.
    fn bus_type(&self) -> BusType;

    /// Escaneia o barramento em busca de dispositivos.
    ///
    /// Chamado durante a inicialização do sistema e também
    /// pode ser chamado para re-scan após hot-plug.
    ///
    /// ## Retorno:
    /// Lista de dispositivos encontrados, já com IDs e classificação.
    /// Cabe ao DriverManager registrar estes dispositivos.
    fn scan(&self) -> Vec<Device>;

    /// Realiza reset físico de um dispositivo específico.
    ///
    /// Chamado pelo RecoveryManager quando software reset falha.
    /// O dispositivo deve voltar ao estado inicial.
    ///
    /// ## Retorno:
    /// - true: Reset bem-sucedido
    /// - false: Reset não é suportado ou falhou
    fn reset_device(&self, dev: &mut Device) -> bool;

    /// Prepara o barramento para suspensão.
    ///
    /// Para todo tráfego de DMA e coloca dispositivos em espera.
    fn quiesce(&self) -> Result<(), DriverError> {
        // Implementação padrão vazia
        Ok(())
    }

    /// Acorda o barramento após suspensão.
    fn resume(&self) -> Result<(), DriverError> {
        // Implementação padrão vazia
        Ok(())
    }

    /// Desliga o barramento permanentemente.
    ///
    /// Chamado durante shutdown do sistema.
    fn shutdown(&self) {
        // Implementação padrão vazia
    }
}

// =============================================================================
// TRAIT ESTENDIDA: BUS OPERATIONS
// =============================================================================

/// Interface estendida para barramentos com operações de transação.
///
/// Alguns barramentos (USB, VirtIO) suportam operações assíncronas
/// onde o driver submete um comando e aguarda a resposta.
pub trait BusOperations: Bus {
    /// Submete uma transação assíncrona para o dispositivo.
    ///
    /// ## STUB - Não implementado ainda
    fn submit(&self, _dev_id: super::device::DeviceId, _cmd: &[u8]) -> TransactionId {
        crate::kwarn!("(Bus) submit() não implementado para este barramento!");
        TransactionId(0)
    }

    /// Verifica o status de uma transação pendente.
    fn poll(&self, _txn: TransactionId) -> TransactionStatus {
        TransactionStatus::Error(DriverError::NotSupported)
    }

    /// Cancela uma transação em andamento.
    fn cancel(&self, _txn: TransactionId) -> bool {
        false
    }
}

/// ID de uma transação assíncrona.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionId(pub u64);

/// Status de uma transação.
#[derive(Debug, Clone)]
pub enum TransactionStatus {
    /// Transação ainda em andamento.
    Pending,

    /// Transação completou com sucesso.
    /// Contém os dados de resposta (se houver).
    Complete(Vec<u8>),

    /// Transação falhou.
    Error(DriverError),
}

// =============================================================================
// REGISTRO GLOBAL DE BARRAMENTOS
// =============================================================================

/// Lista global de barramentos registrados no sistema.
/// Protegida por Spinlock para acesso thread-safe.
static BUS_REGISTRY: Spinlock<Vec<Arc<dyn Bus>>> = Spinlock::new(Vec::new());

/// Registra um novo barramento no sistema.
///
/// Chamado durante inicialização pelos módulos de barramento.
pub fn register(bus: Arc<dyn Bus>) {
    let name = bus.name();
    crate::kinfo!("(Bus) Registrando barramento:", name);
    BUS_REGISTRY.lock().push(bus);
}

/// Escaneia todos os barramentos registrados.
///
/// Retorna lista agregada de todos os dispositivos encontrados.
/// Chamado durante boot para popular o sistema com hardware conhecido.
pub fn scan_all() -> Vec<Device> {
    let mut all_devices = Vec::new();
    let buses = BUS_REGISTRY.lock();

    for bus in buses.iter() {
        crate::kinfo!("(Bus) Escaneando:", bus.name());
        let devices = bus.scan();
        crate::kinfo!("(Bus) Encontrados:", devices.len(), "dispositivos");
        all_devices.extend(devices);
    }

    all_devices
}

/// Busca um barramento específico pelo tipo.
///
/// Útil quando um driver precisa acessar operações específicas do barramento.
pub fn find_by_type(bus_type: BusType) -> Option<Arc<dyn Bus>> {
    BUS_REGISTRY
        .lock()
        .iter()
        .find(|b| b.bus_type() == bus_type)
        .cloned()
}

/// Retorna lista de todos os barramentos registrados.
pub fn get_all() -> Vec<Arc<dyn Bus>> {
    BUS_REGISTRY.lock().clone()
}

/// Desliga todos os barramentos de forma ordenada.
///
/// Chamado durante shutdown do sistema.
pub fn shutdown_all() {
    crate::kinfo!("(Bus) Iniciando shutdown de todos os barramentos...");
    let buses = BUS_REGISTRY.lock();

    for bus in buses.iter() {
        crate::kinfo!("(Bus) Desligando:", bus.name());
        bus.shutdown();
    }
}

/// Inicializa o subsistema de barramentos.
///
/// Chamado pelo mod.rs principal durante init().
pub fn init() {
    crate::kinfo!("(Bus) Subsistema de barramentos inicializado");
    // Barramentos específicos se registram durante seus próprios init()
}
