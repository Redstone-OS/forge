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
pub fn spawn(path: &str) -> Result<Pid, ExecError> {
    crate::kinfo!("(Spawn) Spawning:", path.as_ptr() as u64);

    // 1. Carregar arquivo do Initramfs (via lookup direto temporário)
    let data = match crate::fs::initramfs::lookup_file(path) {
        Some(d) => d,
        None => {
            crate::kerror!("(Spawn) Arquivo não encontrado:", path.as_ptr() as u64);
            return Err(ExecError::NotFound);
        }
    };

    // 3. Criar task
    crate::kinfo!("(Spawn) Creating task struct...");
    let mut task = crate::sched::task::Task::new(path);
    crate::kinfo!("(Spawn) Task created via Task::new");

    // === PROCESS ISOLATION SETUP ===

    // 1. Criar nova Page Table isolada PRIMEIRO
    // Isso copia Kernel Half + Identity Map do kernel P4
    let new_p4 = {
        let mut pmm = FRAME_ALLOCATOR.lock();
        crate::mm::vmm::mapper::create_new_p4(&mut *pmm).expect("(Spawn) Falha ao criar P4")
    };
    task.cr3 = new_p4;
    crate::kinfo!("(Spawn) Nova PML4 criada:", new_p4);

    // 2. Alocar e Mapear Kernel Stack em AMBOS os P4s
    // Isso garante que o TrapFrame seja visível em ambos os CR3s
    let pid = Pid::new(task.tid.as_u32());
    const KERNEL_STACK_BASE: u64 = 0xFFFF_9100_0000_0000;
    const KERNEL_STACK_SIZE: u64 = 8192; // 2 pages

    let pid_u64 = pid.as_u32() as u64;
    let kstack_start = KERNEL_STACK_BASE + (pid_u64 * KERNEL_STACK_SIZE);
    let kstack_top = kstack_start + KERNEL_STACK_SIZE;

    // Alocar frames e mapear em AMBOS os P4s (kernel e processo)
    {
        crate::kinfo!("(Spawn) Allocating KStack frames for PID:", pid_u64);
        let mut pmm = FRAME_ALLOCATOR.lock();
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE; // Kernel acessa
        let pages = KERNEL_STACK_SIZE / FRAME_SIZE;

        for i in 0..pages {
            let vaddr = kstack_start + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                let frame_phys = frame.as_u64();
                unsafe {
                    // Mapear no Kernel P4 atual
                    if let Err(_e) = map_page_with_pmm(vaddr, frame_phys, flags, &mut *pmm) {
                        return Err(ExecError::OutOfMemory);
                    }
                    // Mapear no P4 do processo (MESMO frame físico!)
                    if let Err(_e) = crate::mm::vmm::map_page_in_target_p4(
                        new_p4, vaddr, frame_phys, flags, &mut *pmm,
                    ) {
                        return Err(ExecError::OutOfMemory);
                    }
                    // Zerar stack (pode ser feito com qualquer CR3 pois está mapeado em ambos)
                    let ptr = vaddr as *mut u8;
                    for j in 0..FRAME_SIZE as usize {
                        ptr.add(j).write_volatile(0);
                    }
                }
            } else {
                return Err(ExecError::OutOfMemory);
            }
        }
    }
    task.kernel_stack = VirtAddr::new(kstack_top);

    // 2. Trocar para nova P4 temporariamente para carregar ELF e configurar User Space
    let old_cr3 = crate::mm::vmm::mapper::read_cr3();
    unsafe {
        crate::mm::vmm::mapper::write_cr3(new_p4);
    }
    crate::kinfo!("(Spawn) CR3 trocado para nova P4 (contexto temporário)");

    // 3. Carregar ELF (agora mapeia na nova P4)
    crate::kinfo!("(Spawn) Chamando elf::load_binary...");
    let entry_point = match crate::sched::exec::fmt::elf::load_binary(data) {
        Ok(addr) => {
            crate::kinfo!("(Spawn) elf::load_binary OK. Addr:", addr.as_u64());
            addr
        }
        Err(_) => {
            unsafe {
                crate::mm::vmm::mapper::write_cr3(old_cr3);
            }
            crate::kerror!("(Spawn) elf::load_binary FALHOU");
            return Err(ExecError::InvalidFormat);
        }
    };

    // 4. Conceder Permissões de Usuário
    crate::mm::vmm::mapper::grant_user_access(entry_point.as_u64());
    crate::mm::vmm::mapper::grant_user_access(0x400000);

    // 5. Alocar Stack de Usuário (na nova P4)
    task.user_stack = VirtAddr::new(USER_STACK_TOP);
    crate::kinfo!("(Spawn) Allocating user stack...");
    {
        crate::kinfo!("(Spawn) Locking PMM...");
        let mut pmm = FRAME_ALLOCATOR.lock();
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE | MapFlags::USER;
        let start_page = USER_STACK_TOP - USER_STACK_SIZE as u64;
        let pages = USER_STACK_SIZE as u64 / FRAME_SIZE;

        for i in 0..pages {
            let vaddr = start_page + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                unsafe {
                    if let Err(_e) = map_page_with_pmm(vaddr, frame.as_u64(), flags, &mut *pmm) {
                        crate::mm::vmm::mapper::write_cr3(old_cr3);
                        return Err(ExecError::OutOfMemory);
                    }
                    let ptr = vaddr as *mut u8;
                    for j in 0..FRAME_SIZE as usize {
                        ptr.add(j).write_volatile(0);
                    }
                }
            } else {
                unsafe {
                    crate::mm::vmm::mapper::write_cr3(old_cr3);
                }
                return Err(ExecError::OutOfMemory);
            }
        }
    }

    // DEBUG: Validate entry point code
    unsafe {
        let entry_ptr = entry_point.as_u64() as *const u8;
        crate::ktrace!(
            "(Spawn) Validating Entry Point Code at:",
            entry_point.as_u64()
        );
        let b0 = core::ptr::read_volatile(entry_ptr);
        let b1 = core::ptr::read_volatile(entry_ptr.add(1));
        let b2 = core::ptr::read_volatile(entry_ptr.add(2));
        let b3 = core::ptr::read_volatile(entry_ptr.add(3));
        crate::ktrace!("(Spawn) Code Bytes:", b0 as u64);
        crate::ktrace!("(Spawn) Code Bytes:", b1 as u64);
        crate::ktrace!("(Spawn) Code Bytes:", b2 as u64);
        crate::ktrace!("(Spawn) Code Bytes:", b3 as u64);

        if b0 == 0 && b1 == 0 && b2 == 0 && b3 == 0 {
            crate::kerror!("(Spawn) CRITICAL: Entry point code is all ZEROS!");
        }
    }

    // 6. Configurar Trap Frame na Kernel Stack (ENQUANTO CR3 do processo está ativo!)
    // Hotfix: escrever o TrapFrame com o CR3 do processo ativo garante que
    // a escrita vá para a memória visível pelo processo.
    unsafe {
        let current_cr3 = crate::mm::vmm::mapper::read_cr3();
        crate::ktrace!("(Spawn) TrapFrame escrito com CR3=", current_cr3);
        crate::kinfo!("(Spawn) Building TrapFrame at:", kstack_top);
        let ptr = kstack_top as *mut u64;

        // Seletores (RPL 3)
        const USER_CODE_SEL: u64 = 0x23; // Index 4, RPL 3
        const USER_DATA_SEL: u64 = 0x1B; // Index 3, RPL 3
        const RFLAGS_IF: u64 = 0x202;

        use crate::arch::x86_64::interrupts::ExceptionStackFrame;

        // Calcular base do frame (topo - sizeof(ExceptionStackFrame))
        // ExceptionStackFrame tem 5 u64s = 40 bytes.
        let frame_ptr = (kstack_top - core::mem::size_of::<ExceptionStackFrame>() as u64)
            as *mut ExceptionStackFrame;

        // Escrever frame estruturado
        (*frame_ptr).instruction_pointer = entry_point.as_u64();
        (*frame_ptr).code_segment = USER_CODE_SEL;
        (*frame_ptr).cpu_flags = RFLAGS_IF;
        (*frame_ptr).stack_pointer = USER_STACK_TOP;
        (*frame_ptr).stack_segment = USER_DATA_SEL;

        // Alteração DEADLOCK FIX: Usar user_entry_stub em vez de ir direto para iretq_restore
        let trampoline = crate::sched::core::entry::user_entry_stub as u64;
        let context_rsp = frame_ptr as u64; // RSP aponta para o inicio do frame

        task.context.rsp = context_rsp;
        task.context.rip = trampoline;

        crate::ktrace!("(Spawn) TrapFrame base:", context_rsp);
        crate::ktrace!("(Spawn) Trampolim:", trampoline);
        crate::ktrace!("(Spawn) Entry point escrito:", entry_point.as_u64());

        // Verificar leitura de volta do TrapFrame
        // frame_ptr é *mut ExceptionStackFrame. Instruction Pointer é o primeiro campo (offset 0).
        let rip_check = core::ptr::read_volatile(&(*frame_ptr).instruction_pointer);
        crate::ktrace!("(Spawn) TrapFrame[RIP] lido:", rip_check);
        if rip_check == 0 {
            crate::kerror!("(Spawn) CRITICAL: TrapFrame RIP é ZERO após escrita!");
        }
    }

    // 7. Restaurar CR3 original (APÓS escrever TrapFrame!)
    unsafe {
        crate::mm::vmm::mapper::write_cr3(old_cr3);
    }
    crate::kinfo!("(Spawn) CR3 restaurado para Kernel P4");

    // 6. Marcar como pronta
    task.set_ready();
    // let pid = Pid::new(task.tid.as_u32()); // Already defined above

    // 7. Adicionar ao scheduler
    crate::sched::core::enqueue(Box::pin(task));
    crate::kinfo!("Process spawned from ELF! PID:", pid.as_u32() as u64);

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
