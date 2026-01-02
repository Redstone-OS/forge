//! # Dispositivos de Bloco
//!
//! Drivers e camada de abstração para dispositivos de bloco.
//!
//! ## Dispositivos Suportados
//!
//! | Driver      | Status      | Descrição                    |
//! |-------------|-------------|------------------------------|
//! | VirtIO-BLK  | Em progresso| Disco paravirtualizado QEMU  |
//! | AHCI        | Planejado   | SATA/AHCI                    |
//! | NVMe        | Planejado   | NVMe SSDs                    |
//! | Ramdisk     | Planejado   | Disco em memória             |

pub mod ahci;
pub mod nvme;
pub mod ramdisk;
pub mod traits;
pub mod virtio_blk;

pub use traits::{BlockDevice, BlockDeviceInfo, BlockError};

use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Registro global de dispositivos de bloco
static BLOCK_DEVICES: Spinlock<Vec<Arc<dyn BlockDevice>>> = Spinlock::new(Vec::new());

/// Inicializa o subsistema de dispositivos de bloco
pub fn init() {
    crate::kinfo!("(Block) Inicializando subsistema de dispositivos de bloco...");

    // Tenta inicializar virtio-blk
    if let Some(device) = virtio_blk::init() {
        register_device(device);
    }

    let count = BLOCK_DEVICES.lock().len();
    crate::kinfo!("(Block) Dispositivos detectados:", count as u64);
}

/// Registra um novo dispositivo de bloco
pub fn register_device(device: Arc<dyn BlockDevice>) {
    BLOCK_DEVICES.lock().push(device);
}

/// Obtém um dispositivo de bloco pelo índice
pub fn get_device(index: usize) -> Option<Arc<dyn BlockDevice>> {
    BLOCK_DEVICES.lock().get(index).cloned()
}

/// Obtém o primeiro dispositivo de bloco disponível
pub fn first_device() -> Option<Arc<dyn BlockDevice>> {
    get_device(0)
}

/// Retorna o número total de dispositivos registrados
pub fn device_count() -> usize {
    BLOCK_DEVICES.lock().len()
}
