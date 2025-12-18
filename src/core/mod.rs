//! Núcleo do Kernel Forge
//!
//! Este módulo contém os componentes fundamentais do kernel:
//! - Scheduler (CFS - Completely Fair Scheduler)
//! - Gerenciamento de processos e threads  
//! - Sistema de inicialização por fases
//!
//! # Arquitetura
//! O núcleo segue o modelo de processos pesados + threads leves (estilo Linux).
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar CFS scheduler
//! - TODO(prioridade=alta, versão=v1.0): Migrar código de context/ para process/
//! - TODO(prioridade=média, versão=v2.0): Adicionar suporte a namespaces

pub mod elf;
pub mod init;
pub mod process;
pub mod sched;
pub mod scheduler;
pub mod thread;
