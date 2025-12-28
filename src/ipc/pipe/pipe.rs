//! Pipes unidirecionais
//!
//! Implementado como um wrapper sobre Port para fornecer semântica de stream/uni-direcional.

use super::super::message::Message;
use super::super::port::{IpcError, Port};
use crate::sync::Mutex;
use alloc::sync::Arc;

pub struct Pipe {
    port: Arc<Mutex<Port>>,
}

impl Pipe {
    pub fn new() -> (PipeReader, PipeWriter) {
        let port = Arc::new(Mutex::new(Port::new(32))); // Buffer size default
        (PipeReader { port: port.clone() }, PipeWriter { port })
    }
}

pub struct PipeReader {
    port: Arc<Mutex<Port>>,
}

impl PipeReader {
    pub fn read(&self) -> Result<Message, IpcError> {
        let mut port = self.port.lock();
        // recv returns Result<Message, PortStatus> which acts as IpcError
        match port.recv() {
            Ok(msg) => Ok(msg),
            Err(status) => Err(status),
        }
    }

    pub fn try_read(&self) -> Result<Message, IpcError> {
        let mut port = self.port.lock();
        // recv is non-blocking if we check empty?
        // Port::recv checks queue.pop_front().
        // It returns Empty if empty.
        match port.recv() {
            Ok(msg) => Ok(msg),
            Err(status) => Err(status),
        }
    }
}

pub struct PipeWriter {
    port: Arc<Mutex<Port>>,
}

impl PipeWriter {
    pub fn write(&self, msg: Message) -> Result<(), IpcError> {
        let mut port = self.port.lock();
        match port.send(msg) {
            super::super::port::PortStatus::Ok => Ok(()),
            err => Err(err),
        }
    }
}
