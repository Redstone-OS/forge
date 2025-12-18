//! Gerenciamento de Processos
//!
//! Implementa processos básicos para multitasking cooperativo.

pub mod switch;

extern crate alloc;
use alloc::vec::Vec;
use spin::Mutex;

/// ID do processo
pub type Pid = u32;

/// Estado do processo
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Contexto salvo do processo
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcessContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

impl ProcessContext {
    pub const fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0x202,
        }
    }

    pub fn new_for_entry(entry: fn(), stack_top: u64) -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: stack_top,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: entry as u64,
            rflags: 0x202,
        }
    }
}

/// Processo
pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,
    pub context: ProcessContext,
    pub stack_bottom: u64,
    pub stack_size: usize,
    pub name: &'static str,
}

impl Process {
    pub fn new(pid: Pid, entry: fn(), stack: u64, stack_size: usize, name: &'static str) -> Self {
        let stack_top = stack + stack_size as u64;
        Self {
            pid,
            state: ProcessState::Ready,
            context: ProcessContext::new_for_entry(entry, stack_top),
            stack_bottom: stack,
            stack_size,
            name,
        }
    }
}

/// Gerenciador de Processos
pub struct ProcessManager {
    pub processes: Vec<Process>,
    pub next_pid: Pid,
    pub current_pid: Option<Pid>,
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            processes: Vec::new(),
            next_pid: 1,
            current_pid: None,
        }
    }

    pub fn spawn(&mut self, entry: fn(), name: &'static str) -> Pid {
        let pid = self.next_pid;
        self.next_pid += 1;

        let stack_size = 4096;
        let stack = unsafe {
            let layout = core::alloc::Layout::from_size_align_unchecked(stack_size, 16);
            alloc::alloc::alloc(layout) as u64
        };

        let process = Process::new(pid, entry, stack, stack_size, name);
        self.processes.push(process);
        pid
    }

    pub fn current(&self) -> Option<&Process> {
        self.current_pid
            .and_then(|pid| self.processes.iter().find(|p| p.pid == pid))
    }

    pub fn current_mut(&mut self) -> Option<&mut Process> {
        self.current_pid
            .and_then(|pid| self.processes.iter_mut().find(|p| p.pid == pid))
    }

    pub fn get_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }

    pub fn next_ready(&mut self) -> Option<&mut Process> {
        self.processes
            .iter_mut()
            .find(|p| p.state == ProcessState::Ready)
    }

    pub fn remove(&mut self, pid: Pid) {
        self.processes.retain(|p| p.pid != pid);
    }
}

impl ProcessManager {
    // ============================================================================
    // TODO(prioridade=CRÍTICA, versão=v1.0): REFATORAR CONTEXT SWITCH
    // ============================================================================
    //
    // ⚠️ ATENÇÃO: Este método usa UNSAFE para contornar o borrow checker!
    //
    // PROBLEMA: O borrow checker do Rust não permite múltiplos borrows mutáveis
    // do ProcessManager, o que impede a implementação safe de context switch.
    //
    // SOLUÇÃO ATUAL: Usar ponteiros brutos (unsafe) para obter referências
    // simultâneas aos contextos de dois processos diferentes.
    //
    // RISCOS:
    // - Ponteiros podem ser inválidos se ProcessManager for modificado
    // - Sem garantias de lifetime do Rust
    // - Possível corrupção de memória se usado incorretamente
    //
    // SOLUÇÕES FUTURAS:
    // 1. Refatorar ProcessManager para usar array fixo ao invés de Vec
    // 2. Usar índices ao invés de PIDs para acesso direto
    // 3. Separar contextos em estrutura própria
    // 4. Usar Cell/RefCell para interior mutability
    //
    // REFERÊNCIAS:
    // - https://doc.rust-lang.org/nomicon/
    // - https://os.phil-opp.com/async-await/#save-and-restore-context
    // ============================================================================

    /// Faz context switch entre dois processos
    ///
    /// # Safety
    ///
    /// Esta função é unsafe porque:
    /// - Usa ponteiros brutos para evitar borrow checker
    /// - Assume que from_pid e to_pid são válidos
    /// - Assume que processos não serão removidos durante o switch
    ///
    /// # Arguments
    ///
    /// * `from_pid` - PID do processo atual (salvar contexto)
    /// * `to_pid` - PID do próximo processo (carregar contexto)
    pub unsafe fn switch_context(&mut self, from_pid: Pid, to_pid: Pid) {
        // Obter ponteiros brutos para os contextos
        let from_ptr = self
            .processes
            .iter_mut()
            .find(|p| p.pid == from_pid)
            .map(|p| &mut p.context as *mut ProcessContext);

        let to_ptr = self
            .processes
            .iter()
            .find(|p| p.pid == to_pid)
            .map(|p| &p.context as *const ProcessContext);

        // Se ambos os processos existem, fazer switch
        if let (Some(from), Some(to)) = (from_ptr, to_ptr) {
            // Chamar assembly de context switch
            switch::switch_context(&mut *from, &*to);
        }
    }
}

pub static PROCESS_MANAGER: Mutex<ProcessManager> = Mutex::new(ProcessManager::new());
