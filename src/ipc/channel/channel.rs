//! Canal bidirecional (par de portas)

use super::super::message::Message;
use super::super::port::{IpcError, Port, PortHandle};

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
        // Cast para mutável pois Channel detém as Portas de forma exclusiva (interior mutability via pointers)
        let status = unsafe { (&mut *(self.send as *mut Port)).send(msg) };
        match status {
            crate::ipc::port::PortStatus::Ok => Ok(()),
            _ => Err(IpcError::ChannelClosed), // Simplificação
        }
    }

    /// Recebe mensagem
    pub fn recv(&self) -> Result<Message, IpcError> {
        // SAFETY: Port é válida enquanto Channel existe
        let res = unsafe { (&mut *(self.recv as *mut Port)).recv() };
        res.map_err(|_| IpcError::ChannelClosed)
    }
}
