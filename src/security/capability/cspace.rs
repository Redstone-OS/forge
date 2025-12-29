#![allow(dead_code)]
//! Capability Space - tabela por processo

use super::{CapHandle, CapRights, Capability};
// Note: Guide imported Sync Spinlock but didn't use it in struct definition?
// Maybe intended for internal locking or external usage. Struct CSpace provided in guide doesn't have Spinlock field.
// Following guide literal code.

/// Tamanho máximo do CSpace
const CSPACE_SIZE: usize = 256;

/// CSpace - tabela de capabilities por processo
pub struct CSpace {
    /// Slots de capabilities
    slots: [Option<Capability>; CSPACE_SIZE],
    /// Próximo slot livre
    next_free: usize,
    /// Generation counter global
    generation: u32,
}

impl CSpace {
    /// Cria CSpace vazio
    pub const fn new() -> Self {
        const NONE: Option<Capability> = None;
        Self {
            slots: [NONE; CSPACE_SIZE],
            next_free: 1, // Slot 0 é reservado (INVALID)
            generation: 1,
        }
    }

    /// Insere capability e retorna handle
    pub fn insert(&mut self, cap: Capability) -> Option<CapHandle> {
        // Procurar slot livre
        for i in self.next_free..CSPACE_SIZE {
            if self.slots[i].is_none() {
                self.slots[i] = Some(cap);
                self.next_free = i + 1;
                return Some(CapHandle::new(i as u32));
            }
        }

        // Tentar desde o início
        for i in 1..self.next_free {
            if self.slots[i].is_none() {
                self.slots[i] = Some(cap);
                return Some(CapHandle::new(i as u32));
            }
        }

        None // CSpace cheio
    }

    /// Busca capability por handle
    pub fn lookup(&self, handle: CapHandle) -> Option<&Capability> {
        let index = handle.as_u32() as usize;
        if index >= CSPACE_SIZE {
            return None;
        }
        self.slots[index].as_ref()
    }

    /// Busca capability mutável
    pub fn lookup_mut(&mut self, handle: CapHandle) -> Option<&mut Capability> {
        let index = handle.as_u32() as usize;
        if index >= CSPACE_SIZE {
            return None;
        }
        self.slots[index].as_mut()
    }

    /// Remove capability
    pub fn remove(&mut self, handle: CapHandle) -> Option<Capability> {
        let index = handle.as_u32() as usize;
        if index >= CSPACE_SIZE {
            return None;
        }

        let cap = self.slots[index].take();
        if cap.is_some() && index < self.next_free {
            self.next_free = index;
        }
        cap
    }

    /// Duplica capability
    pub fn duplicate(&mut self, handle: CapHandle) -> Option<CapHandle> {
        let cap = self.lookup(handle)?.clone();

        if !cap.rights.has(CapRights::DUPLICATE) {
            return None;
        }

        self.insert(cap)
    }

    /// Verifica se handle tem direito específico
    pub fn check_rights(&self, handle: CapHandle, required: CapRights) -> bool {
        match self.lookup(handle) {
            Some(cap) => cap.rights.has(required),
            None => false,
        }
    }
}

/// Erro de capability
#[derive(Debug, Clone, Copy)]
pub enum CapError {
    InvalidHandle,
    InsufficientRights,
    TypeMismatch,
    CSpaceFull,
    NotTransferable,
}
