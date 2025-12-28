//! Pipes unidirecionais
//!
//! Implementado como um wrapper sobre Port para fornecer semântica de stream/uni-direcional.

use super::super::port::{Port, IpcError};
use super::super::message::Message;
use alloc::sync::Arc;

pub struct Pipe {
    port: Arc<Port>,
}

impl Pipe {
    pub fn new() -> (PipeReader, PipeWriter) {
        let port = Arc::new(Port::new(32)); // Buffer size default
        (
            PipeReader { port: port.clone() },
            PipeWriter { port },
        )
    }
}

pub struct PipeReader {
    port: Arc<Port>,
}

impl PipeReader {
    pub fn read(&self) -> Result<Message, IpcError> {
        self.port.recv()
    }

    pub fn try_read(&self) -> Result<Message, IpcError> {
        self.port.try_recv()
    }
}

pub struct PipeWriter {
    port: Arc<Port>,
}

impl PipeWriter {
    pub fn write(&self, msg: Message) -> Result<(), IpcError> {
        self.port.send(msg)
    }
}
