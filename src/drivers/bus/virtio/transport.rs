//! # VirtIO Transport Layer
//!
//! Abstração da camada de transporte VirtIO. Suporta:
//! - PCI (dispositivos modernos)
//! - MMIO (ARM, devicetree)
//!
//! O transport é responsável por:
//! - Acesso aos registradores de configuração
//! - Notificação de queues
//! - Negociação de features
//! - Gerenciamento de interrupções

// TODO: Revisar no futuro
#[allow(unused_imports)]
use super::types::*;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use super::{VirtioAddress, VirtioTransportType};
use crate::drivers::bus::pci::{self, PciAddress, PciDevice};

// =============================================================================
// TRAIT DE TRANSPORT
// =============================================================================

/// Interface que todo transport VirtIO deve implementar.
pub trait VirtioTransport: Send + Sync {
    /// Retorna tipo do dispositivo VirtIO.
    fn device_type(&self) -> u32;

    /// Lê o status do dispositivo.
    fn read_status(&self) -> u8;

    /// Escreve o status do dispositivo.
    fn write_status(&self, status: u8);

    /// Lê features oferecidas pelo dispositivo.
    fn read_device_features(&self) -> u64;

    /// Escreve features aceitas pelo driver.
    fn write_driver_features(&self, features: u64);

    /// Seleciona uma virtqueue.
    fn select_queue(&self, queue_index: u16);

    /// Retorna tamanho máximo da queue selecionada.
    fn queue_max_size(&self) -> u16;

    /// Define tamanho da queue selecionada.
    fn set_queue_size(&self, size: u16);

    /// Define endereço da queue.
    fn set_queue_address(&self, desc: u64, avail: u64, used: u64);

    /// Habilita a queue selecionada.
    fn enable_queue(&self);

    /// Notifica o dispositivo sobre novos descritores.
    fn notify(&self, queue_index: u16);

    /// Lê valor do espaço de configuração.
    fn read_config(&self, offset: usize) -> u32;

    /// Escreve valor no espaço de configuração.
    fn write_config(&self, offset: usize, value: u32);

    /// Realiza reset do dispositivo.
    fn reset(&self);
}

// =============================================================================
// PCI TRANSPORT
// =============================================================================

/// Transport VirtIO via PCI.
pub struct VirtioPciTransport {
    /// Dispositivo PCI associado.
    address: PciAddress,

    /// BAR usada para config.
    config_bar: usize,

    /// Base do common config.
    common_config_offset: u64,

    /// Base dos registradores de notificação.
    notify_offset: u64,

    /// Multiplicador de notificação.
    notify_multiplier: u32,

    /// Base do ISR status.
    isr_offset: u64,

    /// Base do device config.
    device_config_offset: u64,
}

impl VirtioPciTransport {
    /// Cria novo transport PCI.
    ///
    /// ## STUB:
    /// Implementação parcial. Necessita ler capabilities PCI para
    /// encontrar offsets corretos.
    pub fn new(pci_dev: &PciDevice) -> Option<Self> {
        crate::kwarn!("(VirtIO PCI) Transport parcialmente implementado");

        // TODO: Percorrer capabilities PCI para encontrar:
        // - VIRTIO_PCI_CAP_COMMON_CFG
        // - VIRTIO_PCI_CAP_NOTIFY_CFG
        // - VIRTIO_PCI_CAP_ISR_CFG
        // - VIRTIO_PCI_CAP_DEVICE_CFG

        // Por enquanto, retorna sem offsets (não funcional)
        Some(Self {
            address: pci_dev.address,
            config_bar: 0,
            common_config_offset: 0,
            notify_offset: 0,
            notify_multiplier: 4,
            isr_offset: 0,
            device_config_offset: 0,
        })
    }

    /// Lê registrador MMIO.
    // TODO: Revisar no futuro
    #[allow(dead_code, unused_variables)]
    fn read_reg(&self, offset: u64) -> u32 {
        // TODO: Implementar acesso real via BAR
        crate::kwarn!("(VirtIO PCI) read_reg() stub");
        0
    }

    /// Escreve registrador MMIO.
    // TODO: Revisar no futuro
    #[allow(dead_code, unused_variables)]
    fn write_reg(&self, offset: u64, value: u32) {
        // TODO: Implementar acesso real via BAR
        crate::kwarn!("(VirtIO PCI) write_reg() stub");
    }
}

impl VirtioTransport for VirtioPciTransport {
    fn device_type(&self) -> u32 {
        // Lê do config space PCI
        let dev_id = pci::read_config(self.address, 0x02) as u16;
        if dev_id >= 0x1040 {
            (dev_id - 0x1040) as u32
        } else if dev_id >= 0x1000 {
            (dev_id - 0x1000) as u32
        } else {
            0
        }
    }

    fn read_status(&self) -> u8 {
        crate::kwarn!("(VirtIO PCI) read_status() stub");
        0
    }

    fn write_status(&self, status: u8) {
        crate::kwarn!("(VirtIO PCI) write_status() stub, status=", status);
    }

    fn read_device_features(&self) -> u64 {
        crate::kwarn!("(VirtIO PCI) read_device_features() stub");
        0
    }

    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    fn write_driver_features(&self, features: u64) {
        crate::kwarn!("(VirtIO PCI) write_driver_features() stub");
    }

    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    fn select_queue(&self, queue_index: u16) {
        crate::kwarn!("(VirtIO PCI) select_queue() stub");
    }

    fn queue_max_size(&self) -> u16 {
        crate::kwarn!("(VirtIO PCI) queue_max_size() stub");
        256 // Valor padrão
    }

    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    fn set_queue_size(&self, size: u16) {
        crate::kwarn!("(VirtIO PCI) set_queue_size() stub");
    }

    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    fn set_queue_address(&self, desc: u64, avail: u64, used: u64) {
        crate::kwarn!("(VirtIO PCI) set_queue_address() stub");
    }

    fn enable_queue(&self) {
        crate::kwarn!("(VirtIO PCI) enable_queue() stub");
    }

    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    fn notify(&self, queue_index: u16) {
        crate::kwarn!("(VirtIO PCI) notify() stub");
    }

    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    fn read_config(&self, offset: usize) -> u32 {
        crate::kwarn!("(VirtIO PCI) read_config() stub");
        0
    }

    // TODO: Revisar no futuro
    #[allow(unused_variables)]
    fn write_config(&self, offset: usize, value: u32) {
        crate::kwarn!("(VirtIO PCI) write_config() stub");
    }

    fn reset(&self) {
        self.write_status(0);
    }
}

// =============================================================================
// CAPABILITY IDS - PCI VIRTIO
// =============================================================================

/// Common configuration.
pub const VIRTIO_PCI_CAP_COMMON_CFG: u8 = 1;

/// Notifications.
pub const VIRTIO_PCI_CAP_NOTIFY_CFG: u8 = 2;

/// ISR status.
pub const VIRTIO_PCI_CAP_ISR_CFG: u8 = 3;

/// Device specific config.
pub const VIRTIO_PCI_CAP_DEVICE_CFG: u8 = 4;

/// PCI config access.
pub const VIRTIO_PCI_CAP_PCI_CFG: u8 = 5;
