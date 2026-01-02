//! # Driver VirtIO Block Device
//!
//! Implementa virtio-blk para discos virtuais do QEMU.
//!
//! ## Referências
//!
//! - [Especificação VirtIO 1.1](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html)
//! - Dispositivo virtio-blk do QEMU
//!
//! ## Funcionamento
//!
//! O VirtIO é um padrão de paravirtualização que permite comunicação
//! eficiente entre guest e host. O dispositivo aparece no barramento PCI
//! com vendor=0x1AF4 (Red Hat/Virtio) e device=0x1001 (block).
//!
//! ## Protocolo VirtIO Block
//!
//! ```text
//! Request:
//! ┌─────────────────┬─────────────────┬─────────────────┐
//! │  Header (16B)   │   Data (512B)   │   Status (1B)   │
//! │  type, sector   │   (R/W buffer)  │   (resultado)   │
//! └─────────────────┴─────────────────┴─────────────────┘
//! ```

#![allow(dead_code)]

use super::traits::{BlockDevice, BlockError};
use super::virtqueue::{desc_flags, Virtqueue, QUEUE_SIZE};
use crate::drivers::pci::{self, PciDevice};
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Spinlock;
use alloc::sync::Arc;
use core::sync::atomic::{fence, Ordering};

/// Tamanho padrão de setor
const SECTOR_SIZE: usize = 512;

/// Tipos de operação VirtIO Block
mod blk_type {
    pub const IN: u32 = 0; // Leitura
    pub const OUT: u32 = 1; // Escrita
}

/// Status de resposta VirtIO Block
mod blk_status {
    pub const OK: u8 = 0;
    pub const IOERR: u8 = 1;
    pub const UNSUPP: u8 = 2;
}

/// Offsets dos registradores VirtIO Legacy (BAR0)
mod regs {
    pub const DEVICE_FEATURES: u64 = 0x00;
    pub const DRIVER_FEATURES: u64 = 0x04;
    pub const QUEUE_ADDRESS: u64 = 0x08;
    pub const QUEUE_SIZE: u64 = 0x0C;
    pub const QUEUE_SELECT: u64 = 0x0E;
    pub const QUEUE_NOTIFY: u64 = 0x10;
    pub const DEVICE_STATUS: u64 = 0x12;
    pub const ISR_STATUS: u64 = 0x13;
    // Configuração específica do blk (offset 0x14+)
    pub const BLK_CAPACITY: u64 = 0x14; // 8 bytes
}

/// Status do dispositivo VirtIO
mod status {
    pub const RESET: u8 = 0;
    pub const ACKNOWLEDGE: u8 = 1;
    pub const DRIVER: u8 = 2;
    pub const DRIVER_OK: u8 = 4;
    pub const FEATURES_OK: u8 = 8;
    pub const FAILED: u8 = 128;
}

/// Header do request VirtIO Block (16 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct BlkReqHeader {
    /// Tipo de operação (IN=0, OUT=1)
    req_type: u32,
    /// Reservado
    reserved: u32,
    /// Setor (LBA)
    sector: u64,
}

/// Dispositivo de Bloco VirtIO
pub struct VirtioBlk {
    /// Dispositivo PCI associado
    pci_device: PciDevice,
    /// Endereço base dos registradores
    mmio_base: VirtAddr,
    /// Total de setores
    total_sectors: u64,
    /// Virtqueue para requisições
    queue: Spinlock<Option<Virtqueue>>,
    /// Buffer para request header (alocado uma vez)
    req_header: Spinlock<BlkReqHeader>,
    /// Buffer para status (1 byte)
    status_buf: Spinlock<u8>,
    /// Se o dispositivo foi inicializado com sucesso
    initialized: bool,
}

// SAFETY: VirtioBlk usa locking interno
unsafe impl Send for VirtioBlk {}
unsafe impl Sync for VirtioBlk {}

impl VirtioBlk {
    /// Cria e inicializa um dispositivo virtio-blk a partir de um PciDevice
    pub fn new(pci_device: PciDevice) -> Option<Self> {
        crate::kinfo!("(VirtIO-BLK) Inicializando dispositivo...");

        // Habilitar Bus Master e Memory Space
        pci_device.enable_bus_master();
        pci_device.enable_memory_space();

        // Obter endereço MMIO do BAR0
        let mmio_addr = pci_device.bar_address(0)?;
        let mmio_base = VirtAddr::new(mmio_addr);

        crate::kinfo!("(VirtIO-BLK) MMIO base:", mmio_addr);

        let mut device = Self {
            pci_device,
            mmio_base,
            total_sectors: 0,
            queue: Spinlock::new(None),
            req_header: Spinlock::new(BlkReqHeader {
                req_type: 0,
                reserved: 0,
                sector: 0,
            }),
            status_buf: Spinlock::new(0),
            initialized: false,
        };

        // Inicializar o dispositivo seguindo o protocolo VirtIO
        if device.init_device() {
            device.initialized = true;
            crate::kinfo!("(VirtIO-BLK) Inicializado com sucesso!");
            crate::kinfo!("(VirtIO-BLK) Capacidade:", device.total_sectors);
            Some(device)
        } else {
            crate::kerror!("(VirtIO-BLK) Falha na inicialização!");
            None
        }
    }

