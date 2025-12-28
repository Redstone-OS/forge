//! Definição de Mensagens IPC.
//!
//! Mensagens são a única forma de comunicação entre processos.
//! Elas são agnósticas de conteúdo (byte array) mas podem carregar Handles.

use crate::security::capability::CapHandle;
use alloc::vec::Vec;

/// Tamanho máximo do payload de dados em bytes.
/// Mantido pequeno para encorajar eficiência (copy overhead) ou uso de Shared Memory para grandes dados.
pub const MAX_MESSAGE_SIZE: usize = 4096;

/// Cabeçalho da Mensagem.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MessageHeader {
    /// ID da mensagem (protocolo específico).
    pub id: u64,
    /// Tamanho do payload de dados.
    pub data_len: u16,
    /// Número de capabilities anexadas.
    pub cap_count: u8,
    /// Prioridade ou flags.
    pub flags: u8,
}

/// A Mensagem IPC completa.
#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    /// Dados brutos.
    pub data: Vec<u8>,
    /// Capabilities sendo transferidas (delegation).
    /// O Kernel move a ownership dessas caps do remetente para o destinatário.
    pub caps: Vec<CapHandle>,
}

impl Message {
    pub fn new(id: u64, data: Vec<u8>) -> Self {
        // Truncar se exceder limite (ou retornar erro no futuro)
        let len = core::cmp::min(data.len(), MAX_MESSAGE_SIZE);

        Self {
            header: MessageHeader {
                id,
                data_len: len as u16,
                cap_count: 0,
                flags: 0,
            },
            data,
            caps: Vec::new(),
        }
    }

    /// Adiciona uma capability para ser transferida.
    pub fn push_cap(&mut self, cap: CapHandle) {
        if self.caps.len() < 255 {
            self.caps.push(cap);
            self.header.cap_count += 1;
        }
    }
}
