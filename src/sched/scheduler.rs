//! Scheduler Round-Robin (Cooperativo/Preemptivo).
//!
//! Gerencia a fila de tarefas e decide quem roda a seguir.
//!
//! # Mecânica
//! O scheduler seleciona a próxima tarefa `Ready` e realiza a troca de contexto.
//! O estado da CPU (registradores) é salvo no `Context` da tarefa antiga e
//! restaurado do `Context` da nova.

use super::task::{Task, TaskId, TaskState};
use crate::sync::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;

pub struct Scheduler {
    tasks: VecDeque<Arc<Mutex<Task>>>,
    current_task: Option<Arc<Mutex<Task>>>,
}

// O Scheduler é global e protegido.
pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            current_task: None,
        }
    }

    /// Adiciona uma tarefa à fila de prontos.
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push_back(Arc::new(Mutex::new(task)));
    }

    /// Chamado pelo Timer Interrupt para trocar tarefas.
    /// Retorna (ponteiro para stack antiga, ponteiro para stack nova) se houver troca.
    ///
    /// # Safety
    /// Retorna ponteiros brutos que serão usados em assembly. O caller deve garantir
    /// que os locks sejam tratados corretamente (embora aqui retornemos cópias/endereços).
    pub fn schedule(&mut self) -> Option<(u64, u64)> {
        // Se não há tarefas, nada a fazer
        if self.tasks.is_empty() {
            return None;
        }

        let old_task_ref = self.current_task.take();

        // Se havia tarefa rodando, move para o fim da fila
        if let Some(old) = old_task_ref.clone() {
            let mut t = old.lock();
            if t.state == TaskState::Running {
                t.state = TaskState::Ready;
            }
            drop(t); // Liberar lock antes de mover
            self.tasks.push_back(old);
        }

        // Pegar próxima tarefa
        if let Some(next) = self.tasks.pop_front() {
            let mut t = next.lock();
            t.state = TaskState::Running;

            // O novo RSP é o que foi salvo no contexto da tarefa
            let next_rsp = t.context.rsp;
            drop(t); // Liberar lock

            self.current_task = Some(next.clone());

            if let Some(old) = old_task_ref {
                // Precisamos do endereço de memória onde o assembly deve salvar o RSP antigo.
                // &mut old.context.rsp
                let old_rsp_ptr = unsafe {
                    let ptr = &mut old.lock().context.rsp as *mut u64;
                    ptr as u64
                };

                return Some((old_rsp_ptr, next_rsp));
            } else {
                // Primeira tarefa (boot -> task 1). Não salvamos o contexto do boot (0).
                return Some((0, next_rsp));
            }
        }

        None
    }
}

/// Função auxiliar para inicializar multitarefa.
/// Cria as tarefas iniciais (Init/Idle).
pub fn init() {
    let mut sched = SCHEDULER.lock();

    // Criar tarefas de kernel para teste
    sched.add_task(Task::new_kernel(example_task_a));
    sched.add_task(Task::new_kernel(example_task_b));

    crate::kinfo!("[Scheduler] Tasks initialized.");
}

extern "C" fn example_task_a() {
    loop {
        crate::kprint!("A");
        // Loop de delay (spin loop)
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
}

extern "C" fn example_task_b() {
    loop {
        crate::kprint!("B");
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
}
