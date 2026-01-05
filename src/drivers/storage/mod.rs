//! # Storage Drivers Layer
//!
//! Este módulo contém os drivers de armazenamento do RedstoneOS. Segue a
//! arquitetura do Redstone Driver System (RDS) para block devices.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Filesystems                │  (VFS, FAT, ext4)
//! ├─────────────────────────────────────────┤
//! │              Block Layer                │  (este módulo)
//! ├──────┬──────┬────────┬─────────┬────────┤
//! │ AHCI │ NVMe │ VirtIO │ Ramdisk │   ATA  │
//! ├──────┴──────┴────────┴─────────┴────────┤
//! │              drivers/base               │  (RDS Core)
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Drivers Implementados:
//!
//! | Driver    | Status | Descrição                    |
//! |-----------|--------|------------------------------|
//! | `ahci`    | Stub   | SATA via AHCI                |
//! | `nvme`    | Stub   | NVMe SSDs via PCIe           |
//! | `ata`     | Stub   | Legacy PATA/IDE              |
//! | `virtio`  | Stub   | VirtIO-Blk (QEMU/KVM)        |
//! | `ramdisk` | Func   | Disco em memória             |  
//!
//!
//! ## Integração RDS:
//! - Drivers implementam `BlockDevice` + `drivers::base::Driver`
//! - Registro via `storage::register_device()`
//! - Recuperação automática via `RecoveryManager`

pub mod ahci;
pub mod ata;
pub mod nvme;
pub mod ramdisk;
pub mod traits;
pub mod virtio;

// Re-exports
pub use traits::*;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::base::driver::Driver;
use crate::sync::Spinlock;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL DO SUBSISTEMA
// =============================================================================

/// Lista de dispositivos de bloco registrados.
static BLOCK_DEVICES: Spinlock<Vec<BlockDeviceRef>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

/// Dispositivo de boot.
static BOOT_DEVICE: Spinlock<Option<usize>> = Spinlock::new(None);

// =============================================================================
// INICIALIZAÇÃO DO SUBSISTEMA
// =============================================================================

/// Inicializa o subsistema de storage.
///
/// Registra todos os drivers de armazenamento no DriverManager.
pub fn init() {
    crate::kinfo!("(Storage) Inicializando subsistema de storage...");

    // Fase 1: Ramdisk (sempre disponível, útil para testes)
    ramdisk::init();

    // Fase 2: VirtIO (prioridade para VMs)
    //virtio::init();

    // Fase 3: NVMe (SSDs modernos)
    nvme::init();

    // Fase 4: AHCI/SATA
    ahci::init();

    // Fase 5: ATA Legacy (fallback)
    ata::init();

    *INITIALIZED.lock() = true;

    let count = device_count();
    crate::kinfo!(
        "(Storage) Subsistema inicializado: {} dispositivo(s)",
        count
    );
}

/// Desliga o subsistema de storage.
pub fn shutdown() {
    crate::kinfo!("(Storage) Shutdown do subsistema...");

    // Flush todos os dispositivos
    let devices = BLOCK_DEVICES.lock();
    for dev in devices.iter() {
        let _ = dev.flush();
    }

    crate::kinfo!("(Storage) Subsistema desligado");
}

// =============================================================================
// REGISTRO DE DISPOSITIVOS
// =============================================================================

/// Registra um novo dispositivo de bloco.
pub fn register_device(dev: BlockDeviceRef) {
    let name = dev.name();
    let capacity_mb = dev.capacity() / (1024 * 1024);
    crate::kinfo!("(Storage) Registrando: {} ({}MB)", name, capacity_mb);

    let mut devices = BLOCK_DEVICES.lock();
    devices.push(dev);
}

/// Remove um dispositivo de bloco.
pub fn unregister_device(name: &str) {
    crate::kinfo!("(Storage) Removendo: {}", name);
    BLOCK_DEVICES.lock().retain(|d| d.name() != name);
}

// =============================================================================
// CONSULTA DE DISPOSITIVOS
// =============================================================================

/// Retorna número de dispositivos registrados.
pub fn device_count() -> usize {
    BLOCK_DEVICES.lock().len()
}

/// Verifica se o subsistema está inicializado.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock()
}

/// Busca dispositivo por nome.
pub fn find_device(name: &str) -> Option<BlockDeviceRef> {
    BLOCK_DEVICES
        .lock()
        .iter()
        .find(|d| d.name() == name)
        .cloned()
}

/// Busca dispositivo por índice.
pub fn get_device(index: usize) -> Option<BlockDeviceRef> {
    BLOCK_DEVICES.lock().get(index).cloned()
}

/// Retorna lista de todos os dispositivos.
pub fn get_all_devices() -> Vec<BlockDeviceRef> {
    BLOCK_DEVICES.lock().clone()
}

/// Retorna o primeiro dispositivo disponível.
pub fn get_first_device() -> Option<BlockDeviceRef> {
    BLOCK_DEVICES.lock().first().cloned()
}

/// Define o dispositivo de boot.
pub fn set_boot_device(name: &str) -> bool {
    let devices = BLOCK_DEVICES.lock();
    if let Some(idx) = devices.iter().position(|d| d.name() == name) {
        *BOOT_DEVICE.lock() = Some(idx);
        crate::kinfo!("(Storage) Boot device: {}", name);
        true
    } else {
        false
    }
}

/// Retorna o dispositivo de boot.
pub fn get_boot_device() -> Option<BlockDeviceRef> {
    let boot = BOOT_DEVICE.lock();
    boot.and_then(|idx| BLOCK_DEVICES.lock().get(idx).cloned())
}

// =============================================================================
// ESTATÍSTICAS GLOBAIS
// =============================================================================

/// Estatísticas agregadas do subsistema.
#[derive(Debug, Clone, Default)]
pub struct GlobalStorageStats {
    /// Número de dispositivos.
    pub device_count: usize,
    /// Total de blocos lidos.
    pub total_blocks_read: u64,
    /// Total de blocos escritos.
    pub total_blocks_written: u64,
    /// Total de bytes lidos.
    pub total_bytes_read: u64,
    /// Total de bytes escritos.
    pub total_bytes_written: u64,
    /// Capacidade total.
    pub total_capacity: u64,
}

/// Retorna estatísticas agregadas.
pub fn get_global_stats() -> GlobalStorageStats {
    let devices = BLOCK_DEVICES.lock();
    let mut global = GlobalStorageStats {
        device_count: devices.len(),
        ..Default::default()
    };

    for dev in devices.iter() {
        let stats = dev.get_stats();
        global.total_blocks_read += stats.blocks_read;
        global.total_blocks_written += stats.blocks_written;
        global.total_bytes_read += stats.bytes_read;
        global.total_bytes_written += stats.bytes_written;
        global.total_capacity += dev.capacity();
    }

    global
}

// =============================================================================
// FUNÇÕES DE CONVENIÊNCIA
// =============================================================================

/// Lê blocos de um dispositivo por nome.
pub fn read(device: &str, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
    let dev = find_device(device).ok_or(BlockError::NotFound)?;
    dev.read_block(lba, buf)
}

/// Escreve blocos em um dispositivo por nome.
pub fn write(device: &str, lba: u64, buf: &[u8]) -> Result<(), BlockError> {
    let dev = find_device(device).ok_or(BlockError::NotFound)?;
    dev.write_block(lba, buf)
}
