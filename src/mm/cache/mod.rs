//! # Page Cache
//!
//! Cache de páginas de arquivos em memória RAM.
//!
//! ## Visão Geral
//!
//! O Page Cache é uma camada entre o VFS e o armazenamento físico.
//! Mantém páginas de arquivos em RAM para evitar I/O desnecessário.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                          VFS                                 │
//! └───────────────────────────┬─────────────────────────────────┘
//!                             │
//! ┌───────────────────────────▼─────────────────────────────────┐
//! │                      PAGE CACHE                              │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
//! │  │ Page 0   │  │ Page 1   │  │ Page N   │  ...              │
//! │  │ (clean)  │  │ (dirty)  │  │ (clean)  │                   │
//! │  └──────────┘  └──────────┘  └──────────┘                   │
//! └───────────────────────────┬─────────────────────────────────┘
//!                             │
//! ┌───────────────────────────▼─────────────────────────────────┐
//! │                     Block Device                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Características
//!
//! - **Read-ahead**: Pré-carrega páginas adjacentes
//! - **Write-back**: Escreve páginas sujas em background
//! - **LRU Eviction**: Remove páginas menos usadas quando sob pressão

pub mod pagecache;

pub use pagecache::{CachedPage, PageCache, PageCacheStats};
