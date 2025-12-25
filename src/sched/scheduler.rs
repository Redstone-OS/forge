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
//!
//! # Nota sobre Arc
//! Usamos Box em vez de Arc porque Arc::new causa GPF 0x32 (Invalid Opcode)
//! devido a problemas com a inicialização do contador atômico em bare-metal.

use super::task::{PinnedTask, Task, TaskState};
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
        crate::kinfo!("[Sched] add_task...");
        // Mutex envolve o Pin - Task NUNCA move
        let wrapped = Box::new(Mutex::new(task));
        crate::kinfo!("[Sched] add_task: push_back...");
        self.tasks.push_back(wrapped);
        crate::kinfo!("[Sched] add_task: OK");
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
                    crate::kinfo!(
                        "[Sched] switch: old_rsp_ptr={:#x} next_rsp={:#x} task={:?}",
                        old_rsp_ptr,
                        next_rsp,
                        next_id
                    );
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
    let mut sched = SCHEDULER.lock();

    crate::kinfo!("[Sched] Scheduler inicializado (sem tarefas de teste)");

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
}

/// Força a troca de contexto voluntária (Yield).
///
/// Chama a interrupção de timer (0x20) via software para invocar o scheduler.
pub fn yield_now() {
    unsafe {
        core::arch::asm!("int 0x20");
    }
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
