//! Definição de Tarefa (Task/Process Control Block).
//!
//! Representa uma unidade de execução agendável no Redstone OS.
//! Suporta tanto tarefas de Kernel (Ring 0) quanto Processos de Usuário (Ring 3).
//!
//! # Estrutura de Memória
//! Cada tarefa possui sua própria Kernel Stack.
//! - Tasks de Kernel rodam inteiramente nesta stack.
//! - Tasks de Usuário usam esta stack apenas durante Syscalls/Interrupções (via TSS RSP0).

use crate::arch::platform::gdt::{KERNEL_CODE_SEL, KERNEL_DATA_SEL, USER_CODE_SEL, USER_DATA_SEL};
use crate::sched::context::Context;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// ID único de tarefa (PID/TID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(u64);

impl TaskId {
    /// Gera um novo ID atômico, sequencial e thread-safe.
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Estado do ciclo de vida da tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Ready,
    Blocked,
    Terminated,
}

/// A estrutura da Tarefa (PCB - Process Control Block).
pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub context: Context,

    // Stack do Kernel (usada durante syscalls/interrupts e tasks ring0)
    // O Vec garante a posse da memória.
    pub kstack: Vec<u8>,

    // Topo da stack (endereço virtual).
    // É atualizado durante o context switch para salvar onde a execução parou.
    pub kstack_top: u64,

    // Endereço Físico da Tabela de Páginas (PML4).
    // Se 0, a tarefa usa o mapeamento do kernel (compartilhado).
    pub cr3: u64,
}

impl Task {
    /// Cria uma nova tarefa de Kernel (Ring 0).
    ///
    /// # Arguments
    /// * `entry`: Ponteiro para a função Rust `extern "C"` a ser executada.
    pub fn new_kernel(entry: extern "C" fn()) -> Self {
        let mut task = Self::create_base();
        // Kernel roda com CS=0x08, SS=0x10, RFLAGS=Default
        task.setup_stack(entry as u64, KERNEL_CODE_SEL, KERNEL_DATA_SEL, 0);
        task
    }

    /// Cria uma nova tarefa de Usuário (Ring 3).
    ///
    /// # Arguments
    /// * `entry_point`: Endereço virtual (RIP) no userspace.
    /// * `user_stack_top`: Endereço virtual (RSP) da stack no userspace.
    /// * `cr3`: Endereço físico da tabela de páginas (PML4) isolada deste processo.
    pub fn new_user(entry_point: u64, user_stack_top: u64, cr3: u64) -> Self {
        let mut task = Self::create_base();
        task.cr3 = cr3;

        // Configurar stack frame para retorno ao Userspace via IRETQ.
        // Stack Frame IRETQ: [SS, RSP, RFLAGS, CS, RIP]
        task.setup_stack(entry_point, USER_CODE_SEL, USER_DATA_SEL, user_stack_top);

        task
    }

    /// Aloca a estrutura básica e a stack do kernel.
    fn create_base() -> Self {
        // 32KB de Stack para o Kernel (seguro para syscalls profundas/interrupções)
        let stack_size = 32 * 1024;
        let mut kstack = Vec::with_capacity(stack_size);

        // Inicializar com zeros para evitar vazamento de dados antigos e facilitar debug
        unsafe {
            kstack.set_len(stack_size);
            core::ptr::write_bytes(kstack.as_mut_ptr(), 0, stack_size);
        }

        // Calcular o topo da stack (Stack cresce para baixo)
        // Alinhamento de 16 bytes é CRÍTICO para System V ABI (SSE instructions vai crashar se desalinhado)
        let stack_start = kstack.as_ptr() as u64;
        let stack_end = stack_start + stack_size as u64;
        let kstack_top = stack_end & !0xF; // Alinhar para baixo em 16 bytes

        Self {
            id: TaskId::new(),
            state: TaskState::Ready,
            context: Context::empty(),
            kstack,
            kstack_top,
            cr3: 0, // 0 = Usa CR3 atual (Kernel Space)
        }
    }

    /// Prepara a stack do kernel para o primeiro Context Switch.
    /// Simula que a tarefa foi "interrompida" anteriormente.
    fn setup_stack(&mut self, rip: u64, cs: u16, ss: u16, user_rsp: u64) {
        unsafe {
            let mut ptr = self.kstack_top as *mut u64;

            // --- Parte 1: Frame de Interrupção (Apenas se for para Userspace) ---
            // Se formos para Ring 3, precisamos forjar o frame que 'iretq' espera desempilhar.
            // Se for Ring 0, o 'ret' do switch vai direto para a função.

            if cs == USER_CODE_SEL {
                // Layout IRETQ:
                ptr = ptr.sub(1);
                *ptr = ss as u64; // SS (User Data)
                ptr = ptr.sub(1);
                *ptr = user_rsp; // RSP (User Stack)
                ptr = ptr.sub(1);
                *ptr = 0x202; // RFLAGS (IF=1, Interrupts Enabled)
                ptr = ptr.sub(1);
                *ptr = cs as u64; // CS (User Code)
                ptr = ptr.sub(1);
                *ptr = rip; // RIP (User Entry)

                // O `context_switch` assembly termina com RET.
                // O RET vai pular para o endereço no topo da stack.
                // Não podemos pular direto para o RIP do usuário porque precisamos rodar IRETQ.
                // Então pulamos para um "Trampolim" no kernel.
                ptr = ptr.sub(1);
                *ptr = crate::sched::user_entry_trampoline as u64;
            } else {
                // Kernel Task: O RET pula direto para a função
                ptr = ptr.sub(1);
                *ptr = rip;
            }

            // --- Parte 2: Registradores "Callee-Saved" (Contexto de Software) ---
            // O `context_switch` salva/restaura: RBX, RBP, R12, R13, R14, R15.
            // Precisamos colocar zeros aqui para o primeiro restore.

            ptr = ptr.sub(1);
            *ptr = 0; // RBP
            ptr = ptr.sub(1);
            *ptr = 0; // RBX
            ptr = ptr.sub(1);
            *ptr = 0; // R12
            ptr = ptr.sub(1);
            *ptr = 0; // R13
            ptr = ptr.sub(1);
            *ptr = 0; // R14
            ptr = ptr.sub(1);
            *ptr = 0; // R15

            // --- Finalização ---
            // Atualizar o ponteiro de stack no contexto salvo.
            // Quando o scheduler trocar para esta task, ele carregará este RSP em `context.rsp`.
            self.context.rsp = ptr as u64;
        }
    }
}
