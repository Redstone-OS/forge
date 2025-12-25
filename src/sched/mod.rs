//! # Multitasking & Scheduler Subsystem
//!
//! O módulo `sched` é o motor de execução do Redstone OS. Ele transforma o hardware single-threaded
//! (ou multi-core físico) em uma abstração capaz de executar múltiplas tarefas "simultaneamente".
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Abstração de Tarefa:** Define o que é uma `Task` (PCB - Process Control Block) e seu ciclo de vida.
//! - **Troca de Contexto:** Gerencia a mágica do `context_switch` assembly para salvar/restaurar estado.
//! - **Política de Escalonamento:** Decide *quem* roda e *por quanto tempo* (atualmente Round-Robin).
//!
//! ## 🏗️ Arquitetura: Cooperative + Preemptive
//! O design atual é híbrido:
//! 1. **Preemptivo:** O Timer Interrupt (IRQ 0) chama o scheduler periodicamente (Timeslice).
//! 2. **Cooperativo:** Tarefas podem ceder CPU voluntariamente via `yield_now()`.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Interface Limpa:** A separação entre `Context`, `Task` e `Scheduler` está bem definida.
//! - **Memory Safety:** O uso de `PinnedTask` (`Pin<Box<Task>>`) previne erros catastróficos de use-after-free
//!   ou movimentação de stack ativa na memória.
//! - **Trampoline Explícito:** A função `user_entry_trampoline` documenta claramente a transição Ring 0 -> Ring 3.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Bare-Metal Naked Functions:** O trampoline está implementado como função `#[naked]`. Embora funcione,
//!   esconde complexidade de stack frame que seria melhor gerida em .asm puro.
//! - **Global Lock Contention:** O `SCHEDULER` é protegido por um único Mutex. Em multicore, isso será o maior gargalo do sistema.
//! - **Missing FPU State:** O contexto atual NÃO salva registradores SSE/AVX. Se uma thread usar float e trocar de contexto,
//!   corromperá o estado da outra thread. (Isso é um BUG crítico em potencial).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical/Bug)** Salvar/Restaurar contexto **FPU/SSE/AVX** (`fxsave`/`fxrstor`).
//!   - *Risco:* Cálculos flutuantes em userspace vão colidir e gerar dados corrompidos aleatoriamente.
//! - [ ] **TODO: (SMP)** Suporte a **Per-CPU Runqueues**.
//!   - *Meta:* Eliminar o lock global do scheduler para escalar linearmente com número de cores.
//! - [ ] **TODO: (Feature)** Implementar **Priority Scheduling** (Feedback Queue).
//!   - *Motivo:* Processos de UI não podem esperar processos de background (compilação/backup).
//! - [ ] **TODO: (Arch)** Mover `user_entry_trampoline` para `src/arch/x86_64/trampoline.s`.

pub mod context;
pub mod scheduler;
pub mod task;
pub mod test;

// Importa o assembly de troca de contexto
core::arch::global_asm!(include_str!("../arch/x86_64/switch.s"));

extern "C" {
    /// Função assembly definida em switch.s
    pub fn context_switch(old_rsp_ptr: *mut u64, new_rsp: u64);
}

/// Trampolim para pular para Userspace.
#[naked]
pub unsafe extern "C" fn user_entry_trampoline() {
    core::arch::asm!(
        // Restaurar segmentos de dados de usuário (Ring 3)
        "mov ax, 0x23", // USER_DATA_SEL (0x20) | RPL 3
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        // A stack já tem [RIP, CS, RFLAGS, RSP, SS] empilhados
        // Executar IRETQ para trocar de Ring 0 -> Ring 3
        "iretq",
        options(noreturn)
    );
}
