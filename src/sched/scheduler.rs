//! Scheduler Round-Robin (Cooperativo/Preemptivo).
//!
//! Gerencia a fila de tarefas e decide quem roda a seguir.

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

            let next_stack_top = t.stack_top;
            drop(t); // Liberar lock

            self.current_task = Some(next.clone());

            // Se tínhamos uma tarefa antiga, precisamos salvar o contexto dela
            if let Some(old) = old_task_ref {
                // Precisamos retornar endereços de memória onde o assembly
                // vai salvar/ler o RSP (Stack Pointer).
                // Como Task é protegida por Mutex, e o assembly precisa de ponteiros crus,
                // a estratégia segura é:
                // O assembly recebe `&mut old_stack_top` e `new_stack_top`.

                // Hack: Vamos usar unsafe para pegar o ponteiro interno do campo stack_top
                // Isso é perigoso, mas padrão em schedulers.
                let old_ptr = unsafe {
                    let ptr = &mut old.lock().stack_top as *mut u64;
                    ptr as u64
                };

                return Some((old_ptr, next_stack_top));
            } else {
                // Primeira tarefa (boot -> task 1). Não salvamos o contexto do boot.
                // Apenas carregamos o novo.
                return Some((0, next_stack_top));
            }
        }

        None
    }
}

/// Função auxiliar para inicializar multitarefa.
/// Cria a tarefa Idle ou Init.
pub fn init() {
    let mut sched = SCHEDULER.lock();
    // Exemplo: Criar tarefa de teste
    sched.add_task(Task::new(example_task));
    sched.add_task(Task::new(example_task_2));
}

extern "C" fn example_task() {
    loop {
        crate::kprint!("A");
        // Loop de delay burro
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
}

extern "C" fn example_task_2() {
    loop {
        crate::kprint!("B");
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
}
