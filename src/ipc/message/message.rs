//! Mensagem IPC

use alloc::vec::Vec;
use crate::core::object::Handle;

/// Header da mensagem
#[repr(C)]
pub struct MessageHeader {
    /// Tamanho total do payload
    pub size: u32,
    /// Número de handles anexados
    pub handle_count: u32,
    /// Tipo de mensagem (definido pelo usuário)
    pub msg_type: u32,
    /// Reservado
    pub reserved: u32,
}

/// Mensagem completa
pub struct Message {
    /// Header
    pub header: MessageHeader,
    /// Dados
    pub payload: Vec<u8>,
    /// Handles transferidos
    pub handles: Vec<Handle>,
}

impl Message {
    /// Cria mensagem vazia
    pub fn new(msg_type: u32) -> Self {
        Self {
            header: MessageHeader {
                size: 0,
                handle_count: 0,
                msg_type,
                reserved: 0,
            },
            payload: Vec::new(),
            handles: Vec::new(),
        }
    }
    
    /// Cria com dados
    pub fn with_data(msg_type: u32, data: &[u8]) -> Self {
        Self {
            header: MessageHeader {
                size: data.len() as u32,
                handle_count: 0,
                msg_type,
                reserved: 0,
            },
            payload: data.to_vec(),
            handles: Vec::new(),
        }
    }
    
    /// Anexa handle
    pub fn attach_handle(&mut self, handle: Handle) {
        self.handles.push(handle);
        self.header.handle_count += 1;
    }
}
