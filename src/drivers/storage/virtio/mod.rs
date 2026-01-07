//! # VirtIO Block Driver
//!
//! Driver para discos paravirtualizados VirtIO (virtio-blk).
//! Alta performance em máquinas virtuais QEMU/KVM.
//!
//! ## Spec: OASIS VirtIO v1.2 - Section 5.2 Block Device
//!
//! ## Arquitetura
//!
//! Este driver segue o modelo RDS correto:
//! 1. Registra o `VirtioBlkDriver` no RDS
//! 2. O RDS faz matching automático quando encontra dispositivo VirtIO-blk
//! 3. Driver usa DMA Pool centralizado para buffers de I/O
//!
//! ## Modo de Operação
//!
//! Usa modo **Legacy VirtIO** com acesso via portas I/O (BAR0).
//! O modo moderno (MMIO via PCI Capabilities) pode ser implementado no futuro.
//!
//! ## Registradores Legacy (BAR0 I/O Port)
//!
//! | Offset | Size | R/W | Registrador               |
//! |--------|------|-----|---------------------------|
//! | 0x00   | 4    | R   | Device Features           |
//! | 0x04   | 4    | W   | Driver Features           |
//! | 0x08   | 4    | RW  | Queue Address (PFN)       |
//! | 0x0C   | 2    | R   | Queue Size                |
//! | 0x0E   | 2    | W   | Queue Select              |
//! | 0x10   | 2    | W   | Queue Notify              |
//! | 0x12   | 1    | RW  | Device Status             |
//! | 0x13   | 1    | R   | ISR Status                |
//! | 0x14+  | var  | R   | Device Config             |

use crate::drivers::base::device::Device;
use crate::drivers::base::dma::{self, DmaBuffer, DmaDirection};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::drivers::bus::pci::{self, PciAddress};
use crate::drivers::bus::virtio::types::*;
use crate::drivers::storage::traits::*;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use core::ptr;
use core::sync::atomic::{fence, Ordering};

// =============================================================================
// CONSTANTES
// =============================================================================

const VIRTIO_VENDOR: u16 = 0x1AF4;
const VIRTIO_BLK_DEVICE_LEGACY: u16 = 0x1001;
const VIRTIO_BLK_DEVICE_MODERN: u16 = 0x1042; // 0x1040 + device_type(2)

/// Tamanho máximo da VirtQueue suportado pelo driver.
const VIRTQUEUE_MAX_SIZE: u16 = 256;

/// Tamanho de um setor.
const SECTOR_SIZE: usize = 512;

// Offsets de registradores I/O (Legacy VirtIO)
const REG_DEVICE_FEATURES: u16 = 0x00;
const REG_DRIVER_FEATURES: u16 = 0x04;
const REG_QUEUE_ADDRESS: u16 = 0x08;
const REG_QUEUE_SIZE: u16 = 0x0C;
const REG_QUEUE_SELECT: u16 = 0x0E;
const REG_QUEUE_NOTIFY: u16 = 0x10;
const REG_DEVICE_STATUS: u16 = 0x12;
// TODO: Será usado para interrupt handling
#[allow(dead_code)]
const REG_ISR_STATUS: u16 = 0x13;
const REG_CONFIG: u16 = 0x14;

// =============================================================================
// DRIVER RDS
// =============================================================================

/// Driver VirtIO Block para o RDS.
pub struct VirtioBlkDriver;

impl Driver for VirtioBlkDriver {
    fn name(&self) -> &'static str {
        "virtio-blk"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Storage
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        let is_virtio_blk = dev.vendor_id == VIRTIO_VENDOR
            && (dev.device_id == VIRTIO_BLK_DEVICE_LEGACY
                || dev.device_id == VIRTIO_BLK_DEVICE_MODERN);

        if !is_virtio_blk {
            return Err(DriverError::NotSupported);
        }

