//! # Standard System Types
//!
//! Define os tipos primitivos usados nas interfaces do kernel (Syscalls).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **ABI Stability:** Usar `Pid` (alias para `usize`) permite mudar a representação interna sem quebrar a assinatura das funções públicas.
//! - **Clarity:** `Time` é mais semântico que `i64`.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Separation:** Centralizar tipos evita "magic numbers" espalhados.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Identity Crisis:** O Redstone OS é um sistema Capability-Based, mas define `Uid` e `Gid` (Access Control List - ACL).
//!   - *Problema:* A presença desses tipos sugere que o kernel ainda pensa em "usuários UNIX", o que contradiz a filosofia Zero Trust/Capabilities.
//! - **Architecture Dependent:** `usize` varia entre 32/64 bits. Se quisermos compatibilidade 32-bits (compat layer), `Pid` deveria ser `u32` fixo na ABI?
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical/Architecture)** Deprecar **Uid/Gid**.
//!   - *Motivo:* Substituir por `SubjectId` ou remover totalmente em favor de Capabilities anônimas.
//! - [ ] **TODO: (Cleanup)** Mover `STDIN/OUT/ERR` para constants em `unistd.rs` ou similar.
//!
//! --------------------------------------------------------------------------------
//!
//! Define aliases padrão para garantir consistência em todo o OS.

pub type Pid = usize; // Process ID
pub type Tid = usize; // Thread ID
pub type Uid = u32; // User ID
pub type Gid = u32; // Group ID
pub type Mode = u16; // File Mode/Permissions
pub type Dev = u64; // Device ID
pub type Ino = u64; // Inode Number
pub type Off = i64; // File Offset
pub type Time = i64; // Timestamp (Unix)

// File Descriptor
pub const STDIN_FILENO: usize = 0;
pub const STDOUT_FILENO: usize = 1;
pub const STDERR_FILENO: usize = 2;
