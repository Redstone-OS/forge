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
use crate::core::handle::HandleTable;
use crate::sched::context::Context;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::pin::Pin;
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

    /// Retorna o valor numérico do ID.
    pub fn as_u64(&self) -> u64 {
        self.0
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

    /// Tabela de handles do processo (capability-based).
    pub handles: HandleTable,
}

/// Tipo alias para Task pinada - nunca pode mover após criação.
/// Isso garante que kstack_top sempre aponta para memória válida.
pub type PinnedTask = Pin<Box<Task>>;

impl Task {
    /// Cria uma nova tarefa de Kernel (Ring 0).
    pub fn new_kernel(entry: extern "C" fn()) -> PinnedTask {
        let mut task = Box::pin(Self::create_base());
        // SAFETY: Task pinada, só mutamos campos internos
        unsafe {
            let t = task.as_mut().get_unchecked_mut();
            t.setup_stack(entry as u64, KERNEL_CODE_SEL, KERNEL_DATA_SEL, 0);
        }
        crate::kdebug!("[Task] new_kernel OK: entry={:#x}", entry as usize);
        task
    }

    /// Cria uma nova tarefa de Usuário (Ring 3).
    ///
    /// # Arguments
    /// * `entry_point`: Endereço virtual (RIP) no userspace.
    /// * `user_stack_top`: Endereço virtual (RSP) da stack no userspace.
    /// * `cr3`: Endereço físico da tabela de páginas do processo.
    pub fn new_user(entry_point: u64, user_stack_top: u64, cr3: u64) -> PinnedTask {
        crate::kinfo!(
            "[Task] new_user INICIO: entry={:#x} stack={:#x} cr3={:#x}",
            entry_point,
            user_stack_top,
            cr3
        );

        crate::kinfo!("[Task] new_user: chamando create_base()...");
        let base = Self::create_base();
        crate::kinfo!("[Task] new_user: create_base() OK, fazendo Box::pin...");
        let mut task = Box::pin(base);
        crate::kinfo!("[Task] new_user: Box::pin OK");

        // SAFETY: Task pinada, só mutamos campos internos
        unsafe {
            crate::kinfo!("[Task] new_user: get_unchecked_mut...");
            let t = task.as_mut().get_unchecked_mut();
            crate::kinfo!("[Task] new_user: t={:p}", t);
            t.cr3 = cr3;
            crate::kinfo!("[Task] new_user: cr3 OK, chamando setup_stack...");
            t.setup_stack(entry_point, USER_CODE_SEL, USER_DATA_SEL, user_stack_top);
            crate::kinfo!("[Task] new_user: setup_stack OK");
        }
        crate::kinfo!(
            "[Task] Processo de usuário criado: PID {}",
            task.id.as_u64()
        );
        task
    }

    /// Aloca estrutura base e stack alinhada.
    fn create_base() -> Self {
        const STACK_SIZE: usize = 32 * 1024; // 32KB

        crate::kinfo!(
            "[Task] create_base: alocando {} bytes para stack...",
            STACK_SIZE
        );

        // SEGURO: Vec::resize inicializa memória sem unsafe
        let mut kstack = Vec::with_capacity(STACK_SIZE);
        crate::kinfo!("[Task] create_base: Vec::with_capacity OK, resize...");
        kstack.resize(STACK_SIZE, 0u8);
        crate::kinfo!("[Task] create_base: resize OK, len={}", kstack.len());

        // Calcular topo da stack com alinhamento de 16 bytes (System V ABI)
        let stack_start = kstack.as_ptr() as u64;
        let stack_end = stack_start + STACK_SIZE as u64;
        let kstack_top = stack_end & !0xF;

        crate::kinfo!(
            "[Task] create_base: stack={:#x}-{:#x}, top={:#x}",
            stack_start,
            stack_end,
            kstack_top
        );

        crate::kinfo!("[Task] create_base: criando TaskId...");
        let id = TaskId::new();
        crate::kinfo!("[Task] create_base: TaskId={}", id.as_u64());

        crate::kinfo!("[Task] create_base: criando Context::empty...");
        let context = Context::empty();

        crate::kinfo!("[Task] create_base: criando HandleTable::empty...");
        let handles = HandleTable::empty();
        crate::kinfo!("[Task] create_base: HandleTable OK");

        crate::kinfo!("[Task] create_base: montando struct Task...");
        let task = Self {
            id,
            state: TaskState::Ready,
            context,
            kstack,
            kstack_top,
            cr3: 0,
            handles,
        };
        crate::kinfo!("[Task] create_base: struct Task OK");
        task
    }

    /// Prepara a stack para o primeiro Context Switch.
    /// Constrói um stack frame artificial que simula uma tarefa interrompida.
    fn setup_stack(&mut self, rip: u64, cs: u16, ss: u16, user_rsp: u64) {
        // Bounds check: validar kstack_top está dentro da região válida
        let stack_start = self.kstack.as_ptr() as u64;
        let stack_end = stack_start + self.kstack.len() as u64;

        assert!(
            self.kstack_top >= stack_start && self.kstack_top <= stack_end,
            "kstack_top ({:#x}) fora dos limites [{:#x} - {:#x}]",
            self.kstack_top,
            stack_start,
            stack_end
        );

        crate::kdebug!(
            "[Task] setup_stack: rip={:#x} cs={:#x} kstack_top={:#x}",
            rip,
            cs,
            self.kstack_top
        );

        unsafe {
            let mut ptr = self.kstack_top as *mut u64;

            // Macro para bounds check em cada operação
            macro_rules! stack_push {
                ($val:expr) => {{
                    ptr = ptr.sub(1);
                    // Validar que ainda estamos dentro da stack
                    assert!(
                        (ptr as u64) >= stack_start,
                        "Stack overflow em setup_stack: ptr={:p} < start={:#x}",
                        ptr,
                        stack_start
                    );
                    *ptr = $val;
                }};
            }

            // 1. Se for tarefa de usuário, empilhar frame IRETQ
            if cs == USER_CODE_SEL {
                // Layout: [SS, RSP, RFLAGS, CS, RIP]
                stack_push!(ss as u64);
                stack_push!(user_rsp);
                // RFLAGS: IF + IOPL=3
                stack_push!(0x3202);
                stack_push!(cs as u64);
                stack_push!(rip);

                // Endereço de retorno: Trampolim
                stack_push!(user_entry_trampoline as usize as u64);
                crate::ktrace!("[Task] IRETQ frame criado, trampoline={:#x}", *ptr);
            } else {
                // Tarefa de Kernel: Endereço de retorno direto
                stack_push!(rip);
            }

            // 2. Empilhar registradores Callee-Saved (RBX, RBP, R12-R15)
            stack_push!(0); // RBP
            stack_push!(0); // RBX
            stack_push!(0); // R12
            stack_push!(0); // R13
            stack_push!(0); // R14
            stack_push!(0); // R15

            // 3. Salvar o novo topo da stack
            self.kstack_top = ptr as u64;
            crate::kdebug!("[Task] setup_stack OK: kstack_top={:#x}", self.kstack_top);
        }
    }
}