        crate::kinfo!(
            "(VirtIO-Blk) Probe OK:",
            dev.vendor_id as u64,
            ":",
            dev.device_id as u64
        );

        // Cria e inicializa o dispositivo de disco
        if let Some(pci_addr) = get_pci_address_from_dev(dev) {
            // Habilitar Bus Master para permitir DMA
            pci::enable_bus_master(pci_addr);

            crate::kinfo!("(VirtIO-Blk) Criando disco...");
            if let Some(disk) = VirtioDisk::new(0, pci_addr) {
                crate::kinfo!("(VirtIO-Blk) Disco criado:", disk.name());
                crate::drivers::storage::register_device(Arc::new(disk));
            } else {
                crate::kerror!("(VirtIO-Blk) Falha ao criar disco");
            }
        } else {
            crate::kerror!("(VirtIO-Blk) Não foi possível obter endereço PCI");
        }

        Ok(())
    }

    fn remove(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::drivers::storage::unregister_device("vda");
        Ok(())
    }
}

/// Extrai endereço PCI do Device do RDS.
fn get_pci_address_from_dev(dev: &Device) -> Option<PciAddress> {
    if let crate::drivers::base::bus::BusAddress::Pci {
        segment,
        bus,
        device,
        function,
    } = dev.bus_address
    {
        Some(PciAddress::new(segment, bus, device, function))
    } else {
        None
    }
}

// =============================================================================
// ESTRUTURAS VIRTQUEUE
// =============================================================================

/// Descritor de buffer na VirtQueue.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

/// Header do Available Ring.
// TODO: Será usado quando implementarmos interrupt-driven I/O
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    // ring: [u16; VIRTQUEUE_SIZE] follows
}

/// Header do Used Ring.
// TODO: Será usado quando implementarmos interrupt-driven I/O
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    // ring: [VirtqUsedElem; VIRTQUEUE_SIZE] follows
}

/// Elemento do Used Ring.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

/// Header de requisição VirtIO-blk.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioBlkReq {
    req_type: u32,
    reserved: u32,
    sector: u64,
}

// =============================================================================
// DISPOSITIVO
// =============================================================================

/// Estado interno do disco VirtIO.
struct VirtioDiskState {
    /// Se está inicializado.
    enabled: bool,
    /// Capacidade em setores.
    capacity: u64,
    /// Estatísticas.
    stats: StorageStats,
    /// Tamanho real da queue negociado.
    queue_size: u16,
    /// Índice do próximo descritor livre.
    free_head: u16,
    /// Último índice do used ring processado.
    last_used_idx: u16,
}

/// Dispositivo VirtIO Block.
pub struct VirtioDisk {
    /// Índice do disco (0 = vda, 1 = vdb, etc).
    index: u8,
    /// Porta base de I/O (BAR0).
    io_base: u16,
    /// Estado protegido por lock.
    state: Spinlock<VirtioDiskState>,
    /// Buffer DMA para VirtQueue.
    vq_buffer: Spinlock<Option<DmaBuffer>>,
    /// Buffer DMA para requests.
    req_buffer: Spinlock<Option<DmaBuffer>>,
}

