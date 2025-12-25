//! Handle Table - Tabela de Handles por Processo
//!
//! Cada processo possui sua própria HandleTable que mapeia índices (u32)
//! para objetos do kernel com direitos específicos.
//!
//! # Modelo de Segurança
//!
//! - Handles são opacos para userspace (apenas índice numérico)
//! - Cada handle tem tipo + endereço do objeto + máscara de direitos
//! - Direitos só podem ser reduzidos (não elevados) ao duplicar
//! - Handles são locais ao processo (não globais)

use crate::security::capability::CapRights;
use alloc::vec::Vec;

/// Índice de handle (o que userspace vê).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle(pub u32);

impl Handle {
    /// Handle inválido (placeholder).
    pub const INVALID: Handle = Handle(u32::MAX);

    /// Verifica se o handle é válido (não é INVALID).
    pub fn is_valid(&self) -> bool {
        *self != Self::INVALID
    }

    /// Converte de usize (vindo de syscall).
    pub fn from_usize(val: usize) -> Self {
        Handle(val as u32)
    }

    /// Converte para usize (para retorno de syscall).
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

/// Tipo de objeto referenciado pelo handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjectType {
    /// Slot vazio
    None = 0,
    /// Porta de IPC
    Port = 1,
    /// Região de memória mapeada
    Memory = 2,
    /// Arquivo/Diretório do VFS
    File = 3,
    /// Dispositivo de hardware
    Device = 4,
    /// Referência a outra tarefa
    Task = 5,
}

/// Entrada na tabela de handles.
#[derive(Debug, Clone)]
pub struct HandleEntry {
    /// Tipo do objeto
    pub object_type: ObjectType,
    /// Ponteiro ou ID do objeto no kernel
    pub object_ptr: usize,
    /// Direitos de acesso
    pub rights: CapRights,
}

impl HandleEntry {
    /// Cria entrada vazia.
    pub const fn empty() -> Self {
        Self {
            object_type: ObjectType::None,
            object_ptr: 0,
            rights: CapRights::empty(),
        }
    }

    /// Cria entrada para um objeto.
    pub fn new(object_type: ObjectType, object_ptr: usize, rights: CapRights) -> Self {
        Self {
            object_type,
            object_ptr,
            rights,
        }
    }

    /// Verifica se a entrada está ativa.
    pub fn is_active(&self) -> bool {
        self.object_type != ObjectType::None
    }
}

/// Tabela de handles de um processo.
///
/// Capacidade inicial é pequena, cresce sob demanda.
pub struct HandleTable {
    /// Entradas (índice = valor do Handle)
    entries: Vec<HandleEntry>,
    /// Próximo slot livre (hint para busca rápida)
    next_free: u32,
}

impl HandleTable {
    /// Capacidade inicial.
    pub const INITIAL_CAPACITY: usize = 16;
    /// Capacidade máxima.
    pub const MAX_CAPACITY: usize = 4096;

    /// Cria tabela vazia sem alocação inicial (lazy).
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            next_free: 0,
        }
    }

    /// Cria nova tabela pré-alocada.
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(Self::INITIAL_CAPACITY);
        for _ in 0..Self::INITIAL_CAPACITY {
            entries.push(HandleEntry::empty());
        }

        Self {
            entries,
            next_free: 0,
        }
    }

    /// Número de handles ativos.
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_active()).count()
    }

    /// Insere um novo handle, retorna o índice.
    pub fn insert(&mut self, entry: HandleEntry) -> Option<Handle> {
        // Buscar slot livre a partir do hint
        let start = self.next_free as usize;
        let len = self.entries.len();

        for i in 0..len {
            let idx = (start + i) % len;
            if !self.entries[idx].is_active() {
                self.entries[idx] = entry;
                self.next_free = ((idx + 1) % len) as u32;
                return Some(Handle(idx as u32));
            }
        }

        // Tabela cheia, tentar expandir
        if len < Self::MAX_CAPACITY {
            let new_len = (len * 2).min(Self::MAX_CAPACITY);
            let idx = len;

            // Expandir
            for _ in len..new_len {
                self.entries.push(HandleEntry::empty());
            }

            self.entries[idx] = entry;
            self.next_free = (idx + 1) as u32;
            return Some(Handle(idx as u32));
        }

        None // Tabela cheia
    }

    /// Obtém referência para entrada por handle.
    pub fn get(&self, handle: Handle) -> Option<&HandleEntry> {
        let entry = self.entries.get(handle.0 as usize)?;
        if entry.is_active() {
            Some(entry)
        } else {
            None
        }
    }

    /// Obtém referência mutável para entrada.
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut HandleEntry> {
        let entry = self.entries.get_mut(handle.0 as usize)?;
        if entry.is_active() {
            Some(entry)
        } else {
            None
        }
    }

    /// Remove um handle e retorna a entrada.
    pub fn remove(&mut self, handle: Handle) -> Option<HandleEntry> {
        let idx = handle.0 as usize;
        if idx >= self.entries.len() {
            return None;
        }

        let entry = &mut self.entries[idx];
        if !entry.is_active() {
            return None;
        }

        // Trocar por entrada vazia
        let old = core::mem::replace(entry, HandleEntry::empty());

        // Atualizar hint de slot livre
        if (self.next_free as usize) > idx {
            self.next_free = idx as u32;
        }

        Some(old)
    }

    /// Verifica se handle possui os direitos especificados.
    pub fn check_rights(&self, handle: Handle, required: CapRights) -> bool {
        self.get(handle)
            .map(|e| e.rights.contains(required))
            .unwrap_or(false)
    }

    /// Duplica handle com direitos potencialmente reduzidos.
    pub fn duplicate(&mut self, handle: Handle, new_rights: CapRights) -> Option<Handle> {
        // Obter entrada original
        let original = self.get(handle)?;

        // Novos direitos devem ser subconjunto dos atuais
        if !original.rights.contains(new_rights) {
            return None;
        }

        // Criar cópia com novos direitos
        let new_entry = HandleEntry {
            object_type: original.object_type,
            object_ptr: original.object_ptr,
            rights: new_rights,
        };

        self.insert(new_entry)
    }
}

impl Default for HandleTable {
    fn default() -> Self {
        Self::new()
    }
}
