//! # Object - Sistema de Objetos do Kernel
//!
//! Inspirado no Zircon (Fuchsia). Todos os recursos são objetos.

pub mod dispatcher;
pub mod handle;
pub mod kobject;
pub mod refcount;
pub mod rights;
