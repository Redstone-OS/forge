//! # Round-Robin Scheduler
//!
//! O `scheduler` orquestra a execução de tarefas na CPU.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Queue Management:** Mantém uma fila de tarefas prontas (`Ready`).
//! - **Context Switching:** Calcula quem é o próximo a rodar e instrui a substituição de pilhas (RSP).
//! - **Cooperative/Preemptive:** Suporta ambos os modelos via `yield_now()` e Timer Interrupt.
//!
//! ## 🏗️ Arquitetura: Global Round-Robin
//! Implementação clássica de fila circular (`VecDeque`):
//! - `schedule()`: Remove a cabeça da fila, coloca a tarefa atual no final, e retorna o par de ponteiros para o switch assembly.
//! - **Global Lock:** Uma única instância `SCHEDULER` protegida por `Mutex` serve o sistema inteiro.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Justiça (Fairness):** Round-Robin garante que todas as tarefas recebam tempo de CPU, prevenindo *starvation* completa.
//! - **Simplicidade:** Algoritmo O(1) para enqueue/dequeue, ideal para boots iniciais ou sistemas simples.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Scalability Nightmare:** O `Mutex<Scheduler>` global é um gargalo severo. Em um sistema com 4 cores, 3 ficarão esperando enquanto 1 decide o agendamento.
//! - **Double Locking:** `VecDeque<Box<Mutex<PinnedTask>>>` implica adquirir dois locks para agendar: um para a fila, outro para a tarefa. Deadlocks são possíveis se a ordem mudar.
//! - **No Priority:** Tarefas críticas (drivers de áudio/input) rodam com a mesma frequência que tarefas de fundo (compilação). Isso destrói a latência percebida.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Performance)** Migrar para **Per-CPU Runqueues**.
//!   - *Meta:* Remover o lock global. Cada CPU agenda suas próprias tarefas (Work Stealing opcional).
//! - [ ] **TODO: (Algorithm)** Implementar **Multilevel Feedback Queue** ou Priority Queue.
//!   - *Motivo:* Priorizar tarefas interativas (IO-bound) sobre tarefas CPU-bound.
//! - [ ] **TODO: (Optimization)** Remover `Box<Mutex<...>>` interno se mudarmos para Per-CPU queues exclusivas (sem lock na task).
//!

use super::task::{PinnedTask, TaskState};
use crate::sync::Mutex;
use alloc::boxed::Box;
use alloc::collections::VecDeque;

/// Estrutura do Scheduler Global.
pub struct Scheduler {
    /// Fila de tarefas prontas para rodar (Ready).
    tasks: VecDeque<Box<Mutex<PinnedTask>>>,
    /// Tarefa atualmente em execução na CPU.
    current_task: Option<Box<Mutex<PinnedTask>>>,
}

/// Instância global do Scheduler, protegida por Mutex.
/// Em sistemas SMP, isso seria per-cpu ou teria locking mais granular.
pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

