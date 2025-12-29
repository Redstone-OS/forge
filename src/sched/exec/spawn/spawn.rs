//! Criação de processos

use crate::mm::pmm::{FRAME_ALLOCATOR, FRAME_SIZE};
use crate::mm::vmm::{map_page_with_pmm, MapFlags};
use crate::mm::VirtAddr;
use crate::sys::types::Pid;
use crate::sys::KernelError;
use alloc::boxed::Box;

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

/// Topo da stack do userspace (final da metade inferior canônica)
const USER_STACK_TOP: u64 = 0x7FFF_FFFF_F000;
/// Tamanho da stack (32KB)
const USER_STACK_SIZE: u64 = 8 * 4096;

/// Cria novo processo a partir de executável
pub fn spawn(path: &str) -> Result<Pid, ExecError> {
    crate::kinfo!("Spawning:", path.as_ptr() as u64);

    // 1. Carregar arquivo do Initramfs (via lookup direto temporário)
    let data = match crate::fs::initramfs::lookup_file(path) {
        Some(d) => d,
        None => {
            crate::kerror!("(Spawn) Arquivo não encontrado:", path.as_ptr() as u64);
            return Err(ExecError::NotFound);
        }
    };

    // 2. Carregar ELF (Mapear segmentos de código/dados)
    crate::kinfo!("(Spawn) Chamando elf::load_binary...");
    let entry_point = match crate::sched::exec::elf::load_binary(data) {
        Ok(addr) => {
            crate::kinfo!("(Spawn) elf::load_binary OK. Addr:", addr.as_u64());
            addr
        }
        Err(_) => {
            crate::kerror!("(Spawn) elf::load_binary FALHOU");
            return Err(ExecError::InvalidFormat);
        }
    };

    // 3. Criar task
    crate::kinfo!("(Spawn) Creating task struct...");
    let mut task = crate::sched::task::Task::new(path);
    crate::kinfo!("(Spawn) Task created via Task::new");

    // 4. Alocar e Mapear Stack de Usuário
    {
        let mut pmm = FRAME_ALLOCATOR.lock();
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE | MapFlags::USER;

        let start_page = USER_STACK_TOP - USER_STACK_SIZE;
        let pages = USER_STACK_SIZE / FRAME_SIZE;

        for i in 0..pages {
            let vaddr = start_page + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                // TODO: Mapear no address space do processo (atualmente no kernel)
                unsafe {
                    if let Err(_e) = map_page_with_pmm(vaddr, frame.as_u64(), flags, &mut *pmm) {
                        return Err(ExecError::OutOfMemory);
                    }
                    // Zerar stack
                    let ptr = vaddr as *mut u8;
                    core::ptr::write_bytes(ptr, 0, FRAME_SIZE as usize);
                }
            } else {
                return Err(ExecError::OutOfMemory);
            }
        }
    }

    // 5. Configurar contexto (Entry Point + Stack Pointer)
    let stack = VirtAddr::new(USER_STACK_TOP);
    task.context.setup(entry_point, stack);

    // 6. Marcar como pronta
    task.set_ready();
    let pid = Pid::new(task.tid.as_u32());

    // 7. Adicionar ao scheduler
    crate::sched::scheduler::enqueue(Box::pin(task));
    crate::kinfo!("Process spawned from ELF! PID:", pid.as_u32() as u64);

    Ok(pid)
}