impl VirtioDisk {
    /// Cria e inicializa um novo disco VirtIO via PCI.
    pub fn new(index: u8, pci_addr: PciAddress) -> Option<Self> {
        crate::kinfo!("(VirtIO-Blk) Inicializando disco", index as u64);

        // Habilita I/O space e bus mastering
        pci::enable_bus_master(pci_addr);

        // Lê BAR0 para obter porta I/O base
        let bar0 = pci::read_config(pci_addr, 0x10);
        if bar0 & 1 == 0 {
            crate::kerror!("(VirtIO-Blk) BAR0 não é I/O port!");
            return None;
        }
        let io_base = (bar0 & 0xFFFC) as u16;
        crate::kinfo!("(VirtIO-Blk) BAR0 I/O base:", io_base as u64);

        // Reset do dispositivo
        unsafe { port_write_u8(io_base + REG_DEVICE_STATUS, 0) };

        // Acknowledge + Driver
        let status = VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER;
        unsafe { port_write_u8(io_base + REG_DEVICE_STATUS, status) };

        // Negotiate features
        let offered = unsafe { port_read_u32(io_base + REG_DEVICE_FEATURES) };
        // Por enquanto, aceitamos o que for oferecido que não quebre o modo legacy
        let acknowledged = offered & !(VIRTIO_F_VERSION_1 as u32);
        crate::kinfo!(
            "(VirtIO-Blk) Negociando features. Oferecidas:",
            offered as u64,
            "Aceitas:",
            acknowledged as u64
        );
        unsafe { port_write_u32(io_base + REG_DRIVER_FEATURES, acknowledged) };

        // Selecionar queue 0
        unsafe { port_write_u16(io_base + REG_QUEUE_SELECT, 0) };

        // Ler e escrever de volta o tamanho da queue para confirmar
        let queue_size = unsafe { port_read_u16(io_base + REG_QUEUE_SIZE) };
        if queue_size == 0 {
            crate::kerror!("(VirtIO-Blk) Queue size is 0!");
            return None;
        }
        let queue_size = queue_size.min(VIRTQUEUE_MAX_SIZE);
        // Em Legacy, não escrevemos no queue_size, apenas lemos.
        // Mas alguns emuladores aceitam como confirmação.
        unsafe { port_write_u16(io_base + REG_QUEUE_SIZE, queue_size) };
        crate::kinfo!("(VirtIO-Blk) Queue Size Final:", queue_size as u64);

        // Calcular tamanho da VirtQueue conforme Spec Legacy
        // Desc table: 16 bytes * queue_size
        // Avail ring: 2 (flags) + 2 (idx) + 2 * queue_size + 2 (used_event)
        // Used ring: 2 (flags) + 2 (idx) + 8 * queue_size + 2 (avail_event) (Alinhado a 4096)
        let desc_size = 16 * queue_size as usize;
        let avail_size = 4 + 2 * queue_size as usize + 2;
        let used_offset = align_up(desc_size + avail_size, 4096);
        let used_size = 4 + 8 * queue_size as usize + 2;
        let total_size = used_offset + align_up(used_size, 4096);

        // Alocar buffer DMA para VirtQueue
        let vq_dma = dma::alloc(
            total_size,
            crate::drivers::base::device::DeviceId(0), // TODO: usar ID real
            DmaDirection::Bidirectional,
        );

        if vq_dma.is_none() {
            crate::kerror!("(VirtIO-Blk) Falha ao alocar DMA para VirtQueue");
            return None;
        }
        let vq_dma = vq_dma.unwrap();
        crate::kinfo!("(VirtIO-Blk) VirtQueue DMA phys:", vq_dma.phys);

        // IMPORTANTE: Zerar toda a VirtQueue antes de configurar
        unsafe {
            core::ptr::write_bytes(vq_dma.virt as *mut u8, 0, total_size);
        }
        fence(Ordering::SeqCst);

        // Configurar endereço da queue (PFN = Physical Frame Number)
        let pfn = (vq_dma.phys / 4096) as u32;
        unsafe { port_write_u32(io_base + REG_QUEUE_ADDRESS, pfn) };

        // Inicializar lista de descritores livres
        let desc_ptr = vq_dma.virt as *mut VirtqDesc;
        for i in 0..queue_size {
            unsafe {
                (*desc_ptr.add(i as usize)).next = (i + 1) % queue_size;
            }
        }

        // Ler capacidade do device config
        let cap_lo = unsafe { port_read_u32(io_base + REG_CONFIG) } as u64;
        let cap_hi = unsafe { port_read_u32(io_base + REG_CONFIG + 4) } as u64;
        let capacity = cap_lo | (cap_hi << 32);
        crate::kinfo!("(VirtIO-Blk) Capacidade (setores):", capacity);

        let capacity_mb = (capacity * SECTOR_SIZE as u64) / (1024 * 1024);
        crate::kinfo!("(VirtIO-Blk) Capacidade (MB):", capacity_mb);

        // Alocar buffer DMA para requests
        // Precisa de: header (16 bytes) + data (512 bytes para um setor) + status (1 byte)
        let req_dma = dma::alloc(
            4096, // Uma página para requests
            crate::drivers::base::device::DeviceId(0),
            DmaDirection::Bidirectional,
        );

        if req_dma.is_none() {
            crate::kerror!("(VirtIO-Blk) Falha ao alocar DMA para requests");
            dma::free(&vq_dma);
            return None;
        }

        // Set DRIVER_OK - dispositivo está pronto
        let status = VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_DRIVER_OK;
        unsafe { port_write_u8(io_base + REG_DEVICE_STATUS, status) };

        crate::kinfo!("(VirtIO-Blk) Dispositivo inicializado!");

        Some(Self {
            index,
            io_base,
            state: Spinlock::new(VirtioDiskState {
                enabled: true,
                capacity,
                stats: StorageStats::default(),
                queue_size,
                free_head: 0,
                last_used_idx: 0,
            }),
            vq_buffer: Spinlock::new(Some(vq_dma)),
            req_buffer: Spinlock::new(req_dma),
        })
    }

