//! # Ring Structures para xHCI
//!
//! Command Ring, Transfer Ring, e Event Ring.

use super::structs::{ErstEntry, Trb};
use crate::mm::translate_addr;
use crate::mm::PhysAddr;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Tamanho padrão do ring (número de TRBs)
pub const DEFAULT_RING_SIZE: usize = 256;

/// Command/Transfer Ring
pub struct Ring {
    /// TRBs do ring
    trbs: &'static mut [Trb],
    /// Índice do próximo TRB a escrever
    enqueue_index: usize,
    /// Cycle bit atual
    cycle: bool,
    /// Endereço físico do início do ring
    phys_addr: PhysAddr,
}

impl Ring {
    /// Cria um novo ring com o tamanho especificado
    pub fn new(size: usize) -> Option<Self> {
        // Alocar TRBs com folga para alinhamento de 64 bytes
        // size+4 garante que mesmo com offset teremos 'size' TRBs
        let mut vec = Vec::with_capacity(size + 4);
        for _ in 0..(size + 4) {
            vec.push(Trb::new());
        }

        // Vazar para garantir estabilidade do ponteiro
        let leaked = Box::leak(vec.into_boxed_slice());
        let raw_base = leaked.as_ptr() as u64;
        let aligned_ptr = (raw_base + 63) & !63;

        // Criar slice estático a partir do ponto alinhado
        let trbs = unsafe { core::slice::from_raw_parts_mut(aligned_ptr as *mut Trb, size) };

        // Obter endereço físico
        let phys = translate_addr(aligned_ptr).expect("Ring Virt->Phys failed");
        let phys_addr = PhysAddr::new(phys);

        Some(Self {
            trbs,
            enqueue_index: 0,
            cycle: true,
            phys_addr,
        })
    }

    /// Cria um ring de tamanho padrão
    pub fn new_default() -> Option<Self> {
        Self::new(DEFAULT_RING_SIZE)
    }

    /// Retorna o endereço físico do ring
    pub fn phys_addr(&self) -> PhysAddr {
        self.phys_addr
    }

    /// Retorna o cycle bit atual
    pub fn cycle(&self) -> bool {
        self.cycle
    }

    /// Retorna o número de TRBs no ring
    pub fn size(&self) -> usize {
        self.trbs.len()
    }

    /// Enqueue um TRB no ring
    ///
    /// Retorna o índice onde foi colocado
    pub fn enqueue(&mut self, mut trb: Trb) -> usize {
        let index = self.enqueue_index;

        // Set cycle bit
        trb.set_cycle(self.cycle);

        // Escrever TRB
        self.trbs[index] = trb;

        // Avançar índice
        self.enqueue_index += 1;

        // Verificar se precisa de Link TRB (wrap around)
        if self.enqueue_index >= self.trbs.len() - 1 {
            // Último slot é reservado para Link TRB
            let mut link = Trb::link(self.phys_addr.as_u64(), true);
            link.set_cycle(self.cycle);
            self.trbs[self.enqueue_index] = link;

            // Wrap around
            self.enqueue_index = 0;
            self.cycle = !self.cycle;
        }

        index
    }

    /// Obtém referência a um TRB pelo índice
    pub fn get(&self, index: usize) -> Option<&Trb> {
        self.trbs.get(index)
    }

    /// Obtém referência mutável a um TRB pelo índice
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Trb> {
        self.trbs.get_mut(index)
    }

    /// Retorna o endereço físico do próximo TRB a ser escrito
    pub fn enqueue_ptr(&self) -> u64 {
        self.phys_addr.as_u64() + (self.enqueue_index * core::mem::size_of::<Trb>()) as u64
    }
}

/// Event Ring
pub struct EventRing {
    /// TRBs do ring
    trbs: &'static mut [Trb],
    /// Event Ring Segment Table
    erst: &'static mut [ErstEntry],
    /// Índice do próximo TRB a ler
    dequeue_index: usize,
    /// Cycle bit esperado
    cycle: bool,
    /// Endereço físico do ring
    ring_phys: PhysAddr,
    /// Endereço físico da ERST
    erst_phys: PhysAddr,
}

