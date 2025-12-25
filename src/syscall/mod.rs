//! # Redstone OS Syscall Interface
//!
//! O subsistema de Syscalls é a fronteira definitiva entre o Kernel (Ring 0) e as Aplicações (Ring 3).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Kernel Entry Point:** Define os módulos e a tabela de dispatch para todas as chamadas de sistema.
//! - **API Surface:** O conjunto de funções exportadas aqui constitui a "Standard Library" do mundo bare-metal.
//!
//! ## 🏗️ Arquitetura
//! - **Micro-Modular:** Cada categoria de syscall (processo, memória, ipc) vive em seu próprio submódulo.
//! - **Capability-First:** Syscalls operam sobre `Handles`, não sobre recursos globais (ex: não existe `open("/dev/sda")`, existe `handle_create`).
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Dispatch Síncrono:** O dispatcher atual roda na stack da thread do usuário (kernel stack). Syscalls demoradas (IO) travam a thread.
//!   - *Impacto:* Não há suporte nativo para AIO (Asynchronous IO) real no nível da syscall (io_uring style).
//! - **Falta de Versionamento:** Não há mecanismo para negociar versão da ABI.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Architecture)** Implementar **vDSO** para chamadas de tempo (`clock_get`) sem switch de contexto.
//! - [ ] **TODO: (Security)** Adicionar **Syscall Filter** (seccomp-like) por processo.
//!   - *Meta:* Permitir que um processo (ex: codec de vídeo) inicie e imediatamente feche acesso a todas syscalls exceto `recv_msg` e `yield`.
//!
//! --------------------------------------------------------------------------------
//!
//! Arquitetura capability-based com handles.
//! Numeração própria (NÃO compatível com Linux/POSIX).
//!
//! # Módulos
//!
//! - `abi`: Convenção de chamada, estruturas (IoVec, TimeSpec)
//! - `error`: Códigos de erro (SysError)
//! - `numbers`: Constantes das syscalls
//! - `dispatch`: Dispatcher central
//! - `process`: exit, spawn, wait, yield
//! - `memory`: alloc, free, map, unmap
//! - `handle`: handle_create, dup, close
//! - `ipc`: create_port, send, recv
//! - `io`: readv, writev
//! - `time`: clock, sleep, monotonic
//! - `system`: sysinfo, debug

pub mod abi;
pub mod dispatch;
pub mod error;
pub mod numbers;

// Módulos de implementação
pub mod handle;
pub mod io;
pub mod ipc;
pub mod memory;
pub mod process;
pub mod system;
pub mod time;

// Re-exports principais
pub use dispatch::syscall_dispatcher;
pub use error::{SysError, SysResult};

pub mod test;