    /// Submete uma requisição de leitura/escrita.
    fn do_request(&self, req_type: u32, sector: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        let vq_guard = self.vq_buffer.lock();
        let mut req_guard = self.req_buffer.lock();
        let mut state = self.state.lock();

        if !state.enabled {
            return Err(BlockError::NotReady);
        }

        let vq_dma = vq_guard.as_ref().ok_or(BlockError::NotReady)?;
        let req_dma = req_guard.as_mut().ok_or(BlockError::NotReady)?;

        // Calcular quantos setores vamos ler/escrever
        let num_sectors = (buf.len() / SECTOR_SIZE).max(1);
        let data_size = num_sectors * SECTOR_SIZE;

        // Limitar ao espaço disponível no buffer DMA (4096 - 16 header - 1 status = 4079)
        // Usamos no máximo 7 setores (3584 bytes) para garantir espaço
        let max_sectors = 7;
        let num_sectors = num_sectors.min(max_sectors);
        let data_size = num_sectors * SECTOR_SIZE;

        // Preparar header da requisição no buffer DMA
        // Layout: [header: 16 bytes][data: N*512 bytes][status: 1 byte]
        let header_offset = 0u64;
        let data_offset = 16u64;
        let status_offset = 16u64 + data_size as u64;

        let req_ptr = (req_dma.virt + header_offset) as *mut VirtioBlkReq;
        unsafe {
            (*req_ptr).req_type = req_type;
            (*req_ptr).reserved = 0;
            (*req_ptr).sector = sector;
        }

        // Status byte
        let status_ptr = (req_dma.virt + status_offset) as *mut u8;
        unsafe { *status_ptr = 0xFF };

        // Data buffer
        let data_ptr = (req_dma.virt + data_offset) as *mut u8;
        if req_type == VIRTIO_BLK_T_OUT {
            unsafe {
                core::ptr::copy_nonoverlapping(buf.as_ptr(), data_ptr, data_size.min(buf.len()));
            }
        }

        // Montar cadeia de descritores
        let desc_ptr = vq_dma.virt as *mut VirtqDesc;
        let desc_base = state.free_head;

        // Descritor 0: Header (16 bytes, device-readable)
        unsafe {
            let d = desc_ptr.add(desc_base as usize);
            (*d).addr = req_dma.phys + header_offset;
            (*d).len = 16;
            (*d).flags = VIRTQ_DESC_F_NEXT;
            (*d).next = (desc_base + 1) % state.queue_size;
        }

        // Descritor 1: Data (device-writable para read, device-readable para write)
        let data_flags = if req_type == VIRTIO_BLK_T_IN {
            VIRTQ_DESC_F_NEXT | VIRTQ_DESC_F_WRITE
        } else {
            VIRTQ_DESC_F_NEXT
        };
        unsafe {
            let d = desc_ptr.add(((desc_base + 1) % state.queue_size) as usize);
            (*d).addr = req_dma.phys + data_offset;
            (*d).len = data_size as u32; // Tamanho real dos dados
            (*d).flags = data_flags;
            (*d).next = (desc_base + 2) % state.queue_size;
        }

        // Descritor 2: Status (1 byte, device-writable)
        unsafe {
            let d = desc_ptr.add(((desc_base + 2) % state.queue_size) as usize);
            (*d).addr = req_dma.phys + status_offset;
            (*d).len = 1;
            (*d).flags = VIRTQ_DESC_F_WRITE;
            (*d).next = 0;
        }

        state.free_head = (state.free_head + 3) % state.queue_size;

        // Adicionar ao available ring
        let avail_off = 16 * state.queue_size as usize;
        let avail_ptr = (vq_dma.virt + avail_off as u64) as *mut u16;

        // Avail Ring Layout: [flags: u16][idx: u16][ring: u16 * queue_size]
        let avail_idx = unsafe { ptr::read_volatile(avail_ptr.add(1)) };
        let ring_idx = (avail_idx % state.queue_size) as usize;

        unsafe {
            ptr::write_volatile(avail_ptr.add(2 + ring_idx), desc_base);
            fence(Ordering::SeqCst);
            ptr::write_volatile(avail_ptr.add(1), avail_idx.wrapping_add(1));
        }

        // Notificar dispositivo
        fence(Ordering::SeqCst);
        unsafe { port_write_u16(self.io_base + REG_QUEUE_NOTIFY, 0) };

        // Esperar pela conclusão (polling)
        let used_off = align_up(avail_off + 4 + 2 * state.queue_size as usize + 2, 4096);
        let used_ptr = (vq_dma.virt + used_off as u64) as *mut u16;

        // Aumentado para ~1 segundo em emulação lenta
        let mut timeout = 50_000_000u32;
        loop {
            // Limpa/Lê status de interrupção (ajuda em alguns emuladores)
            let _isr = unsafe { port_read_u8(self.io_base + REG_ISR_STATUS) };

            let used_idx = unsafe { ptr::read_volatile(used_ptr.add(1)) };
            if used_idx != state.last_used_idx {
                state.last_used_idx = used_idx;
                break;
            }
            timeout = timeout.saturating_sub(1);
            if timeout == 0 {
                crate::kerror!(
                    "(VirtIO-Blk) Timeout esperando I/O! Current used idx= ",
                    used_idx as u64
                );
                crate::kerror!(
                    "(VirtIO-Blk) Last processed idx= ",
                    state.last_used_idx as u64
                );
                return Err(BlockError::Timeout);
            }
            core::hint::spin_loop();
        }

        // Barreira antes de ler o status/dados escritos pelo device
        fence(Ordering::SeqCst);

        // Verificar status (leitura volátil)
        let status = unsafe { ptr::read_volatile(status_ptr) };
        if status != VIRTIO_BLK_S_OK {
            crate::kerror!("(VirtIO-Blk) I/O erro, status=", status as u64);
            return Err(BlockError::IoError);
        }

        // Para leitura: copiar dados do DMA buffer para o buf do caller
        if req_type == VIRTIO_BLK_T_IN {
            let copy_size = data_size.min(buf.len());
            unsafe {
                core::ptr::copy_nonoverlapping(data_ptr, buf.as_mut_ptr(), copy_size);
            }
        }

        // Atualizar estatísticas
        match req_type {
            VIRTIO_BLK_T_IN => {
                state.stats.blocks_read += 1;
                state.stats.bytes_read += buf.len() as u64;
            }
            VIRTIO_BLK_T_OUT => {
                state.stats.blocks_written += 1;
                state.stats.bytes_written += buf.len() as u64;
            }
            _ => {}
        }

        Ok(())
    }
}

