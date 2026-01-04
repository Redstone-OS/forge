//! # Camada de Barramento (Bus Layer)
//!
//! Define como o kernel interage com diferentes meios de transporte de dados.
//! Barramentos são responsáveis pela descoberta física e isolamento elétrico/lógico.
//!
//! ## Papel do Barramento:
//! - **Enumerar**: Escanear o hardware físico (ex: percorrer slots PCI ou portas USB).
//! - **Identificar**: Criar objetos `Device` com as informações cruciais para o matching.
//! - **Gestão de Sinais**: Implementar resets de hardware e mudanças de velocidade.

use super::device::Device;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Tipos de barramento suportados pelo RDM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    Platform, // Dispositivos fixos (PIT, RTC, UART legada)
    Pci,      // Peripheral Component Interconnect (PCI/PCIe)
    Usb,      // Universal Serial Bus
    Acpi,     // Dispositivos via tabelas ACPI
    Virtio,   // Barramento virtual de IO
    System,   // Barramento raiz do processador
}

/// Endereço abstrato em um barramento
/// Cada barramento usa uma forma de endereçamento (PCI usa BDF, USB usa Port/Address)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusAddress {
    None,
    Pci { bus: u8, dev: u8, func: u8 },
    Usb { hub_addr: u8, port: u8 },
    Mmio(u64),
    IoPort(u16),
}

/// Interface fundamental que todo barramento de hardware deve implementar
pub trait Bus: Send + Sync {
    /// Nome descritivo (ex: "PCI Express Root Complex")
    fn name(&self) -> &'static str;

    /// Tipo funcional do barramento
    fn bus_type(&self) -> BusType;

    /// Escaneia o barramento em busca de novos dispositivos.
    /// Retorna uma lista de dispositivos prontos para registro.
    fn scan(&self) -> Vec<Device>;

    /// Realiza um reset no barramento inteiro ou em um dispositivo específico
    fn reset_device(&self, dev: &mut Device) -> bool;

    /// Quiesce: Prepara o barramento para suspensão (para tráfego de DMA)
    fn quiesce(&self) -> Result<(), super::driver::DriverError> {
        Ok(())
    }

    /// Resume: Acorda o barramento após suspensão
    fn resume(&self) -> Result<(), super::driver::DriverError> {
        Ok(())
    }

    /// Shutdown: Desliga o barramento permanentemente
    fn shutdown(&self) {
        // Implementação padrão vazia
    }
}

// =============================================================================
// GERENCIADOR DE BARRAMENTOS (BUS REGISTRY)
// =============================================================================

static BUS_REGISTRY: Spinlock<Vec<Arc<dyn Bus>>> = Spinlock::new(Vec::new());

/// Registra um novo barramento no sistema
pub fn register(bus: Arc<dyn Bus>) {
    crate::kinfo!("(Bus) Registrando barramento:", bus.name());
    BUS_REGISTRY.lock().push(bus);
}

/// Escaneia todos os barramentos registrados e retorna todos os novos dispositivos encontrados
pub fn scan_all() -> Vec<Device> {
    let mut all_found = Vec::new();
    let buses = BUS_REGISTRY.lock();

    for bus in buses.iter() {
        crate::kinfo!("(Bus) Iniciando scan em:", bus.name());
        let mut devices = bus.scan();
        all_found.append(&mut devices);
    }

    all_found
}

/// Busca um barramento específico pelo tipo
pub fn find_by_type(bus_type: BusType) -> Option<Arc<dyn Bus>> {
    BUS_REGISTRY
        .lock()
        .iter()
        .find(|b| b.bus_type() == bus_type)
        .cloned()
}

/// Desliga todos os barramentos (chamado no power-off)
pub fn shutdown_all() {
    let buses = BUS_REGISTRY.lock();
    for bus in buses.iter() {
        bus.shutdown();
    }
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. isolamento de DMA (IOMMU):
//    - Implementar janelas de proteção de memória para evitar que dispositivos
//      bugados acessem memória indevida via DMA.
//
// 2. Interrupções de Mensagem (MSI/MSI-X):
//    - Evoluir o roteamento de IRQs para suporte a vetores MSI, permitindo que
//      dispositivos modernos sinalizem eventos diretamente para CPUs específicas.
//
// 3. Suporte a Pontes (Bridges):
//    - Implementar recursividade no scan para detectar barramentos secundários
//      (ex: PCI-to-PCI, Hubs USB dentro de hubs).
//
// 4. Hotplug Dinâmico:
//    - Sistema de traps de eventos para descoberta automática sem necessidade
//      de scan manual (polling vs interrupts).
//
// 5. Erros Avançados (AER):
//    - Monitoramento de qualidade de sinal físico via barramento para degradação
//      controlada de velocidade em caso de falha de hardware.
//
// 6. Virtualização de Hardware (SR-IOV):
//    - Suporte para expor Virtual Functions de hardware real diretamente para
//      Virtual Machines (guests).
