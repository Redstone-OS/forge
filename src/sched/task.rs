//! Definição de Tarefa (Task/Process Control Block).
//!
//! Representa uma unidade de execução agendável no Redstone OS.
//! Suporta tanto tarefas de Kernel (Ring 0) quanto Processos de Usuário (Ring 3).
//!
//! # Estrutura de Memória
//! Cada tarefa possui sua própria Kernel Stack (kstack).
//! - Tasks de Kernel: Rodam inteiramente nesta stack.
//! - Tasks de Usuário: Usam esta stack apenas ao entrar no kernel (Syscalls/Interrupts).

use crate::arch::x86_64::gdt::{KERNEL_CODE_SEL, KERNEL_DATA_SEL, USER_CODE_SEL, USER_DATA_SEL};
use crate::sched::context::Context;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

// REMOVIDO: extern "C" { fn user_entry_trampoline(); }
// A solução profissional usa o caminho do módulo Rust para garantir que o compilador
// resolva o endereço corretamente, sem depender de strings de símbolos no Linker.
use crate::sched::user_entry_trampoline;

/// ID único de tarefa (PID/TID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(u64);

impl TaskId {
    /// Gera um novo ID atômico.
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

    // Stack do Kernel (propriedade da tarefa).
    pub kstack: Vec<u8>,

    // Topo da stack (Stack Pointer Salvo).
    // Atualizado pelo Scheduler durante o Context Switch.
    pub kstack_top: u64,

    // Endereço Físico da Tabela de Páginas (PML4).
    // 0 = Usa o mapeamento padrão do Kernel.
    pub cr3: u64,
}

impl Task {
    /// Cria uma nova tarefa de Kernel (Ring 0).
    pub fn new_kernel(entry: extern "C" fn()) -> Self {
        let mut task = Self::create_base();
        // Configura stack para execução direta no Kernel
        task.setup_stack(entry as u64, KERNEL_CODE_SEL, KERNEL_DATA_SEL, 0);
        task
    }

    /// Cria uma nova tarefa de Usuário (Ring 3).
    ///
    /// # Arguments
    /// * `entry_point`: Endereço virtual (RIP) no userspace.
    /// * `user_stack_top`: Endereço virtual (RSP) da stack no userspace.
    /// * `cr3`: Endereço físico da tabela de páginas do processo.
    pub fn new_user(entry_point: u64, user_stack_top: u64, cr3: u64) -> Self {
        let mut task = Self::create_base();
        task.cr3 = cr3;

        // Configura stack para retorno ao Userspace via IRETQ
        task.setup_stack(entry_point, USER_CODE_SEL, USER_DATA_SEL, user_stack_top);

        task
    }

    /// Aloca estrutura base e stack alinhada.
    fn create_base() -> Self {
        const STACK_SIZE: usize = 32 * 1024; // 32KB
        let mut kstack = Vec::with_capacity(STACK_SIZE);

        // Inicialização segura da memória e ajuste de tamanho
        unsafe {
            kstack.set_len(STACK_SIZE);
            core::ptr::write_bytes(kstack.as_mut_ptr(), 0, STACK_SIZE);
        }

        // Calcular topo da stack com alinhamento de 16 bytes (System V ABI)
        let stack_start = kstack.as_ptr() as u64;
        let stack_end = stack_start + STACK_SIZE as u64;
        let kstack_top = stack_end & !0xF;

        Self {
            id: TaskId::new(),
            state: TaskState::Ready,
            context: Context::empty(),
            kstack,
            kstack_top,
            cr3: 0,
        }
    }

    /// Prepara a stack para o primeiro Context Switch.
    /// Constrói um stack frame artificial que simula uma tarefa interrompida.
    fn setup_stack(&mut self, rip: u64, cs: u16, ss: u16, user_rsp: u64) {
        unsafe {
            let mut ptr = self.kstack_top as *mut u64;

            // 1. Se for tarefa de usuário, empilhar frame IRETQ
            if cs == USER_CODE_SEL {
                // Layout: [SS, RSP, RFLAGS, CS, RIP]
                ptr = ptr.sub(1);
                *ptr = ss as u64; // SS
                ptr = ptr.sub(1);
                *ptr = user_rsp; // RSP (User)
                ptr = ptr.sub(1);
                *ptr = 0x202; // RFLAGS (Interrupts Enabled)
                ptr = ptr.sub(1);
                *ptr = cs as u64; // CS
                ptr = ptr.sub(1);
                *ptr = rip; // RIP (User Entry)

                // Endereço de retorno do 'ret' no switch.s: Trampolim
                // CORREÇÃO: Usamos o símbolo importado do módulo Rust diretamente.
                ptr = ptr.sub(1);
                *ptr = user_entry_trampoline as usize as u64;
            } else {
                // Tarefa de Kernel: Endereço de retorno direto
                ptr = ptr.sub(1);
                *ptr = rip;
            }

            // 2. Empilhar registradores Callee-Saved (RBX, RBP, R12-R15)
            // O switch.s vai dar 'pop' nestes valores.
            // Inicializamos com 0 para evitar lixo.
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

            // 3. Salvar o novo topo da stack
            self.kstack_top = ptr as u64;
        }
    }
}
