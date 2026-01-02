//! # Barramento PCI
//!
//! Driver para enumeração e acesso ao barramento PCI.
//!
//! ## Funcionalidades
//!
//! - Acesso ao espaço de configuração PCI
//! - Enumeração de dispositivos
//! - Detecção de VirtIO devices
//!
//! ## Uso
//!
//! ```ignore
//! // Escanear barramento (chamado pelo init de drivers)
//! pci::scan();
//!
//! // Encontrar dispositivo VirtIO Block
//! if let Some(dev) = pci::find_virtio_blk() {
//!     println!("VirtIO Block encontrado!");
//! }
//! ```

pub mod config;
pub mod pci;

pub use pci::{
    all_devices, find_device, find_virtio_blk, scan, PciDevice, DEVICE_VIRTIO_BLK,
    DEVICE_VIRTIO_NET, VENDOR_REDHAT,
};
