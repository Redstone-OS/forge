//! # Virtqueue - Fila de Comunicação VirtIO
//!
//! Implementa as estruturas de dados para comunicação com dispositivos VirtIO.
//!
//! ## Estrutura da Virtqueue
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Descriptor Table                         │
//! │ ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐          │
//! │ │  0  │  1  │  2  │  3  │ ... │     │     │ N-1 │          │
//! │ └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘          │
//! └─────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Available Ring                           │
//! │ ┌──────┬───────┬─────────────────────────────┐             │
//! │ │flags │  idx  │     ring[0..N]              │             │
//! │ └──────┴───────┴─────────────────────────────┘             │
//! └─────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Used Ring                              │
//! │ ┌──────┬───────┬─────────────────────────────┐             │
//! │ │flags │  idx  │  ring[0..N] (id, len)       │             │
//! │ └──────┴───────┴─────────────────────────────┘             │
//! └─────────────────────────────────────────────────────────────┘
//! ```

#![allow(dead_code)]

use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::Spinlock;
use alloc::vec::Vec;
use core::sync::atomic::{fence, Ordering};

/// Tamanho padrão da queue
pub const QUEUE_SIZE: u16 = 128;

/// Flags do descriptor
pub mod desc_flags {
    /// O buffer continua no próximo descriptor
    pub const NEXT: u16 = 1;
    /// Buffer é write-only (para o dispositivo)
    pub const WRITE: u16 = 2;
    /// Buffer contém uma lista de buffers indiretos
    pub const INDIRECT: u16 = 4;
}

/// Descriptor da virtqueue (16 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VirtqDesc {
    /// Endereço físico do buffer
    pub addr: u64,
    /// Tamanho do buffer
    pub len: u32,
    /// Flags do descriptor
    pub flags: u16,
    /// Índice do próximo descriptor (se NEXT flag)
    pub next: u16,
}

/// Entrada do Available Ring
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VirtqAvail {
    /// Flags
    pub flags: u16,
    /// Índice do próximo descriptor a ser adicionado
    pub idx: u16,
    // ring: [u16; QUEUE_SIZE] - segue após esta struct
    // used_event: u16 - após o ring
}

/// Elemento do Used Ring
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VirtqUsedElem {
    /// Índice do descriptor chain usado
    pub id: u32,
    /// Total de bytes escritos no buffer
    pub len: u32,
}

/// Header do Used Ring
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VirtqUsed {
    /// Flags
    pub flags: u16,
    /// Índice do próximo elemento a ser escrito pelo dispositivo
    pub idx: u16,
    // ring: [VirtqUsedElem; QUEUE_SIZE] - segue após
    // avail_event: u16 - após o ring
}

/// Virtqueue completa
pub struct Virtqueue {
    /// Tamanho da queue (potência de 2)
    size: u16,
    /// Descriptors
    desc: *mut VirtqDesc,
    /// Available ring
    avail: *mut VirtqAvail,
    /// Used ring
    used: *mut VirtqUsed,
    /// Último índice seen no used ring
    last_used_idx: u16,
    /// Lista de descritores livres
    free_head: u16,
    /// Número de descritores livres
    num_free: u16,
    /// Endereço físico da queue (para passar ao dispositivo)
    phys_addr: PhysAddr,
    /// Lock para operações
    lock: Spinlock<()>,
}

// SAFETY: Virtqueue usa locking interno
unsafe impl Send for Virtqueue {}
unsafe impl Sync for Virtqueue {}

