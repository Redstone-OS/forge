//! Criação de processos

use crate::sys::{KernelError, KernelResult};
use crate::sys::types::Pid;
use crate::mm::VirtAddr;
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
    
    // 1. Abrir arquivo
    // TODO: usar VFS
    
    // 2. Carregar ELF
    // TODO: parsear headers, mapear segmentos
    
    // 3. Criar address space
    // TODO: criar page tables
    
    // 4. Criar task
    let mut task = crate::sched::task::Task::new(path);
    
    // 5. Configurar entry point
    // TODO: pegar do ELF header
    let entry = VirtAddr::new(0x400000); // placeholder
    let stack = VirtAddr::new(0x7FFF_FFFF_0000); // placeholder
    task.context.setup(entry, stack);
    
    // 6. Marcar como pronta
    task.set_ready();
    let pid = Pid::new(task.tid.as_u32());
    
    // 7. Adicionar ao scheduler
    crate::sched::scheduler::enqueue(Box::pin(task));
    
    Ok(pid)
}
