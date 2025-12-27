//! # Handle Table
//!
//! Tabela de handles per-process com refcounting.

use super::rights::HandleRights;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

/// Handle é índice + generation
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle(u32);

impl Handle {
    pub const INVALID: Self = Self(u32::MAX);

    pub fn new(index: u16, generation: u16) -> Self {
        Self((generation as u32) << 16 | index as u32)
    }

    pub fn index(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }

    pub fn generation(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    pub fn is_valid(&self) -> bool {
        *self != Self::INVALID
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Tipo de objeto apontado pelo handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleType {
    File,
    Directory,
    Port,
    Process,
    Memory,
    Event,
}

/// Entrada na handle table
pub struct HandleEntry {
    pub htype: HandleType,
    pub object: *mut (),
    pub rights: HandleRights,
    pub refcount: AtomicU32,
    pub generation: u16,
    pub in_use: bool,
}

impl HandleEntry {
    pub const fn empty() -> Self {
        Self {
            htype: HandleType::File,
            object: core::ptr::null_mut(),
            rights: HandleRights::empty(),
            refcount: AtomicU32::new(0),
            generation: 0,
            in_use: false,
        }
    }

    pub fn acquire(&self) {
        self.refcount.fetch_add(1, Ordering::Acquire);
    }

    pub fn release(&self) -> bool {
        self.refcount.fetch_sub(1, Ordering::Release) == 1
    }
}

/// Tabela de handles para um processo
#[allow(dead_code)]
pub struct HandleTable {
    entries: Vec<HandleEntry>,
    capacity: usize,
}

impl HandleTable {
    pub const DEFAULT_CAPACITY: usize = 64;

    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            entries.push(HandleEntry::empty());
        }
        Self { entries, capacity }
    }

    /// Aloca um novo handle
    pub fn alloc(
        &mut self,
        htype: HandleType,
        object: *mut (),
        rights: HandleRights,
    ) -> Option<Handle> {
        // Encontrar slot livre
        for (idx, entry) in self.entries.iter_mut().enumerate() {
            if !entry.in_use {
                entry.htype = htype;
                entry.object = object;
                entry.rights = rights;
                entry.refcount = AtomicU32::new(1);
                entry.generation = entry.generation.wrapping_add(1);
                entry.in_use = true;
                return Some(Handle::new(idx as u16, entry.generation));
            }
        }
        None
    }

    /// Obtém entrada por handle (validando generation)
    pub fn get(&self, handle: Handle) -> Option<&HandleEntry> {
        let idx = handle.index() as usize;
        if idx >= self.entries.len() {
            return None;
        }
        let entry = &self.entries[idx];
        if !entry.in_use || entry.generation != handle.generation() {
            return None;
        }
        Some(entry)
    }

    /// Obtém entrada mutável
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut HandleEntry> {
        let idx = handle.index() as usize;
        if idx >= self.entries.len() {
            return None;
        }
        let entry = &mut self.entries[idx];
        if !entry.in_use || entry.generation != handle.generation() {
            return None;
        }
        Some(entry)
    }

    /// Fecha handle
    pub fn close(&mut self, handle: Handle) -> bool {
        if let Some(entry) = self.get_mut(handle) {
            if entry.release() {
                entry.in_use = false;
                entry.object = core::ptr::null_mut();
                return true;
            }
        }
        false
    }

    /// Duplica handle com rights reduzidos
    pub fn dup(&mut self, handle: Handle, new_rights: HandleRights) -> Option<Handle> {
        let (htype, object, current_rights) = {
            let entry = self.get(handle)?;
            if !entry.rights.contains(HandleRights::DUP) {
                return None;
            }
            if !entry.rights.can_reduce_to(new_rights) {
                return None;
            }
            (entry.htype, entry.object, new_rights)
        };

        self.alloc(htype, object, current_rights)
    }
}

impl Default for HandleTable {
    fn default() -> Self {
        Self::new()
    }
}
