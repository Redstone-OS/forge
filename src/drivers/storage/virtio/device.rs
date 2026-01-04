//! # VirtIO Block Device
//!
//! Implementação do dispositivo VirtIO Block.

use super::regs::{blk_status, blk_type, offsets, status, SECTOR_SIZE};
use super::request::BlkReqHeader;
use crate::drivers::block::traits::{BlockDevice, BlockError};
use crate::drivers::block::virtqueue::{desc_flags, Virtqueue, QUEUE_SIZE};
use crate::drivers::pci::PciDevice;
use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Spinlock;
use core::sync::atomic::{fence, Ordering};

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
            req_header: Spinlock::new(BlkReqHeader::new()),
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
            self.write_reg8(offsets::DEVICE_STATUS, status::RESET);

            // 2. Set ACKNOWLEDGE
            self.write_reg8(offsets::DEVICE_STATUS, status::ACKNOWLEDGE);

            // 3. Set DRIVER
            self.write_reg8(offsets::DEVICE_STATUS, status::ACKNOWLEDGE | status::DRIVER);

            // 4. Ler features do dispositivo
            let device_features = self.read_reg32(offsets::DEVICE_FEATURES);
            crate::kdebug!("(VirtIO-BLK) Features:", device_features as u64);

            // 5. Negociar features (aceitar todas por enquanto)
            self.write_reg32(offsets::DRIVER_FEATURES, device_features);

            // 6. Set FEATURES_OK
            self.write_reg8(
                offsets::DEVICE_STATUS,
                status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK,
            );

            // 7. Verificar se FEATURES_OK foi aceito
            let current_status = self.read_reg8(offsets::DEVICE_STATUS);
            if current_status & status::FEATURES_OK == 0 {
                crate::kerror!("(VirtIO-BLK) Features não aceitas!");
                return false;
            }

            // 8. Configurar virtqueue
            self.write_reg16(offsets::QUEUE_SELECT, 0); // Selecionar queue 0

            let queue_size = self.read_reg16(offsets::QUEUE_SIZE);
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
            self.write_reg32(offsets::QUEUE_ADDRESS, queue_pfn as u32);

            *self.queue.lock() = Some(queue);

            // 9. Ler capacidade do disco
            let cap_lo = self.read_reg32(offsets::BLK_CAPACITY);
            let cap_hi = self.read_reg32(offsets::BLK_CAPACITY + 4);
            self.total_sectors = ((cap_hi as u64) << 32) | (cap_lo as u64);

            // 10. Set DRIVER_OK
            self.write_reg8(
                offsets::DEVICE_STATUS,
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
    #[allow(dead_code)]
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
        self.write_reg16(offsets::QUEUE_NOTIFY, 0);
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
        let io_status = *self.status_buf.lock();
        match io_status {
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
