//! Criação de processos

use crate::mm::pmm::{FRAME_ALLOCATOR, FRAME_SIZE};
use crate::mm::vmm::MapFlags;
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

    // 1. Carregar arquivo via VFS (roteia para initramfs ou FAT)
    let data = match crate::fs::vfs::read_file(path) {
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

    // 6. Carregar ELF (agora registra VMAs no aspace e mapeia via HHDM)
    let entry_point = match crate::sched::exec::fmt::elf::load_binary(&data, &aspace) {
        Ok(addr) => addr,
        Err(_) => {
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

    let target_cr3 = aspace.lock().cr3();

    // Alocar frames para a User Stack (via HHDM no alvo)
    {
        let mut pmm = FRAME_ALLOCATOR.lock();
        for i in 0..(ustack_size as u64 / FRAME_SIZE) {
            let vaddr = ustack_start + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                unsafe {
                    crate::mm::vmm::mapper::map_page_in_target_p4(
                        target_cr3,
                        vaddr,
                        frame.as_u64(),
                        MapFlags::PRESENT | MapFlags::WRITABLE | MapFlags::USER,
                        &mut *pmm,
                    )
                    .expect("(Spawn) Falha ao mapear User Stack");

                    // Zerar página no alvo via HHDM
                    core::ptr::write_bytes(
                        crate::mm::addr::phys_to_virt::<u8>(frame.as_u64()),
                        0,
                        FRAME_SIZE as usize,
                    );
                }
            }
        }
    }
    task.user_stack = VirtAddr::new(USER_STACK_TOP);
    // 8. Configurar Trap Frame na stack do kernel do ALVO via HHDM
    unsafe {
        const USER_CODE_SEL: u64 = 0x23; // Index 4, RPL 3
        const USER_DATA_SEL: u64 = 0x1B; // Index 3, RPL 3
        const RFLAGS_IF: u64 = 0x202; // IF=1, reserva=1
        use crate::arch::x86_64::interrupts::ExceptionStackFrame;
        const SWITCH_RESERVE: u64 = 8; // Slot que context_switch consome

        // Precisamos achar o endereço FÍSICO do topo da stack de kernel do alvo
        if let Some(phys_top) =
            crate::mm::vmm::mapper::translate_addr_in_p4(target_cr3, kstack_top - 8)
        {
            // Ajustar phys_top para o endereço real do topo (translate_addr_in_p4 retorna o byte físico)
            let phys_page = phys_top & !0xFFF;
            let offset = (kstack_top - 8) % FRAME_SIZE;

            // Frame address via HHDM
            let stack_top_hhdm =
                crate::mm::addr::phys_to_virt::<u8>(phys_page).add(offset as usize + 8);

            let frame_ptr = (stack_top_hhdm as u64
                - core::mem::size_of::<ExceptionStackFrame>() as u64
                - SWITCH_RESERVE) as *mut ExceptionStackFrame;

            (*frame_ptr).instruction_pointer = entry_point.as_u64();
            (*frame_ptr).code_segment = USER_CODE_SEL;
            (*frame_ptr).cpu_flags = RFLAGS_IF;
            (*frame_ptr).stack_pointer = USER_STACK_TOP;
            (*frame_ptr).stack_segment = USER_DATA_SEL;

            let trampoline = crate::sched::core::entry::user_entry_stub as u64;

            // task.context.rsp deve ser o valor que o registrador RSP terá ANTES do salto.
            // Após o salto (via jump_to_context_asm ou ret), o RSP estará em kstack_top.
            // O user_entry_stub então descerá 48 bytes para apontar ao TrapFrame.
            task.context.rsp = kstack_top - SWITCH_RESERVE;
            task.context.rip = trampoline;
        } else {
            panic!("(Spawn) Erro fatal: Stack de kernel do alvo não mapeada na P4!");
        }
    }

    // 10. Enfileirar Task
    task.set_ready();
    crate::sched::core::enqueue(alloc::boxed::Box::pin(task));

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
