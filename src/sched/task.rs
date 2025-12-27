//! # Task / Process Control Block (PCB)
//!
//! Este módulo define a unidade atômica de escalonamento do Redstone OS: a `Task`.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **PCB (Process Control Block):** Mantém o estado completo de execução (Contexto, Stack, CR3).
//! - **Kernel Stack Ownership:** Cada tarefa possui sua própria pilha de kernel de 32KB.
//! - **Resource Holding:** Detém a `HandleTable` (permissões/capabilities) e o espaço de endereçamento (CR3).
//!
//! ## 🏗️ Arquitetura: Pinned Task
//! Devido à natureza sensível da stack de kernel, as tarefas são criadas como `PinnedTask` (`Pin<Box<Task>>`).
//! - **Por que Pin?** O `context_switch` armazena o endereço do topo da stack (`kstack_top`) dentro da própria estrutura `Task`.
//!   Se a `Task` fosse movida na memória (ex: `realloc` de um `Vec<Task>`), o ponteiro `current_rsp` salvo apontaria para lixo.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Stack Isolada:** O uso de `Vec<u8>` para a kstack garante que cada tarefa tenha memória contígua e segura (exceto por overflows).
//! - **Capability-Based:** A inclusão de `HandleTable` no núcleo do PCB reforça o modelo de segurança zero-trust.
//! - **ID Atômico:** `TaskId` monotonicamente crescente com `AtomicU64` previne colisão de PIDs.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Heap Allocation:** `Task` e `kstack` são alocados no Heap (`Vec`). Isso gera:
//!   1. Fragmentação.
//!   2. Dependência de alocador complexo em caminhos críticos (spawn).
//!   3. Risco de OOM imprevisível.
//! - **Hardcoded Stack Size:** 32KB é fixo. Drivers complexos ou recursão podem causar **Stack Overflow** silencioso (corrupção de heap),
//!   poís não há "Guard Pages".
//! - **Lack of Hierarchy:** Não existe conceito de "Task Pai" ou "Task Filho". `waitpid` é impossível hoje.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical/Security)** Implementar **Guard Pages** na base da stack.
//!   - *Como:* Deixar uma página não-mapeada (zero permissão) antes da stack. Se estourar, gera Page Fault (bom) em vez de corromper o vizinho (catastrófico).
//! - [ ] **TODO: (Performance)** Migrar alocação de stacks para **PMM Direct** (evitar Heap).
//!   - *Ganho:* Stacks são sempre múltiplos de página (4KB). Alocar direto do PMM é mais rápido e reduz pressão no Heap.
//! - [ ] **TODO: (Feature)** Adicionar `parent_id` e lista de `children` para suportar árvores de processos.
//!

use crate::arch::x86_64::gdt::{KERNEL_CODE_SEL, KERNEL_DATA_SEL, USER_CODE_SEL, USER_DATA_SEL};
use crate::core::handle::HandleTable;
use crate::drivers::serial;
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
        crate::kdebug!("[Task] new_kernel OK: entrada=", entry as usize);
        task
    }

    /// Cria uma nova tarefa de Usuário (Ring 3).
    ///
    /// # Arguments
    /// * `entry_point`: Endereço virtual (RIP) no userspace.
    /// * `user_stack_top`: Endereço virtual (RSP) da stack no userspace.
    /// * `cr3`: Endereço físico da tabela de páginas do processo.
    pub fn new_user(entry_point: u64, user_stack_top: u64, cr3: u64) -> PinnedTask {
        #[cfg(feature = "log_trace")]
        {
            crate::klog!(
                "[TRAC] (Task) new_user: entrada=",
                entry_point,
                " pilha=",
                user_stack_top
            );
            crate::klog!(" cr3=", cr3);
            crate::knl!();
        }

        let mut task = Box::pin(Self::create_base());

        // SAFETY: Task pinada, só mutamos campos internos
        unsafe {
            let t = task.as_mut().get_unchecked_mut();
            t.cr3 = cr3;
            t.setup_stack(entry_point, USER_CODE_SEL, USER_DATA_SEL, user_stack_top);
        }
        crate::kinfo!("(Task) Processo de usuário criado: PID=", task.id.as_u64());
        task
    }

    /// Aloca estrutura base e stack alinhada.
    fn create_base() -> Self {
        const STACK_SIZE: usize = 32 * 1024; // 32KB

        #[cfg(feature = "log_trace")]
        {
            serial::emit_str("[TRAC] (Task) create_base: Alocando ");
            serial::emit_dec(STACK_SIZE);
            serial::emit_str(" bytes para pilha...\n\r");
        }

        // SEGURO: Vec::resize inicializa memória sem unsafe
        let mut kstack = Vec::with_capacity(STACK_SIZE);
        kstack.resize(STACK_SIZE, 0u8);

        // Calcular topo da stack com alinhamento de 16 bytes (System V ABI)
        let stack_start = kstack.as_ptr() as u64;
        let stack_end = stack_start + STACK_SIZE as u64;
        let kstack_top = stack_end & !0xF;

        let id = TaskId::new();
        let context = Context::empty();
        let handles = HandleTable::empty();

        Self {
            id,
            state: TaskState::Ready,
            context,
            kstack,
            kstack_top,
            cr3: 0,
            handles,
        }
    }

    /// Prepara a stack para o primeiro Context Switch.
    /// Constrói um stack frame artificial que simula uma tarefa interrompida.
    fn setup_stack(&mut self, rip: u64, cs: u16, ss: u16, user_rsp: u64) {
        let stack_start = self.kstack.as_ptr() as u64;
        let stack_end = stack_start + self.kstack.len() as u64;

        if !(self.kstack_top >= stack_start && self.kstack_top <= stack_end) {
            serial::emit_str("[ERRO] kstack_top=");
            serial::emit_hex(self.kstack_top);
            serial::emit_str(" fora dos limites do kernel!\n\r");
            panic!("Task kstack_top out of bounds");
        }

        #[cfg(any(feature = "log_debug", feature = "log_trace"))]
        {
            crate::klog!(
                "[DEBG] [Task] setup_stack: rip=",
                rip,
                " kstack_top=",
                self.kstack_top
            );
            crate::knl!();
        }

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
                crate::ktrace!("[Task] Frame IRETQ criado, trampolim=", *ptr);
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
            crate::kdebug!("[Task] setup_stack OK: novo kstack_top=", self.kstack_top);
        }
    }
}
