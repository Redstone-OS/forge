//! # Storage Device Traits and Types
//!
//! Define as interfaces e tipos fundamentais para dispositivos de armazenamento.
//!
//! ## Arquitetura RDS:
//! Todos os drivers de storage implementam `BlockDevice` e são registrados
//! no subsistema central via `storage::register_device()`.
//!
//! ## Tipos de Dispositivos:
//! - **AHCI**: SATA via AHCI (moderno)
//! - **NVMe**: NVMe SSDs via PCIe
//! - **ATA**: Legacy PATA/IDE
//! - **VirtIO**: Paravirtualizado (QEMU/KVM)
//! - **Ramdisk**: Disco em memória

use alloc::string::String;
use alloc::sync::Arc;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use alloc::vec::Vec;

// =============================================================================
// ERROS DE BLOCO
// =============================================================================

/// Erros de operações de bloco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    /// Dispositivo não encontrado.
    NotFound,
    /// Dispositivo não inicializado.
    NotInitialized,
    /// Dispositivo não está pronto.
    NotReady,
    /// Bloco fora do range válido.
    InvalidBlock,
    /// LBA inválido.
    InvalidLba,
    /// Buffer inválido ou tamanho incorreto.
    InvalidBuffer,
    /// Dispositivo é somente leitura.
    ReadOnly,
    /// Erro de I/O durante transferência.
    IoError,
    /// Timeout na operação.
    Timeout,
    /// Dispositivo ocupado.
    Busy,
    /// Erro de hardware.
    HardwareError,
    /// Erro de CRC/checksum.
    CrcError,
    /// Mídia removida.
    MediaRemoved,
    /// Operação não suportada.
    NotSupported,
    /// Erro desconhecido.
    Unknown,
}

impl BlockError {
    /// Retorna descrição do erro.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotFound => "Device Not Found",
            Self::NotInitialized => "Device Not Initialized",
            Self::NotReady => "Device Not Ready",
            Self::InvalidBlock => "Invalid Block Number",
            Self::InvalidLba => "Invalid LBA",
            Self::InvalidBuffer => "Invalid Buffer",
            Self::ReadOnly => "Device is Read-Only",
            Self::IoError => "I/O Error",
            Self::Timeout => "Operation Timeout",
            Self::Busy => "Device Busy",
            Self::HardwareError => "Hardware Error",
            Self::CrcError => "CRC/Checksum Error",
            Self::MediaRemoved => "Media Removed",
            Self::NotSupported => "Operation Not Supported",
            Self::Unknown => "Unknown Error",
        }
    }

    /// Verifica se é erro recuperável.
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Busy | Self::Timeout | Self::NotReady)
    }
}

// =============================================================================
// TIPOS DE DISPOSITIVO
// =============================================================================

/// Tipo de dispositivo de armazenamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// Disco rígido (HDD).
    Hdd,
    /// Solid State Drive.
    Ssd,
    /// NVMe SSD.
    Nvme,
    /// Disco óptico (CD/DVD).
    Optical,
    /// Disco virtual (VirtIO, Ramdisk).
    Virtual,
    /// Removível (USB, SD Card).
    Removable,
    /// Tipo desconhecido.
    Unknown,
}

/// Interface/protocolo do dispositivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageInterface {
    /// PATA/IDE (legacy).
    Ata,
    /// SATA via AHCI.
    Ahci,
    /// NVMe via PCIe.
    Nvme,
    /// VirtIO paravirtualizado.
    Virtio,
    /// Disco em memória.
    Ramdisk,
    /// USB Mass Storage.
    Usb,
    /// SCSI.
    Scsi,
}

// =============================================================================
// CAPACIDADES DO DISPOSITIVO
// =============================================================================

/// Capacidades de um dispositivo de storage.
#[derive(Debug, Clone, Default)]
pub struct StorageCapabilities {
    /// Suporta escrita.
    pub writable: bool,
    /// Suporta flush/sync.
    pub flush: bool,
    /// Suporta discard/trim.
    pub discard: bool,
    /// Suporta write zeros.
    pub write_zeros: bool,
    /// Suporta scatter-gather I/O.
    pub scatter_gather: bool,
    /// Suporta I/O assíncrono.
    pub async_io: bool,
    /// Tamanho máximo de uma operação (blocos).
    pub max_transfer_blocks: u32,
    /// Número de filas de I/O.
    pub io_queues: u16,
}

// =============================================================================
// ESTATÍSTICAS DO DISPOSITIVO
// =============================================================================

