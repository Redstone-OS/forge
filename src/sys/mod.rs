//! # System ABI & Definitions
//!
//! Este módulo define a "Língua Franca" falada entre o Kernel e o Mundo Exterior (Userspace).
//! Ele contém as definições binárias (ABI) que garantem que aplicações compiladas hoje
//! continuem rodando amanhã, independente de mudanças internas no Kernel.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **ABI Estável:** Define tipos, constantes e números de syscall que **NUNCA** devem mudar (salvo versionamento).
//! - **Contracts:** Define o contrato de erro (`Errno`) e tipos primitivos (`Pid`, `Time`).
//!
//! ## 🏗️ Arquitetura: System Call Interface
//! O Redstone OS utiliza uma interface baseada em:
//! 1. **Instruction:** `syscall` (x86_64) para transição rápida Ring 3 -> Ring 0.
//! 2. **Registers:** System V AMD64 ABI para passagem de argumentos (RDI, RSI, RDX, R10, R8, R9).
//! 3. **Return:** RAX contém o resultado (positivo) ou erro (negativo, `-errno`).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **POSIX-Like Error Codes:** O uso de `Errno` padrão facilita porting de ferramentas (libc, busybox) e familiaridade.
//! - **Type Aliases:** Em `types.rs`, o uso de `Pid`, `Uid` abstrai a representação interna (embora `Uid` precise morrer).
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Legacy Types:** `Uid`, `Gid` em `types.rs` são resquícios de sistemas multi-usuário UNIX. O Redstone é Capability-based.
//!   - *Conflito:* Isso gera confusão sobre se o kernel deve verificar "Users" ou "Capabilities".
//! - **Lack of vdso:** Não há mecanismo para syscalls rápidas (ex: `gettimeofday` sem entrar no kernel).
//! - **Sync Dispatch:** O dispatcher atual é síncrono. Syscalls bloqueantes travam a thread do kernel (e o core, se não houver preempção).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical)** Remover **Uid/Gid** de `types.rs`.
//!   - *Ação:* Substituir identidade global por identificadores de `Session` ou `Personality`.
//! - [ ] **TODO: (Performance)** Implementar **vDSO (Virtual Dynamic Shared Object)**.
//!   - *Meta:* Mapear página read-only em todo processo para ler relógio do sistema sem syscall.
//! - [ ] **TODO: (Safety)** Implementar **User Pointer Validation** (`copy_from_user` / `copy_to_user`).
//!   - *Risco Atual:* Syscalls acessam ponteiros crus sem verificar se pertencem ao espaço de usuário válido (SMAP/SMEP bypass).
//!
//! --------------------------------------------------------------------------------
//!
//! Contém as constantes e tipos que definem a interface entre o Kernel e o Mundo.

pub mod error;
pub mod types;

pub use error::Errno;

pub mod test;
