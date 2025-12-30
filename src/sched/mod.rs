//! # Módulo de Agendamento (Scheduler)
//!
//! Este módulo é o coração do gerenciamento de processos e threads no RedstoneOS (Forge).
//! Ele é responsável por decidir qual tarefa (Task) deve ser executada pela CPU em um
//! determinado momento, gerenciando a troca de contexto e os estados dos processos.
//!
//! ## Visão Geral da Arquitetura
//!
//! O subsistema de agendamento é dividido em componentes lógicos claros:
//!
//! 1.  **Task Management (Gerenciamento de Tarefas):**
//!     Define o que é uma `Task` (a unidade básica de execução, análoga a thread/processo).
//!     Mantém o estado da tarefa (`Running`, `Ready`, `Blocked`, etc.), seu ID único (`Tid`)
//!     e seu contexto de hardware salvo (registradores).
//!
//! 2.  **Scheduler Core (Núcleo do Agendador):**
//!     Implementa a política de agendamento (atualmente uma fila Round-Robin simples).
//!     Mantém a `RunQueue` (fila de tarefas prontas para rodar) e decide qual será a próxima.
//!     A função `schedule()` é o ponto central que realiza a troca efetiva.
//!
//! 3.  **Context Switching (Troca de Contexto):**
//!     Código de baixo nível (Assembly) necessário para salvar os registradores da tarefa
//!     atual e carregar os da próxima. Isso inclui a troca do Stack Pointer (RSP) e,
//!     se necessário, do espaço de endereçamento (CR3).
//!
//! 4.  **Execution Loader (Carregador e Execução):**
//!     Responsável por criar novas tarefas a partir de binários (ELF), configurar sua
//!     memória virtual inicial, alocar pilhas (stack) e inseri-las na RunQueue.
//!
//! ## Fluxo de Vida de uma Task
//!
//! ```text
//! [Created] --> [Ready] <==> [Running] --> [Blocked]
//!                  ^              |
//!                  |______________|
//! ```
//!
//! - **Created:** A Task está sendo construída (alocação de memória, carregamento de ELF).
//! - **Ready:** A Task está pronta para rodar e aguarda sua vez na `RunQueue`.
//! - **Running:** A Task está sendo executada pela CPU.
//! - **Blocked:** A Task está aguardando um evento (ex: I/O, Mutex, Sleep) e não consome CPU.
//!

// =============================================================================
// SUB-MÓDULOS DE GERENCIAMENTO DE TAREFAS
// =============================================================================

/// Configurações globais do scheduler
pub mod config;

/// Definição da estrutura `Task` e controle de threads.
/// Aqui residem as estruturas que representam um processo ou thread individual
/// e seus metadados (TID, prioridade, pilhas).
pub mod task;

// Re-exportações para facilitar o uso interno e externo (Flattening)
pub use task::{Task, TaskState, Tid};

// =============================================================================
// NÚCLEO DO AGENDADOR (SCHEDULER CORE)
// =============================================================================

/// Implementação dos algoritmos de agendamento e da `RunQueue`.
/// Contém a lógica para `pick_next`, `enqueue` e o loop principal de decisão.
pub mod core;

// Funções principais expostas para o kernel controlar o fluxo
pub use core::{schedule, yield_now};

// =============================================================================
// CARREGAMENTO E EXECUÇÃO (EXECUTION)
// =============================================================================

/// Funcionalidades para carregar programas e criar novos processos.
/// Inclui o parser de ELF e a lógica de `spawn` que configura o ambiente
/// inicial de um processo de usuário.
pub mod exec;

pub use exec::{spawn, ExecError};

// =============================================================================
// SINAIS E COMUNICAÇÃO (SIGNALS)
// =============================================================================

/// Sistema de entrega e tratamento de Sinais (Signals).
/// Permite notificações assíncronas para os processos (ex: SIGKILL, SIGSEGV).
pub mod signal;

// =============================================================================
// SINCRONIZAÇÃO E BLOQUEIO (WAIT)
// =============================================================================

/// Estruturas de listas de espera (Wait Queues).
/// Usadas para colocar tarefas para dormir enquanto aguardam recursos ou eventos,
/// integrando-se com o scheduler para acordá-las posteriormente.
pub mod sync;

pub use sync::WaitQueue;

// =============================================================================
// VÍNCULO COM ASSEMBLY (ASSEMBLY LINKAGE)
// =============================================================================

// Importa o código Assembly que realiza a "mágica" da troca de registradores.
// O `global_asm!` insere o conteúdo do arquivo assembly diretamente na unidade
// de compilação.
::core::arch::global_asm!(include_str!("../arch/x86_64/switch.s"));

extern "C" {
    /// Função de baixo nível em Assembly que efetua a troca de contexto.
    ///
    /// # Argumentos
    ///
    /// * `old_rsp` - Ponteiro mutável para onde salvar o RSP antigo.
    /// * `new_rsp` - O valor do novo RSP a ser carregado.
    pub fn context_switch(old_rsp: *mut u64, new_rsp: u64);
}

// =============================================================================
// TRAMPOLINS (TRAMPOLINES)
// =============================================================================

/// Trampolim de entrada para o Modo de Usuário (Ring 3).
///
/// Esta função é o ponto de "retorno" forçado para iniciar uma tarefa em user space.
/// Ela configura os seletores de segmento de dados para os valores de usuário (RPL 3)
/// e executa `iretq` para pular para o código do usuário com o nível de privilégio correto.
///
/// # Safety
/// Esta função é `naked` (sem prólogo/epílogo) e manipula registradores de segmento
/// diretamente. Se a Stack Frame não estiver configurada corretamente antes do salto
/// para cá, o resultado é indefinido (provavelmente um GP Fault).
#[naked]
#[no_mangle]
pub unsafe extern "C" fn user_entry_trampoline() {
    ::core::arch::asm!(
        "mov ax, 0x23", // Carrega Seletor de Dados de Usuário (Index 4 | RPL 3)
        "mov ds, ax",   // Atualiza DS
        "mov es, ax",   // Atualiza ES
        "mov fs, ax",   // Atualiza FS
        "mov gs, ax",   // Atualiza GS
        "iretq", // Retorna da "interrupção", trocando para Ring 3 e pulando para o RIP do usuário
        options(noreturn)
    );
}

// =============================================================================
// INICIALIZAÇÃO DO SUBSISTEMA
// =============================================================================

/// Inicializa o subsistema de agendamento.
///
/// Deve ser chamado durante o boot do kernel, após a configuração básica de memória
/// e interrupções, mas antes de habilitar a multitarefa preemptiva.
pub fn init() {
    crate::kinfo!("(Sched) Inicializando scheduler...");
    core::init();
    crate::kinfo!("(Sched) Scheduler inicializado");
}

// =============================================================================
// TESTES
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