    /// Inicializa o dispositivo VirtIO
    fn init_device(&mut self) -> bool {
        unsafe {
            // 1. Reset (escrever 0 no status)
            self.write_reg8(regs::DEVICE_STATUS, status::RESET);

            // 2. Set ACKNOWLEDGE
            self.write_reg8(regs::DEVICE_STATUS, status::ACKNOWLEDGE);

            // 3. Set DRIVER
            self.write_reg8(regs::DEVICE_STATUS, status::ACKNOWLEDGE | status::DRIVER);

            // 4. Ler features do dispositivo
            let device_features = self.read_reg32(regs::DEVICE_FEATURES);
            crate::kdebug!("(VirtIO-BLK) Features:", device_features as u64);

            // 5. Negociar features (aceitar todas por enquanto)
            self.write_reg32(regs::DRIVER_FEATURES, device_features);

            // 6. Set FEATURES_OK
            self.write_reg8(
                regs::DEVICE_STATUS,
                status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK,
            );

            // 7. Verificar se FEATURES_OK foi aceito
            let status = self.read_reg8(regs::DEVICE_STATUS);
            if status & status::FEATURES_OK == 0 {
                crate::kerror!("(VirtIO-BLK) Features não aceitas!");
                return false;
            }

            // 8. Configurar virtqueue
            self.write_reg16(regs::QUEUE_SELECT, 0); // Selecionar queue 0

            let queue_size = self.read_reg16(regs::QUEUE_SIZE);
            crate::kdebug!("(VirtIO-BLK) Queue size:", queue_size as u64);

            if queue_size == 0 {
                crate::kerror!("(VirtIO-BLK) Queue size inválido!");
                return false;
            }

            // Criar virtqueue
            let queue = match Virtqueue::new(queue_size.min(QUEUE_SIZE)) {
                Some(q) => q,
                None => {
                    crate::kerror!("(VirtIO-BLK) Falha ao criar virtqueue!");
                    return false;
                }
            };

            // Passar endereço físico da queue para o dispositivo
            // O endereço é dividido por 4096 (page size)
            let queue_pfn = queue.phys_addr().as_u64() / 4096;
            self.write_reg32(regs::QUEUE_ADDRESS, queue_pfn as u32);

            *self.queue.lock() = Some(queue);

            // 9. Ler capacidade do disco
            let cap_lo = self.read_reg32(regs::BLK_CAPACITY);
            let cap_hi = self.read_reg32(regs::BLK_CAPACITY + 4);
            self.total_sectors = ((cap_hi as u64) << 32) | (cap_lo as u64);

            // 10. Set DRIVER_OK
            self.write_reg8(
                regs::DEVICE_STATUS,
                status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK | status::DRIVER_OK,
            );

            true
        }
    }

    /// Lê um registrador (32 bits)
    #[inline]
    unsafe fn read_reg32(&self, offset: u64) -> u32 {
        let addr = self.mmio_base.as_u64() + offset;
        core::ptr::read_volatile(addr as *const u32)
    }

    /// Escreve em um registrador (32 bits)
    #[inline]
    unsafe fn write_reg32(&self, offset: u64, value: u32) {
        let addr = self.mmio_base.as_u64() + offset;
        core::ptr::write_volatile(addr as *mut u32, value);
    }

    /// Lê um registrador (16 bits)
    #[inline]
    unsafe fn read_reg16(&self, offset: u64) -> u16 {
        let addr = self.mmio_base.as_u64() + offset;
        core::ptr::read_volatile(addr as *const u16)
    }

    /// Escreve em um registrador (16 bits)
    #[inline]
    unsafe fn write_reg16(&self, offset: u64, value: u16) {
        let addr = self.mmio_base.as_u64() + offset;
        core::ptr::write_volatile(addr as *mut u16, value);
    }

    /// Lê um registrador (8 bits)
    #[inline]
    unsafe fn read_reg8(&self, offset: u64) -> u8 {
        let addr = self.mmio_base.as_u64() + offset;
        core::ptr::read_volatile(addr as *const u8)
    }

    /// Escreve em um registrador (8 bits)
    #[inline]
    unsafe fn write_reg8(&self, offset: u64, value: u8) {
        let addr = self.mmio_base.as_u64() + offset;
        core::ptr::write_volatile(addr as *mut u8, value);
    }

    /// Notifica o dispositivo de uma nova requisição
    #[inline]
    unsafe fn notify(&self) {
        self.write_reg16(regs::QUEUE_NOTIFY, 0);
    }

