//! # Display Engine
//!
//! Controle do Display Engine do GPU Intel.
//! Gerencia pipes, planes e outputs.

pub mod pipe;
pub mod plane;

pub use pipe::Pipe;
pub use plane::Plane;
