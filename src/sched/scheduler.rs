//! Scheduler Round-Robin (Cooperativo/Preemptivo).
//!
//! Gerencia a fila de tarefas e decide quem roda a seguir.
//!
//! # Mecânica de Troca (Context Switch)
//! O scheduler não troca registradores individualmente. Ele troca o **Stack Pointer (RSP)**.
//! 1. O Assembly `context_switch` salva os registradores na pilha *atual*.
//! 2. O Assembly salva o RSP atual em `old_task.kstack_top`.
//! 3. O Assembly carrega o RSP de `new_task.kstack_top`.
//! 4. O Assembly restaura os registradores da *nova* pilha.

use super::task::{Task, TaskState};
use crate::sync::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;

/// Estrutura do Scheduler Global.
pub struct Scheduler {
    /// Fila de tarefas prontas para rodar (Ready).
    tasks: VecDeque<Arc<Mutex<Task>>>,
    /// Tarefa atualmente em execução na CPU.
    current_task: Option<Arc<Mutex<Task>>>,
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
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push_back(Arc::new(Mutex::new(task)));
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
        // Nota: Em um sistema real, teríamos uma "Idle Task" sempre pronta.
        if self.tasks.is_empty() {
            return None;
        }

        // 1. Processar a tarefa atual (Old)
        let old_task_ref = self.current_task.take();
        if let Some(old) = old_task_ref.clone() {
            let mut t = old.lock();

            // Se estava rodando, volta para o estado Ready e vai para o fim da fila.
            if t.state == TaskState::Running {
                t.state = TaskState::Ready;
            }
            // Se estava Blocked/Terminated, não re-enfileira.

            drop(t); // Liberar lock
            self.tasks.push_back(old);
        }

        // 2. Escolher a próxima tarefa (Next)
        if let Some(next) = self.tasks.pop_front() {
            let mut t = next.lock();
            t.state = TaskState::Running;

            // Obter o valor do Stack Pointer onde a tarefa parou.
            let next_rsp = t.kstack_top;

            drop(t); // Liberar lock

            // Atualizar referência global
            self.current_task = Some(next.clone());

            // 3. Preparar ponteiros para o switch
            if let Some(old) = old_task_ref {
                // Precisamos do ENDEREÇO do campo `kstack_top` da tarefa antiga.
                // O assembly vai escrever o RSP atual nesse endereço.
                let old_rsp_ptr = unsafe {
                    let ptr = &mut old.lock().kstack_top as *mut u64;
                    ptr as u64
                };

                return Some((old_rsp_ptr, next_rsp));
            } else {
                // Primeira troca (Boot -> Task 1).
                // Não salvamos o contexto do Boot (passamos 0).
                return Some((0, next_rsp));
            }
        }

        None
    }
}

/// Inicializa o subsistema de multitarefa.
/// Cria tarefas iniciais para teste.
pub fn init() {
    let mut sched = SCHEDULER.lock();

    crate::kinfo!("[Scheduler] Creating kernel tasks...");

    // Criar Tasks de Kernel
    sched.add_task(Task::new_kernel(task_a));
    sched.add_task(Task::new_kernel(task_b));
    sched.add_task(Task::new_kernel(task_c));
}

// --- Tarefas de Teste ---

extern "C" fn task_a() {
    loop {
        crate::kprint!("A");
        spin_delay(500000);
    }
}

extern "C" fn task_b() {
    loop {
        crate::kprint!("B");
        spin_delay(500000);
    }
}

extern "C" fn task_c() {
    loop {
        crate::kprint!("C");
        spin_delay(500000);
    }
}

fn spin_delay(count: usize) {
    for _ in 0..count {
        core::hint::spin_loop();
    }
}
