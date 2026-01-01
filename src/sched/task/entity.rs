//! Thread Control Block

use super::accounting::Accounting;
use super::context::CpuContext;
use super::state::TaskState;
use crate::mm::VirtAddr;
use crate::sys::types::Tid;
use crate::syscall::handle::table::HandleTable;

/// Task ID counter
static NEXT_TID: crate::sync::AtomicCounter = crate::sync::AtomicCounter::new(1);

/// Thread Control Block
pub struct Task {
    /// ID único
    pub tid: Tid,
    /// Estado atual
    pub state: TaskState,
    /// Contexto de CPU salvo
    pub context: CpuContext,
    /// Stack pointer do kernel
    pub kernel_stack: VirtAddr,
    /// Stack pointer do usuário
    pub user_stack: VirtAddr,
    /// CR3 (Physical Address da PML4)
    pub cr3: u64,
    /// Prioridade (0 = maior)
    pub priority: u8,
    /// Estatísticas de contabilidade
    pub accounting: Accounting,

    // --- Hierarquia ---
    /// ID da tarefa pai (quem criou esta)
    pub parent_id: Option<Tid>,
    /// Código de saída (para waitpid)
    pub exit_code: Option<i32>,

    // --- Sinais ---
    /// Sinais pendentes (bitmap 64-bit)
    pub pending_signals: u64,
    /// Sinais bloqueados (máscara)
    pub blocked_signals: u64,

    /// Nome (debug)
    pub name: [u8; 32],
    /// Tabela de handles
    pub handle_table: HandleTable,
    /// Momento de acordar (jiffies) se estiver dormindo
    pub wake_at: Option<u64>,
    /// Base da heap do usuário
    pub heap_start: u64,
    /// Próximo endereço livre da heap
    pub heap_next: u64,
}

impl Task {
    /// Cria nova task diretamente no heap
    pub fn new(name: &str) -> Self {
        // Criar TID
        let tid = Tid::new(NEXT_TID.inc() as u32);

        // Preparar buffer de nome
        let mut name_buf = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(31);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            tid,
            state: TaskState::Created,
            context: CpuContext::new(),
            kernel_stack: VirtAddr::new(0),
            user_stack: VirtAddr::new(0),
            cr3: 0,
            priority: super::super::config::PRIORITY_DEFAULT,
            accounting: Accounting::new(),
            parent_id: None,
            exit_code: None,
            pending_signals: 0,
            blocked_signals: 0,
            name: name_buf,
            handle_table: HandleTable::new(),
            wake_at: None,
            heap_start: 0x10000000,
            heap_next: 0x10000000,
        }
    }

    /// Marca como pronta
    pub fn set_ready(&mut self) {
        self.state = TaskState::Ready;
    }

    /// Marca como bloqueada
    pub fn set_blocked(&mut self) {
        self.state = TaskState::Blocked;
    }

    /// Aplica o estado de hardware da task (GDT, CR3) na CPU atual.
    ///
    /// # Safety
    /// Deve ser chamado com interrupções desabilitadas e lock do scheduler.
    pub unsafe fn apply_hardware_state(&self) {
        let kernel_stack = self.kernel_stack.as_u64();

        // 1. Configurar stack do kernel para interrupções/syscalls
        if kernel_stack != 0 {
            crate::arch::x86_64::gdt::set_kernel_stack(kernel_stack);
            crate::arch::x86_64::syscall::set_kernel_rsp(kernel_stack);
        }

        // 2. Trocar espaço de endereçamento (CR3)
        if self.cr3 != 0 {
            crate::arch::Cpu::write_cr3(self.cr3);
        }
    }
}