impl EventRing {
    /// Cria um novo event ring
    pub fn new(size: usize) -> Option<Self> {
        // Alocar TRBs (Event Ring deve ter alinhamento de 64 bytes também)
        let mut trbs_vec = Vec::with_capacity(size + 4);
        for _ in 0..(size + 4) {
            trbs_vec.push(Trb::new());
        }
        let leaked_trbs = Box::leak(trbs_vec.into_boxed_slice());
        let trbs_raw = leaked_trbs.as_ptr() as u64;
        let trbs_aligned = (trbs_raw + 63) & !63;
        let trbs = unsafe { core::slice::from_raw_parts_mut(trbs_aligned as *mut Trb, size) };

        let ring_phys_val = translate_addr(trbs_aligned).expect("EventRing Virt->Phys failed");
        let ring_phys = PhysAddr::new(ring_phys_val);

        // Alocar ERST (Deve ser alinhada a 64 bytes)
        let mut erst_vec = Vec::with_capacity(8); // Folga para alinhar 1 entrada
        for _ in 0..8 {
            erst_vec.push(ErstEntry::new());
        }
        let leaked_erst = Box::leak(erst_vec.into_boxed_slice());
        let erst_raw = leaked_erst.as_ptr() as u64;
        let erst_aligned = (erst_raw + 63) & !63;
        let erst = unsafe { core::slice::from_raw_parts_mut(erst_aligned as *mut ErstEntry, 1) };

        erst[0] = ErstEntry {
            base_lo: ring_phys_val as u32,
            base_hi: (ring_phys_val >> 32) as u32,
            size: size as u32,
            reserved: 0,
        };

        let erst_phys_val = translate_addr(erst_aligned).expect("ERST Virt->Phys failed");
        let erst_phys = PhysAddr::new(erst_phys_val);

        Some(Self {
            trbs,
            erst,
            dequeue_index: 0,
            cycle: true,
            ring_phys,
            erst_phys,
        })
    }

    /// Retorna o endereço físico da ERST
    pub fn erst_phys(&self) -> PhysAddr {
        self.erst_phys
    }

    /// Retorna o número de segmentos na ERST
    pub fn erst_size(&self) -> usize {
        self.erst.len()
    }

    /// Retorna o dequeue pointer atual
    pub fn dequeue_ptr(&self) -> u64 {
        self.ring_phys.as_u64() + (self.dequeue_index * core::mem::size_of::<Trb>()) as u64
    }

    /// Tenta ler o próximo evento
    ///
    /// Retorna Some(Trb) se há evento disponível, None caso contrário
    pub fn dequeue(&mut self) -> Option<Trb> {
        let trb = &self.trbs[self.dequeue_index];

        // Verificar se o cycle bit corresponde ao esperado
        if trb.cycle() != self.cycle {
            return None;
        }

        // Copiar TRB
        let event = *trb;

        // Avançar índice
        self.dequeue_index += 1;
        if self.dequeue_index >= self.trbs.len() {
            self.dequeue_index = 0;
            self.cycle = !self.cycle;
        }

        Some(event)
    }

    /// Verifica se há eventos pendentes
    pub fn has_events(&self) -> bool {
        self.trbs[self.dequeue_index].cycle() == self.cycle
    }
}

/// Wrapper thread-safe para Ring
pub struct SyncRing {
    inner: Spinlock<Ring>,
}

impl SyncRing {
    pub fn new(ring: Ring) -> Self {
        Self {
            inner: Spinlock::new(ring),
        }
    }

    pub fn enqueue(&self, trb: Trb) -> usize {
        self.inner.lock().enqueue(trb)
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.inner.lock().phys_addr()
    }

    pub fn cycle(&self) -> bool {
        self.inner.lock().cycle()
    }

    pub fn enqueue_ptr(&self) -> u64 {
        self.inner.lock().enqueue_ptr()
    }
}

/// Wrapper thread-safe para EventRing
pub struct SyncEventRing {
    inner: Spinlock<EventRing>,
}

impl SyncEventRing {
    pub fn new(ring: EventRing) -> Self {
        Self {
            inner: Spinlock::new(ring),
        }
    }

    pub fn dequeue(&self) -> Option<Trb> {
        self.inner.lock().dequeue()
    }

    pub fn has_events(&self) -> bool {
        self.inner.lock().has_events()
    }

    pub fn erst_phys(&self) -> PhysAddr {
        self.inner.lock().erst_phys()
    }

    pub fn erst_size(&self) -> usize {
        self.inner.lock().erst_size()
    }

    pub fn dequeue_ptr(&self) -> u64 {
        self.inner.lock().dequeue_ptr()
    }
}