    /// Executa uma operação de I/O
    fn do_io(&self, sector: u64, buf: &mut [u8], is_write: bool) -> Result<(), BlockError> {
        let mut queue_guard = self.queue.lock();
        let queue = queue_guard.as_mut().ok_or(BlockError::NotFound)?;

        // Preparar header
        let mut header = self.req_header.lock();
        header.req_type = if is_write {
            blk_type::OUT
        } else {
            blk_type::IN
        };
        header.reserved = 0;
        header.sector = sector;

        // Reset status
        *self.status_buf.lock() = 0xFF;

        // Alocar 3 descritores: header, data, status
        let desc0 = queue.alloc_desc().ok_or(BlockError::Busy)?;
        let desc1 = queue.alloc_desc().ok_or(BlockError::Busy)?;
        let desc2 = queue.alloc_desc().ok_or(BlockError::Busy)?;

        // Configurar descritores
        // Desc 0: Header (device-readable)
        let header_ptr = &*header as *const BlkReqHeader;
        queue.set_desc(
            desc0,
            PhysAddr::new(header_ptr as u64),
            core::mem::size_of::<BlkReqHeader>() as u32,
            desc_flags::NEXT,
            desc1,
        );

        // Desc 1: Data buffer
        let data_flags = if is_write {
            desc_flags::NEXT // readable pelo device (write)
        } else {
            desc_flags::NEXT | desc_flags::WRITE // writable pelo device (read)
        };
        queue.set_desc(
            desc1,
            PhysAddr::new(buf.as_ptr() as u64),
            buf.len() as u32,
            data_flags,
            desc2,
        );

        // Desc 2: Status (device-writable)
        let status_ptr = &*self.status_buf.lock() as *const u8;
        queue.set_desc(
            desc2,
            PhysAddr::new(status_ptr as u64),
            1,
            desc_flags::WRITE,
            0,
        );

        // Adicionar ao available ring
        queue.push_avail(desc0);

        // Memory barrier
        fence(Ordering::SeqCst);

        // Notificar dispositivo
        unsafe {
            self.notify();
        }

        // Aguardar completion (polling)
        let mut timeout = 1_000_000u32;
        while !queue.has_used() && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }

        if timeout == 0 {
            crate::kerror!("(VirtIO-BLK) Timeout na operação!");
            // Liberar descritores
            queue.free_desc(desc0);
            queue.free_desc(desc1);
            queue.free_desc(desc2);
            return Err(BlockError::IoError);
        }

        // Pop do used ring
        let _ = queue.pop_used();

        // Liberar descritores
        queue.free_desc(desc0);
        queue.free_desc(desc1);
        queue.free_desc(desc2);

        // Verificar status
        let status = *self.status_buf.lock();
        match status {
            blk_status::OK => Ok(()),
            blk_status::IOERR => Err(BlockError::IoError),
            _ => Err(BlockError::HardwareError),
        }
    }
}

impl BlockDevice for VirtioBlk {
    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        if !self.initialized {
            return Err(BlockError::NotFound);
        }

        if lba >= self.total_sectors {
            return Err(BlockError::InvalidBlock);
        }
        if buf.len() < SECTOR_SIZE {
            return Err(BlockError::InvalidBuffer);
        }

        self.do_io(lba, buf, false)
    }

    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError> {
        if !self.initialized {
            return Err(BlockError::NotFound);
        }

        if lba >= self.total_sectors {
            return Err(BlockError::InvalidBlock);
        }
        if buf.len() < SECTOR_SIZE {
            return Err(BlockError::InvalidBuffer);
        }

        // Cast para &mut [u8] é necessário pela interface
        // SAFETY: O dispositivo não modifica o buffer em writes
        let buf_mut =
            unsafe { core::slice::from_raw_parts_mut(buf.as_ptr() as *mut u8, buf.len()) };
        self.do_io(lba, buf_mut, true)
    }

    fn block_size(&self) -> usize {
        SECTOR_SIZE
    }

    fn total_blocks(&self) -> u64 {
        self.total_sectors
    }
}

/// Tenta inicializar dispositivo virtio-blk
///
/// Escaneia o PCI procurando por dispositivo VirtIO Block e inicializa.
pub fn init() -> Option<Arc<dyn BlockDevice>> {
    crate::kinfo!("(VirtIO-BLK) Procurando dispositivo...");

    // Procurar dispositivo VirtIO Block no PCI
    let pci_device = pci::find_virtio_blk()?;

    crate::kinfo!("(VirtIO-BLK) Dispositivo encontrado!");
    crate::kinfo!("  Bus:", pci_device.bus as u64);
    crate::kinfo!("  Device:", pci_device.device as u64);
    crate::kinfo!("  Function:", pci_device.function as u64);

    // Criar e inicializar driver
    let device = VirtioBlk::new(pci_device)?;

    Some(Arc::new(device))
}
