//! Handle - Referência de userspace para objetos do kernel
//!
//! TODO: Mover implementação existente de core/handle.rs para cá
//! - Handle = índice + generation (previne use-after-free)
//! - HandleTable por processo
//! - Operações: insert, get, remove, duplicate
