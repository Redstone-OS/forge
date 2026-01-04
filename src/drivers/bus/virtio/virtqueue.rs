//! # Virtqueue Implementation
//!
//! Implementação universal dos anéis de transferência do VirtIO (Split Virtqueues).
//! Segue a especificação VirtIO 1.1+.

use super::transport::VirtioTransport;
use crate::mm::translate_addr;
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use core::sync::atomic::{self, Ordering};

#[repr(C, align(16))]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

pub const VIRTQ_DESC_F_NEXT: u16 = 1;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;

#[repr(C, align(2))]
pub struct VirtqAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; 0], // Tamanho dinâmico
}

#[repr(C)]
pub struct VirtqUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C, align(4))]
pub struct VirtqUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtqUsedElem; 0], // Tamanho dinâmico
}

pub struct Virtqueue {
    index: u16,
    size: u16,
    last_used_idx: u16,
    free_head: u16,
    num_free: u16,

    // Memória física dos anéis
    desc: &'static mut [VirtqDesc],
    avail: &'static mut VirtqAvail,
    used: &'static mut VirtqUsed,

    // Keep-alive da memória
    _mem_guard: Vec<Box<[u8]>>,
}

impl Virtqueue {
    pub fn new(index: u16, size: u16) -> Option<Self> {
        if !size.is_power_of_two() {
            return None;
        }

        let mut mem_guard = Vec::new();

        // 1. Alocar Deescritores (16 bytes cada)
        let desc_size = size as usize * 16;
        let desc_mem = Box::leak(vec![0u8; desc_size].into_boxed_slice());
        let desc = unsafe {
            core::slice::from_raw_parts_mut(desc_mem.as_mut_ptr() as *mut VirtqDesc, size as usize)
        };

        // 2. Alocar Avail Ring (6 + 2*size bytes)
        let avail_size = 6 + (2 * size as usize);
        let avail_mem = Box::leak(vec![0u8; avail_size].into_boxed_slice());
        let avail = unsafe { &mut *(avail_mem.as_mut_ptr() as *mut VirtqAvail) };

        // 3. Alocar Used Ring (6 + 8*size bytes)
        let used_size = 6 + (8 * size as usize);
        let used_mem = Box::leak(vec![0u8; used_size].into_boxed_slice());
        let used = unsafe { &mut *(used_mem.as_mut_ptr() as *mut VirtqUsed) };

        // Linkar descritores na lista livre
        for i in 0..(size - 1) {
            desc[i as usize].next = i + 1;
            desc[i as usize].flags = VIRTQ_DESC_F_NEXT;
        }

        Some(Self {
            index,
            size,
            last_used_idx: 0,
            free_head: 0,
            num_free: size,
            desc,
            avail,
            used,
            _mem_guard: mem_guard,
        })
    }

    pub fn phys_desc(&self) -> u64 {
        translate_addr(self.desc.as_ptr() as u64).unwrap()
    }
    pub fn phys_avail(&self) -> u64 {
        translate_addr(self.avail as *const _ as u64).unwrap()
    }
    pub fn phys_used(&self) -> u64 {
        translate_addr(self.used as *const _ as u64).unwrap()
    }

    /// Adiciona um buffer para a fila.
    /// write: true se o dispositivo escreve no buffer, false se lê.
    pub fn add_buffer(&mut self, addr: u64, len: u32, write: bool) -> Result<u16, ()> {
        if self.num_free < 1 {
            return Err(());
        }

        let head = self.free_head;
        let d = &mut self.desc[head as usize];
        d.addr = addr;
        d.len = len;
        d.flags = if write { VIRTQ_DESC_F_WRITE } else { 0 };

        self.free_head = d.next;
        self.num_free -= 1;

        // Atualizar Avail Ring
        let avail_idx = self.avail.idx as usize % self.size as usize;
        unsafe {
            let ring_ptr = self.avail.ring.as_ptr() as *mut u16;
            ring_ptr.add(avail_idx).write_volatile(head);
        }

        atomic::fence(Ordering::Release);
        self.avail.idx = self.avail.idx.wrapping_add(1);

        Ok(head)
    }
}
