//! IPC - Inter-Process Communication
//!
//! Mecanismos de comunicação entre processos.
//!
//! # Mecanismos Suportados
//! - Pipes: Comunicação unidirecional
//! - Shared Memory: Memória compartilhada
//! - Futex: Fast userspace mutex
//! - Unix Sockets: Sockets locais
//! - Signals: Sinais (já existe)
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar pipes
//! - TODO(prioridade=alta, versão=v1.0): Implementar shared memory
//! - TODO(prioridade=alta, versão=v1.0): Implementar futex
//! - TODO(prioridade=média, versão=v1.0): Implementar unix sockets
//! - TODO(prioridade=baixa, versão=v2.0): Implementar message queues
//! - TODO(prioridade=baixa, versão=v2.0): Implementar semaphores

pub mod pipe;
pub mod shm;
pub mod futex;
pub mod unix_socket;
pub mod signal;