impl BlockDevice for VirtioDisk {
    fn name(&self) -> &str {
        match self.index {
            0 => "vda",
            1 => "vdb",
            2 => "vdc",
            _ => "vdX",
        }
    }

    fn info(&self) -> StorageInfo {
        StorageInfo {
            model: alloc::string::String::from("VirtIO Block Device"),
            device_type: Some(StorageType::Virtual),
            interface: Some(StorageInterface::Virtio),
            ..Default::default()
        }
    }

    fn block_size(&self) -> usize {
        SECTOR_SIZE
    }

    fn total_blocks(&self) -> u64 {
        self.state.lock().capacity
    }

    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        if buf.len() < SECTOR_SIZE {
            return Err(BlockError::InvalidBuffer);
        }
        self.do_request(VIRTIO_BLK_T_IN, lba, buf)
    }

    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError> {
        if buf.len() < SECTOR_SIZE {
            return Err(BlockError::InvalidBuffer);
        }
        let buf_mut =
            unsafe { core::slice::from_raw_parts_mut(buf.as_ptr() as *mut u8, buf.len()) };
        self.do_request(VIRTIO_BLK_T_OUT, lba, buf_mut)
    }

    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            writable: true,
            flush: true,
            discard: false,
            max_transfer_blocks: 7, // Até 7 setores por requisição
            ..Default::default()
        }
    }

    /// Leitura otimizada multi-setor
    fn read_blocks(&self, lba: u64, count: usize, buf: &mut [u8]) -> Result<(), BlockError> {
        let bs = self.block_size();
        if buf.len() < count * bs {
            return Err(BlockError::InvalidBuffer);
        }

        let mut offset = 0;
        let mut sector = lba;
        let mut remaining = count;

        while remaining > 0 {
            // Ler até 7 setores por vez
            let batch = remaining.min(7);
            let batch_size = batch * bs;

            self.do_request(
                VIRTIO_BLK_T_IN,
                sector,
                &mut buf[offset..offset + batch_size],
            )?;

            offset += batch_size;
            sector += batch as u64;
            remaining -= batch;
        }

        Ok(())
    }

    fn get_stats(&self) -> StorageStats {
        self.state.lock().stats
    }
}