impl Scheduler {
    /// Cria um novo scheduler vazio.
    pub const fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            current_task: None,
        }
    }

    /// Adiciona uma tarefa à fila de prontos.
    /// A tarefa será agendada na próxima oportunidade.
    pub fn add_task(&mut self, task: PinnedTask) {
        let id = task.id.as_u64();
        crate::ktrace!("(Sched) add_task: Adicionando PID=", id);
        // Mutex envolve o Pin - Task NUNCA move
        let wrapped = Box::new(Mutex::new(task));
        self.tasks.push_back(wrapped);
        crate::kdebug!("(Sched) Tarefa PID adicionada ao escalonador: PID=", id);
    }

    /// Executa o algoritmo de agendamento (Round-Robin).
    ///
    /// # Retorno
    /// * `Some((old_ptr, new_ptr))`: Endereços para realizar o switch em Assembly.
    /// * `None`: Nenhuma troca necessária (fila vazia ou apenas uma tarefa).
    ///
    /// # Safety
    /// Retorna ponteiros crus que devem ser usados imediatamente pelo `context_switch`.
    /// O lock das tarefas é liberado antes de retornar para evitar deadlocks durante o switch.
    pub fn schedule(&mut self) -> Option<(u64, u64)> {
        // Se não há tarefas na fila (além da atual), não faz nada.
        if self.tasks.is_empty() {
            return None;
        }

        // 1. Processar a tarefa atual (Old)
        let old_task_ref = self.current_task.take();

        // Calcular old_rsp_ptr ANTES de re-enfileirar (senão perdemos a referência!)
        let old_rsp_ptr = if let Some(ref old) = old_task_ref {
            let mut pinned = old.lock();
            // SAFETY: Acessando campos internos do Pin<Box<Task>>
            let t = unsafe { pinned.as_mut().get_unchecked_mut() };
            // Se estava rodando, volta para o estado Ready.
            if t.state == TaskState::Running {
                t.state = TaskState::Ready;
            }
            let ptr = &mut t.kstack_top as *mut u64;
            ptr as u64
        } else {
            0 // Primeira troca, não há tarefa antiga
        };

        // Re-enfileirar a tarefa antiga no fim da fila
        if let Some(old) = old_task_ref {
            self.tasks.push_back(old);
        }

        // 2. Escolher a próxima tarefa (Next)
        if let Some(next) = self.tasks.pop_front() {
            let mut pinned = next.lock();
            // SAFETY: Acessando campos internos do Pin<Box<Task>>
            let t = unsafe { pinned.as_mut().get_unchecked_mut() };
            t.state = TaskState::Running;

            // Obter o valor do Stack Pointer onde a tarefa parou.
            let next_rsp = t.kstack_top;
            let next_id = t.id;

            drop(pinned); // Liberar lock

            // Atualizar referência global
            self.current_task = Some(next);

            // Debug: mostrar troca (apenas a cada 100 ticks para não poluir)
            static mut TICK_COUNT: u64 = 0;
            unsafe {
                TICK_COUNT += 1;
                if TICK_COUNT % 100 == 1 {
                    crate::klog!("[TRAC] (Sched) switch: [", old_rsp_ptr, " -> ", next_rsp);
                    crate::klog!("] tarefa=", next_id.as_u64());
                    crate::knl!();
                }
            };

            return Some((old_rsp_ptr, next_rsp));
        }

        None
    }
}

/// Inicializa o subsistema de multitarefa.
/// Cria tarefas iniciais para teste.
pub fn init() {
    let _sched = SCHEDULER.lock();
    crate::kinfo!("(Sched) Inicializado (Escalonador Round-Robin)");
}

// Tarefas de teste comentadas para testar init sozinho
/*
crate::kinfo!("[Teste] Criando tarefas do kernel...");

// Criar Tasks de Kernel
crate::kinfo!("[Sched] Criando task_a...");
sched.add_task(Task::new_kernel(task_a));
crate::kinfo!("[Sched] task_a adicionada OK");

crate::kinfo!("[Sched] Criando task_b...");
sched.add_task(Task::new_kernel(task_b));
crate::kinfo!("[Sched] task_b adicionada OK");

crate::kinfo!("[Sched] Criando task_c...");
sched.add_task(Task::new_kernel(task_c));
crate::kinfo!("[Sched] task_c adicionada OK");
*/

/// Força a troca de contexto voluntária (Yield).
///
/// Chama a interrupção de timer (0x20) via software para invocar o scheduler.
pub fn yield_now() {
    unsafe {
        core::arch::asm!("int 0x20");
    }
}

// --- Tarefas de Teste ---

#[allow(dead_code)]
extern "C" fn task_a() {
    loop {
        crate::klog!("A");
        spin_delay(500000);
    }
}

#[allow(dead_code)]
extern "C" fn task_b() {
    loop {
        crate::klog!("B");
        spin_delay(500000);
    }
}

#[allow(dead_code)]
extern "C" fn task_c() {
    loop {
        crate::klog!("C");
        spin_delay(500000);
    }
}

#[allow(dead_code)]
fn spin_delay(count: usize) {
    for _ in 0..count {
        core::hint::spin_loop();
    }
}
