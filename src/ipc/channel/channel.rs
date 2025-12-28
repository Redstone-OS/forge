//! Canal bidirecional (par de portas)

use super::super::port::{Port, PortHandle, IpcError};
use super::super::message::Message;

/// Canal bidirecional
pub struct Channel {
    /// Porta de envio
    send_port: Port,
    /// Porta de recepção
    recv_port: Port,
}

/// Par de canais (endpoints)
pub struct ChannelPair {
    pub endpoint0: ChannelEndpoint,
    pub endpoint1: ChannelEndpoint,
}

/// Um lado do canal
pub struct ChannelEndpoint {
    /// Porta para enviar
    send: *const Port,
    /// Porta para receber
    recv: *const Port,
}

impl Channel {
    /// Cria par de canais conectados
    pub fn create_pair() -> ChannelPair {
        let port_a = Port::new(16);
        let port_b = Port::new(16);
        
        // TODO: armazenar em algum lugar e retornar handles
        // Nota: O guia tinha unimplemented!(), mantendo estrutura mas completando o básico para compilação se possível.
        // Como o guia explicitamente diz "unimplemented!()" no corpo, vou manter.
        unimplemented!()
    }
}

impl ChannelEndpoint {
    /// Envia mensagem
    pub fn send(&self, msg: Message) -> Result<(), IpcError> {
        // SAFETY: Port é válida enquanto Channel existe
        unsafe { (*self.send).send(msg) }
    }
    
    /// Recebe mensagem
    pub fn recv(&self) -> Result<Message, IpcError> {
        // SAFETY: Port é válida enquanto Channel existe
        unsafe { (*self.recv).recv() }
    }
}