// =============================================================================
// FUNÇÕES DE I/O
// =============================================================================

/// Lê um byte de uma porta I/O.
// TODO: Será usado para ISR status
#[allow(dead_code)]
#[inline]
unsafe fn port_read_u8(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nostack, preserves_flags));
    value
}

/// Escreve um byte em uma porta I/O.
#[inline]
unsafe fn port_write_u8(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
}

/// Lê uma word de uma porta I/O.
#[inline]
unsafe fn port_read_u16(port: u16) -> u16 {
    let value: u16;
    core::arch::asm!("in ax, dx", out("ax") value, in("dx") port, options(nostack, preserves_flags));
    value
}

/// Escreve uma word em uma porta I/O.
#[inline]
unsafe fn port_write_u16(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nostack, preserves_flags));
}

/// Lê um dword de uma porta I/O.
#[inline]
unsafe fn port_read_u32(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!("in eax, dx", out("eax") value, in("dx") port, options(nostack, preserves_flags));
    value
}

/// Escreve um dword em uma porta I/O.
#[inline]
unsafe fn port_write_u32(port: u16, value: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nostack, preserves_flags));
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Alinha para cima.
#[inline]
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa o driver VirtIO Block.
///
/// Registra o driver no RDS. O matching com dispositivos é automático.
pub fn init() {
    crate::kinfo!("(VirtIO-Blk) Registrando driver...");
    crate::drivers::base::register_driver(Arc::new(VirtioBlkDriver));
}