impl Virtqueue {
    /// Cria uma nova virtqueue
    ///
    /// Aloca memória para descriptors, available ring e used ring.
    pub fn new(size: u16) -> Option<Self> {
        // Calcular tamanhos necessários
        let desc_size = core::mem::size_of::<VirtqDesc>() * size as usize;
        let avail_size = 2 + 2 + (2 * size as usize) + 2; // flags + idx + ring + used_event
        let used_size = 2 + 2 + (8 * size as usize) + 2; // flags + idx + ring + avail_event

        // Alinhamentos: descriptors (16), available (2), used (4)
        let total_size =
            align_up(desc_size, 16) + align_up(avail_size, 2) + align_up(used_size, 4096); // used precisa estar em página separada

        crate::kdebug!("(Virtqueue) Alocando:", total_size as u64);

        // Alocar memória contígua alinhada a página
        let layout = core::alloc::Layout::from_size_align(total_size, 4096).ok()?;
        let base = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if base.is_null() {
            crate::kerror!("(Virtqueue) Falha na alocação!");
            return None;
        }

        let base_addr = base as u64;

        // Calcular ponteiros
        let desc = base as *mut VirtqDesc;
        let avail = (base_addr + align_up(desc_size, 16) as u64) as *mut VirtqAvail;
        let used = (base_addr + align_up(desc_size, 16) as u64 + align_up(avail_size, 4096) as u64)
            as *mut VirtqUsed;

        // Inicializar lista de descritores livres
        unsafe {
            for i in 0..size {
                let d = desc.add(i as usize);
                (*d).next = if i + 1 < size { i + 1 } else { 0 };
            }
        }

        // Obter endereço físico (assumindo identity mapping ou HHDM)
        // TODO: Converter para endereço físico real via page tables
        let phys_addr = PhysAddr::new(base_addr);

        crate::kinfo!("(Virtqueue) Criada com tamanho:", size as u64);

        Some(Self {
            size,
            desc,
            avail,
            used,
            last_used_idx: 0,
            free_head: 0,
            num_free: size,
            phys_addr,
            lock: Spinlock::new(()),
        })
    }

    /// Retorna o endereço físico da queue (para configurar no dispositivo)
    pub fn phys_addr(&self) -> PhysAddr {
        self.phys_addr
    }

    /// Retorna o tamanho da queue
    pub fn size(&self) -> u16 {
        self.size
    }

    /// Aloca um descriptor
    pub fn alloc_desc(&mut self) -> Option<u16> {
        let _guard = self.lock.lock();

        if self.num_free == 0 {
            return None;
        }

        let idx = self.free_head;
        let desc = unsafe { &*self.desc.add(idx as usize) };
        self.free_head = desc.next;
        self.num_free -= 1;

        Some(idx)
    }

    /// Libera um descriptor
    pub fn free_desc(&mut self, idx: u16) {
        let _guard = self.lock.lock();

        let desc = unsafe { &mut *self.desc.add(idx as usize) };
        desc.next = self.free_head;
        self.free_head = idx;
        self.num_free += 1;
    }

    /// Configura um descriptor
    pub fn set_desc(&mut self, idx: u16, addr: PhysAddr, len: u32, flags: u16, next: u16) {
        let desc = unsafe { &mut *self.desc.add(idx as usize) };
        desc.addr = addr.as_u64();
        desc.len = len;
        desc.flags = flags;
        desc.next = next;
    }

    /// Adiciona um descriptor ao available ring
    pub fn push_avail(&mut self, desc_idx: u16) {
        let _guard = self.lock.lock();

        unsafe {
            // Obter ponteiro para o ring array (logo após o header)
            let ring = (self.avail as *mut u16).add(2);
            let avail_idx = (*self.avail).idx;

            // Escrever no ring
            *ring.add((avail_idx % self.size) as usize) = desc_idx;

            // Memory barrier antes de atualizar o índice
            fence(Ordering::SeqCst);

            // Incrementar índice
            (*self.avail).idx = avail_idx.wrapping_add(1);
        }
    }

    /// Verifica se há entradas novas no used ring
    pub fn has_used(&self) -> bool {
        fence(Ordering::SeqCst);
        unsafe { (*self.used).idx != self.last_used_idx }
    }

    /// Pop uma entrada do used ring
    pub fn pop_used(&mut self) -> Option<VirtqUsedElem> {
        let _guard = self.lock.lock();

        fence(Ordering::SeqCst);
        unsafe {
            if (*self.used).idx == self.last_used_idx {
                return None;
            }

            // Obter ponteiro para o ring array (logo após o header)
            let ring = (self.used as *mut VirtqUsedElem).add(1); // Pular flags e idx
            let elem = *ring.add((self.last_used_idx % self.size) as usize);

            self.last_used_idx = self.last_used_idx.wrapping_add(1);

            Some(elem)
        }
    }
}

/// Alinha valor para cima
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
