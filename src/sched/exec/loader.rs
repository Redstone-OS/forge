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

// Use constantes do config
use crate::sched::config::USER_STACK_SIZE;

/// Topo da stack do userspace (final da metade inferior canônica)
const USER_STACK_TOP: u64 = 0x7FFF_FFFF_F000;

/// Cria novo processo a partir de executável
pub fn spawn(path: &str, parent_id: Option<crate::sys::types::Tid>) -> Result<Pid, ExecError> {
    crate::kinfo!("(Spawn) Spawning:", path.as_ptr() as u64);

    // 1. Carregar arquivo do Initramfs
    let data = match crate::fs::initramfs::lookup_file(path) {
        Some(d) => d,
        None => {
            crate::kerror!("(Spawn) Arquivo não encontrado:", path.as_ptr() as u64);
            return Err(ExecError::NotFound);
        }
    };

    // 2. Criar task
    let mut task = crate::sched::task::Task::new(path);
    task.parent_id = parent_id;
    let pid = Pid::new(task.tid.as_u32());
    let pid_u64 = pid.as_u32() as u64;

    // 3. Criar AddressSpace isolado
    use crate::mm::aspace::vma::{MemoryIntent, Protection, VmaFlags};
    use crate::mm::aspace::AddressSpace;
    use crate::sync::Spinlock;
    use alloc::sync::Arc;

    let aspace = Arc::new(Spinlock::new(
        AddressSpace::new(pid_u64).map_err(|_| ExecError::OutOfMemory)?,
    ));
    task.aspace = Some(aspace.clone());

    // 4. Mapear Stack do Kernel (Espaço do Kernel - compartilhado mas visível na P4 do processo)
    const KERNEL_STACK_BASE: u64 = 0xFFFF_9100_0000_0000;
    let kstack_size = crate::sched::config::KERNEL_STACK_SIZE as u64;
    let kstack_start = KERNEL_STACK_BASE + (pid_u64 * kstack_size);
    let kstack_top = kstack_start + kstack_size;

    {
        let mut pmm = FRAME_ALLOCATOR.lock();
        let pages = kstack_size / FRAME_SIZE;
        for i in 0..pages {
            let vaddr = kstack_start + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                unsafe {
                    // Mapeia no P4 do processo (estamos usando a P4 do aspace)
                    crate::mm::vmm::map_page_in_target_p4(
                        aspace.lock().cr3(),
                        vaddr,
                        frame.as_u64(),
                        MapFlags::PRESENT | MapFlags::WRITABLE,
                        &mut *pmm,
                    )
                    .expect("(Spawn) Falha ao mapear KStack");

                    // Zerar stack via HHDM (seguro com qualquer CR3)
                    core::ptr::write_bytes(
                        crate::mm::hhdm::phys_to_virt::<u8>(frame.as_u64()),
                        0,
                        FRAME_SIZE as usize,
                    );
                }
            }
        }
    }
    task.kernel_stack = VirtAddr::new(kstack_top);

    // 5. Ativar AddressSpace para carregar ELF e configurar User Space
    let old_cr3 = crate::mm::vmm::mapper::read_cr3();
    unsafe {
        aspace.lock().activate();
    }

    // 6. Carregar ELF (agora registra VMAs no aspace)
    let entry_point = match crate::sched::exec::fmt::elf::load_binary(data, &aspace) {
        Ok(addr) => addr,
        Err(_) => {
            unsafe {
                crate::mm::vmm::mapper::write_cr3(old_cr3);
            }
            return Err(ExecError::InvalidFormat);
        }
    };

    // 7. Configurar Stack de Usuário via VMA
    let ustack_size = USER_STACK_SIZE as usize;
    let ustack_start = USER_STACK_TOP - ustack_size as u64;

    {
        let mut as_lock = aspace.lock();
        as_lock
            .map_region(
                Some(VirtAddr::new(ustack_start)),
                ustack_size,
                Protection::RW,
                VmaFlags::GROWS_DOWN,
                MemoryIntent::Stack,
            )
            .expect("(Spawn) Falha ao registrar User Stack VMA");
    }

    // Alocar frames para a User Stack (Imediato por enquanto)
    {
        let mut pmm = FRAME_ALLOCATOR.lock();
        for i in 0..(ustack_size as u64 / FRAME_SIZE) {
            let vaddr = ustack_start + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                unsafe {
                    map_page_with_pmm(
                        vaddr,
                        frame.as_u64(),
                        MapFlags::PRESENT | MapFlags::WRITABLE | MapFlags::USER,
                        &mut *pmm,
                    )
                    .expect("(Spawn) Falha ao mapear User Stack");
                    core::ptr::write_bytes(vaddr as *mut u8, 0, FRAME_SIZE as usize);
                }
            }
        }
    }
    task.user_stack = VirtAddr::new(USER_STACK_TOP);

    // 8. Configurar Trap Frame
    unsafe {
        const USER_CODE_SEL: u64 = 0x23;
        const USER_DATA_SEL: u64 = 0x1B;
        const RFLAGS_IF: u64 = 0x202;
        use crate::arch::x86_64::interrupts::ExceptionStackFrame;
        const SWITCH_RESERVE: u64 = 8;

        let frame_ptr =
            (kstack_top - SWITCH_RESERVE - core::mem::size_of::<ExceptionStackFrame>() as u64)
                as *mut ExceptionStackFrame;
        (*frame_ptr).instruction_pointer = entry_point.as_u64();
        (*frame_ptr).code_segment = USER_CODE_SEL;
        (*frame_ptr).cpu_flags = RFLAGS_IF;
        (*frame_ptr).stack_pointer = USER_STACK_TOP;
        (*frame_ptr).stack_segment = USER_DATA_SEL;

        let trampoline = crate::sched::core::entry::user_entry_stub as u64;
        task.context.rsp =
            frame_ptr as u64 + core::mem::size_of::<ExceptionStackFrame>() as u64 + SWITCH_RESERVE
                - 8;
        task.context.rip = trampoline;
    }

    // 9. Restaurar CR3
    unsafe {
        crate::mm::vmm::mapper::write_cr3(old_cr3);
    }

    // 10. Enfileirar Task
    task.set_ready();
    crate::sched::core::enqueue(Box::pin(task));

    crate::kinfo!("Process spawned successfully! PID:", pid.as_u32() as u64);
    Ok(pid)
}

/// Função de teste para validar troca de contexto
#[no_mangle]
extern "C" fn test_kernel_task() {
    crate::kinfo!("!!! OLÁ DA TAREFA DO KERNEL !!!");
    crate::kinfo!("A troca de contexto funcionou corretamente.");
    loop {
        crate::arch::Cpu::halt();
    }
}
