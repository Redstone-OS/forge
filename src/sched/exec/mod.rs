//! # Execution and Process Creation
//!
//! Este módulo é responsável por carregar executáveis e criar novos processos.
//! Ele gerencia todo o ciclo de vida da criação de um processo:
//!
//! 1. **Parsing** - Interpretação do formato binário (ELF, scripts)
//! 2. **Address Space Setup** - Criação do espaço de endereçamento isolado
//! 3. **Memory Mapping** - Mapeamento de segmentos de código, dados e stack
//! 4. **Context Setup** - Configuração do contexto de CPU para primeira execução
//! 5. **Scheduling** - Enfileiramento da nova task no scheduler
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                           sched::exec                                   │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │   spawn(path, parent) ─────────────────────────────────────────────┐    │
//! │         │                                                          │    │
//! │         ▼                                                          │    │
//! │   ┌─────────────┐      ┌─────────────┐     ┌─────────────┐         │    │
//! │   │ fmt::detect │────▶│ fmt::elf    │────▶│  process    │         │    │
//! │   │   (magic)   │      │  (parser)   │     │  (setup)    │         │    │
//! │   └─────────────┘      └─────────────┘     └─────────────┘         │    │
//! │         │                                        │                 │    │
//! │         ▼ (script?)                              ▼                 │    │
//! │   ┌─────────────┐                        ┌─────────────┐           │    │
//! │   │ fmt::script │                        │  context    │           │    │
//! │   │  (shebang)  │                        │ (trap frame)│           │    │
//! │   └─────────────┘                        └─────────────┘           │    │
//! │                                                                    │    │
//! └────────────────────────────────────────────────────────────────────┘    │
//!                                                                           │
//!                                            ┌─────────────┐                │
//!                                            │   config    │◀───────────────┘
//!                                            │ (constantes)│
//!                                            └─────────────┘
//! ```
//!
//! ## Módulos
//!
//! - [`config`] - Constantes de memória e seletores de segmento
//! - [`error`] - Tipos de erro para operações de execução
//! - [`process`] - Orquestração do spawn de processos
//! - [`context`] - Setup do contexto de CPU (trap frame)
//! - [`fmt`] - Formatos de executáveis suportados (ELF, scripts)
//!
//! ## Uso
//!
//! ```rust,ignore
//! use crate::sched::exec::{spawn, ExecError};
//!
//! // Spawnar processo a partir de binário ELF
//! match spawn("/bin/init", None) {
//!     Ok(pid) => kinfo!("Processo criado: PID", pid.as_u32()),
//!     Err(ExecError::NotFound) => kerror!("Arquivo não encontrado"),
//!     Err(e) => kerror!("Erro ao spawnar processo"),
//! }
//! ```
//!
//! ## Formatos Suportados
//!
//! | Formato      | Status     | Descrição                        |
//! |--------------|------------|----------------------------------|
//! | ELF64 EXEC   | Suportado  | Executáveis estáticos x86_64     |
//! | ELF64 DYN    | Planejado  | PIE não suportado (retorna erro) |
//! | Scripts (#!) | Planejado  | Shebang para interpreters        |
//!
//! ## Integração com RMM
//!
//! Este módulo usa o novo Redstone Memory Manager (RMM) para:
//! - Alocar frames físicos para código, dados e stacks
//! - Criar e gerenciar Address Spaces isolados por processo
//! - Mapear páginas no espaço de endereçamento do processo alvo
//!
//! ## Segurança
//!
//! - Cada processo recebe um Address Space isolado (CR3 próprio)
//! - Stack de kernel separada por processo
//! - Validação de bounds em headers ELF
//! - Rejeição de formatos não suportados com erro explícito

pub mod config;
pub mod context;
pub mod error;
pub mod fmt;
pub mod process;

// Re-exports públicos
pub use error::ExecError;
pub use process::spawn;
