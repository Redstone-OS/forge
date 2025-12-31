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
}

impl Task {
    /// Cria nova task diretamente no heap
    ///
    /// Evita cópia de struct grande através do stack
    #[inline(never)]
    pub fn new(name: &str) -> Self {
        crate::ktrace!("(Task) new entrada");

        // Criar TID primeiro
        let tid_val = NEXT_TID.inc() as u32;
        crate::ktrace!("(Task) tid_val=", tid_val as u64);
        let tid = Tid::new(tid_val);

        // Buffer de nome zerado
        let mut name_buf = [0u8; 32];

        // Copiar bytes do nome manualmente usando ponteiros raw
        let name_bytes = name.as_bytes();
        let len = if name_bytes.len() < 31 {
            name_bytes.len()
        } else {
            31
        };

        crate::ktrace!("(Task) copiando nome, len=", len as u64);

        // Cópia ultra segura byte a byte via ponteiro
        let src = name_bytes.as_ptr();
        let dst = name_buf.as_mut_ptr();
        let mut i = 0usize;
        while i < len {
            unsafe {
                // Ler byte da fonte
                let byte = core::ptr::read_volatile(src.add(i));
                // Escrever byte no destino
                core::ptr::write_volatile(dst.add(i), byte);
            }
            i = i.wrapping_add(1);
        }

        crate::ktrace!("(Task) nome copiado OK");
        crate::ktrace!("(Task) construindo struct...");

        // Construir struct manualmente
        let task = Self {
            tid,
            state: TaskState::Created,
            context: CpuContext::new(),
            kernel_stack: VirtAddr::new(0),
            user_stack: VirtAddr::new(0),
            cr3: 0, // Será definido no spawn
            priority: 128,
            accounting: Accounting::new(),
            parent_id: None, // Define no spawn
            exit_code: None,
            pending_signals: 0,
            blocked_signals: 0,
            name: name_buf,
            handle_table: HandleTable::new(),
        };

        crate::ktrace!("(Task) struct construída, retornando...");
        task
    }

    /// Marca como pronta
    pub fn set_ready(&mut self) {
        self.state = TaskState::Ready;
    }

    /// Marca como bloqueada
    pub fn set_blocked(&mut self) {
        self.state = TaskState::Blocked;
    }
}
