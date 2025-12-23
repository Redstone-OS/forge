//! Definição de Tarefa com Suporte a Userspace.

use crate::arch::x86_64::gdt::{KERNEL_CODE_SEL, KERNEL_DATA_SEL, USER_CODE_SEL, USER_DATA_SEL};
use crate::sched::context::Context;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// ID único de tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Ready,
    Blocked,
    Terminated,
}

pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub context: Context,

    // Stack do Kernel (usada durante syscalls/interrupts)
    pub kstack: Vec<u8>,
    pub kstack_top: u64,

    // Endereço Físico da PML4 (Page Table)
    // Se 0, usa a do kernel (para threads de kernel).
    pub cr3: u64,
}

impl Task {
    /// Cria tarefa de Kernel (Ring 0).
    pub fn new_kernel(entry: extern "C" fn()) -> Self {
        let mut task = Self::create_base();
        task.setup_stack(entry as u64, KERNEL_CODE_SEL, KERNEL_DATA_SEL, 0); // CS, SS, RFLAGS
        task
    }

    /// Cria tarefa de Usuário (Ring 3).
    /// entry_point: Endereço virtual no userspace.
    /// user_stack_top: Endereço virtual do topo da stack do usuário.
    /// cr3: Page Table física do processo.
    pub fn new_user(entry_point: u64, user_stack_top: u64, cr3: u64) -> Self {
        let mut task = Self::create_base();
        task.cr3 = cr3;

        // Configurar stack do kernel para "retornar" para userspace via IRETQ.
        // Stack Frame IRETQ: [SS, RSP, RFLAGS, CS, RIP]

        // Flags: Interrupts Enabled (0x200) | User IOPL (bônus)
        let rflags = 0x202;

        task.setup_stack(entry_point, USER_CODE_SEL, USER_DATA_SEL, user_stack_top);

        // Ajustar RFLAGS manualmente no contexto salvo se necessário,
        // mas setup_stack já deve lidar com a estrutura básica.
        task.context.rflags = rflags;

        task
    }

    fn create_base() -> Self {
        let stack_size = 32 * 1024; // 32KB Kernel Stack
        let mut kstack = Vec::with_capacity(stack_size);
        unsafe {
            kstack.set_len(stack_size);
        }
        let kstack_top = kstack.as_ptr() as u64 + stack_size as u64;

        Self {
            id: TaskId::new(),
            state: TaskState::Ready,
            context: Context::empty(),
            kstack,
            kstack_top,
            cr3: 0, // 0 = Kernel CR3 (não troca)
        }
    }

    fn setup_stack(&mut self, rip: u64, cs: u16, ss: u16, rsp: u64) {
        unsafe {
            let sp = self.kstack_top as *mut u64;

            // Simulando o frame de interrupção (IRETQ frame) se for userspace
            // Ou apenas RIP se for kernel space switch simples.
            // Para unificar, vamos usar o modelo "Switch salva Callee-saved".
            // Para ir para User, precisamos de um "Trampoline" ou o switch deve suportar.
            // SIMPLIFICAÇÃO: Vamos assumir que a primeira execução é via `context_switch`
            // para um wrapper que faz `iretq`.

            // Layout na Stack do Kernel para task nova:
            // [ ... ]
            // [ SS  ] (User Data) -> Só se for Ring 3
            // [ RSP ] (User Stack) -> Só se for Ring 3
            // [ RFLAGS ]
            // [ CS  ] (User Code)
            // [ RIP ] (Entry)
            // [ RBP, RBX, R12-R15 ] (Callee-saved)

            let mut ptr = sp;

            if cs == USER_CODE_SEL {
                ptr = ptr.sub(1);
                *ptr = ss as u64; // SS
                ptr = ptr.sub(1);
                *ptr = rsp; // RSP
                ptr = ptr.sub(1);
                *ptr = 0x202; // RFLAGS
                ptr = ptr.sub(1);
                *ptr = cs as u64; // CS
                ptr = ptr.sub(1);
                *ptr = rip; // RIP

                // O `switch.s` vai dar POP nos registradores e depois RET.
                // O RET vai pegar o RIP. Mas nós empilhamos um IRETQ frame acima!
                // Então precisamos de um "Trampoline" que faça o IRETQ.

                // Endereço de retorno do switch é o Trampoline
                ptr = ptr.sub(1);
                *ptr = crate::sched::user_entry_trampoline as u64;
            } else {
                // Kernel Task Direta
                ptr = ptr.sub(1);
                *ptr = rip;
            }

            // Callee-saved registers (zeroed)
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

            self.context.rsp = ptr as u64; // O `rsp` no Context é onde salvamos o topo
        }
    }
}