/// Estatísticas de um dispositivo de storage.
#[derive(Debug, Clone, Copy, Default)]
pub struct StorageStats {
    /// Blocos lidos.
    pub blocks_read: u64,
    /// Blocos escritos.
    pub blocks_written: u64,
    /// Bytes lidos.
    pub bytes_read: u64,
    /// Bytes escritos.
    pub bytes_written: u64,
    /// Erros de leitura.
    pub read_errors: u32,
    /// Erros de escrita.
    pub write_errors: u32,
    /// Operações de flush.
    pub flushes: u32,
}

// =============================================================================
// INFORMAÇÕES DO DISPOSITIVO
// =============================================================================

/// Informações de identificação do dispositivo.
#[derive(Debug, Clone, Default)]
pub struct StorageInfo {
    /// Modelo do dispositivo.
    pub model: String,
    /// Número de série.
    pub serial: String,
    /// Versão do firmware.
    pub firmware: String,
    /// Tipo de dispositivo.
    pub device_type: Option<StorageType>,
    /// Interface.
    pub interface: Option<StorageInterface>,
}

// =============================================================================
// TRAIT PRINCIPAL: BLOCK DEVICE
// =============================================================================

/// Interface base para dispositivos de bloco.
///
/// Todo driver de storage deve implementar esta trait.
pub trait BlockDevice: Send + Sync {
    // -------------------------------------------------------------------------
    // Identificação
    // -------------------------------------------------------------------------

    /// Retorna nome do dispositivo (ex: "sda", "nvme0n1").
    fn name(&self) -> &str;

    /// Retorna informações detalhadas.
    fn info(&self) -> StorageInfo {
        StorageInfo::default()
    }

    // -------------------------------------------------------------------------
    // Geometria
    // -------------------------------------------------------------------------

    /// Tamanho do bloco em bytes (geralmente 512 ou 4096).
    fn block_size(&self) -> usize;

    /// Número total de blocos.
    fn total_blocks(&self) -> u64;

    /// Capacidade total em bytes.
    fn capacity(&self) -> u64 {
        self.total_blocks() * self.block_size() as u64
    }

    // -------------------------------------------------------------------------
    // Operações de I/O
    // -------------------------------------------------------------------------

    /// Lê um ou mais blocos do dispositivo.
    ///
    /// ## Parâmetros:
    /// - `lba`: Endereço lógico do primeiro bloco
    /// - `buf`: Buffer para receber os dados (deve ser múltiplo de block_size)
    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError>;

    /// Escreve um ou mais blocos no dispositivo.
    ///
    /// ## Parâmetros:
    /// - `lba`: Endereço lógico do primeiro bloco
    /// - `buf`: Buffer com os dados (deve ser múltiplo de block_size)
    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError>;

    /// Lê múltiplos blocos contíguos.
    fn read_blocks(&self, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), BlockError> {
        let bs = self.block_size();
        if buf.len() < count * bs {
            return Err(BlockError::InvalidBuffer);
        }
        for i in 0..count {
            let offset = i * bs;
            self.read_block(lba + i as u64, &mut buf[offset..offset + bs])?;
        }
        Ok(())
    }

    /// Escreve múltiplos blocos contíguos.
    fn write_blocks(&self, lba: u64, count: usize, buf: &[u8]) -> Result<(), BlockError> {
        let bs = self.block_size();
        if buf.len() < count * bs {
            return Err(BlockError::InvalidBuffer);
        }
        for i in 0..count {
            let offset = i * bs;
            self.write_block(lba + i as u64, &buf[offset..offset + bs])?;
        }
        Ok(())
    }

    /// Força escrita de dados em cache para o disco.
    fn flush(&self) -> Result<(), BlockError> {
        Ok(()) // Default: no-op
    }

    // -------------------------------------------------------------------------
    // Estado
    // -------------------------------------------------------------------------

    /// Verifica se é somente leitura.
    fn is_read_only(&self) -> bool {
        false
    }

    /// Verifica se o dispositivo está pronto.
    fn is_ready(&self) -> bool {
        true
    }

    /// Retorna capacidades do dispositivo.
    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            writable: !self.is_read_only(),
            ..Default::default()
        }
    }

    // -------------------------------------------------------------------------
    // Diagnóstico
    // -------------------------------------------------------------------------

    /// Retorna estatísticas do dispositivo.
    fn get_stats(&self) -> StorageStats {
        StorageStats::default()
    }

    /// Reseta estatísticas.
    fn reset_stats(&self) {}
}

/// Tipo wrapper para armazenar qualquer dispositivo de bloco.
pub type BlockDeviceRef = Arc<dyn BlockDevice>;
