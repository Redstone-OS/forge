//! Criação de processos

use crate::mm::VirtAddr;
use crate::sys::types::Pid;
use crate::sys::{KernelError, KernelResult};
use alloc::boxed::Box;
use core::pin::Pin;

/// Erro de execução
#[derive(Debug, Clone, Copy)]
pub enum ExecError {
    NotFound,
    InvalidFormat,
    OutOfMemory,
    PermissionDenied,
}

impl From<ExecError> for KernelError {
    fn from(e: ExecError) -> Self {
        match e {
            ExecError::NotFound => KernelError::NotFound,
            ExecError::InvalidFormat => KernelError::InvalidArgument,
            ExecError::OutOfMemory => KernelError::OutOfMemory,
            ExecError::PermissionDenied => KernelError::PermissionDenied,
        }
    }
}

/// Cria novo processo a partir de executável
pub fn spawn(path: &str) -> Result<Pid, ExecError> {
    crate::kinfo!("Spawning:", path.as_ptr() as u64);

    crate::ktrace!("(Spawn) [1] antes Task::new...");

    // 1. Abrir arquivo
    // TODO: usar VFS

    // 2. Carregar ELF
    // TODO: parsear headers, mapear segmentos

    // 3. Criar address space
    // TODO: criar page tables

    // 4. Criar task
    let mut task = crate::sched::task::Task::new(path);
    crate::ktrace!("(Spawn) [2] Task::new OK");

    // 5. Configurar entry point
    // TODO: pegar do ELF header
    let entry = VirtAddr::new(0x400000); // placeholder
    let stack = VirtAddr::new(0x7FFF_FFFF_0000); // placeholder
    task.context.setup(entry, stack);
    crate::ktrace!("(Spawn) [3] context.setup OK");

    // 6. Marcar como pronta
    task.set_ready();
    let pid = Pid::new(task.tid.as_u32());
    crate::ktrace!("(Spawn) [4] set_ready OK, pid=", pid.as_u32() as u64);

    // 7. Adicionar ao scheduler
    crate::ktrace!("(Spawn) [5] antes Box::pin...");
    crate::sched::scheduler::enqueue(Box::pin(task));
    crate::ktrace!("(Spawn) [6] enqueue OK");

    Ok(pid)
}
