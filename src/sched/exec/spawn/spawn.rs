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

    // 3. Criar task
    crate::kinfo!("(Spawn) Creating task struct...");
    let mut task = crate::sched::task::Task::new(path);
    crate::kinfo!("(Spawn) Task created via Task::new");

    // === PROCESS ISOLATION SETUP ===

    // 0. Alocar e Mapear Kernel Stack (ANTES de criar P4, para que seja copiado)
    let pid = Pid::new(task.tid.as_u32());
    const KERNEL_STACK_BASE: u64 = 0xFFFF_9100_0000_0000;
    const KERNEL_STACK_SIZE: u64 = 8192; // 2 pages

    let pid_u64 = pid.as_u32() as u64;
    let kstack_start = KERNEL_STACK_BASE + (pid_u64 * KERNEL_STACK_SIZE);
    let kstack_top = kstack_start + KERNEL_STACK_SIZE;

    // Alocar frames e mapear (no Kernel P4 atual)
    {
        crate::kinfo!("(Spawn) Allocating KStack frames for PID:", pid_u64);
        let mut pmm = FRAME_ALLOCATOR.lock();
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE; // Kernel acessa
        let pages = KERNEL_STACK_SIZE / FRAME_SIZE;

        for i in 0..pages {
            let vaddr = kstack_start + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                unsafe {
                    if let Err(_e) = map_page_with_pmm(vaddr, frame.as_u64(), flags, &mut *pmm) {
                        return Err(ExecError::OutOfMemory);
                    }
                    // Zerar stack
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

    // 1. Criar nova Page Table isolada (copia Kernel Half + Identity Map)
    // Agora inclui o mapeamento da KStack que acabamos de criar.
    let new_p4 = {
        let mut pmm = FRAME_ALLOCATOR.lock();
        crate::mm::vmm::mapper::create_new_p4(&mut *pmm).expect("(Spawn) Falha ao criar P4")
    };
    task.cr3 = new_p4;
    crate::kinfo!("(Spawn) Nova PML4 criada:", new_p4);

    // 2. Trocar para nova P4 temporariamente para carregar ELF e configurar User Space
    let old_cr3 = crate::mm::vmm::mapper::read_cr3();
    unsafe {
        crate::mm::vmm::mapper::write_cr3(new_p4);
    }
    crate::kinfo!("(Spawn) CR3 trocado para nova P4 (contexto temporário)");

    // 3. Carregar ELF (agora mapeia na nova P4)
    crate::kinfo!("(Spawn) Chamando elf::load_binary...");
    let entry_point = match crate::sched::exec::elf::load_binary(data) {
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
        let start_page = USER_STACK_TOP - USER_STACK_SIZE;
        let pages = USER_STACK_SIZE / FRAME_SIZE;

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
        crate::kinfo!("(Spawn) Stack allocated OK.");
    }

    // 6. Restaurar CR3 original
    unsafe {
        crate::mm::vmm::mapper::write_cr3(old_cr3);
    }
    crate::kinfo!("(Spawn) CR3 restaurado para Kernel P4");

    // 7. Configurar Trap Frame na Kernel Stack (Visível em ambas P4s)
    unsafe {
        crate::kinfo!("(Spawn) Building TrapFrame at:", kstack_top);
        let ptr = kstack_top as *mut u64;

        // Seletores (RPL 3)
        // GDT Order: 0=Null, 1=KCode, 2=KData, 3=UData, 4=UCode, 5-6=TSS
        // index 3 = User Data = (3 << 3) | 3 = 0x1B
        // index 4 = User Code = (4 << 3) | 3 = 0x23
        const USER_CODE_SEL: u64 = 0x23; // Index 4, RPL 3
        const USER_DATA_SEL: u64 = 0x1B; // Index 3, RPL 3

        // RFLAGS:
        // 0x202 = Interrupts Enabled + Reserved Bit 1
        const RFLAGS_IF: u64 = 0x202;

        // === Layout do TrapFrame na stack ===
        //
        // O jump_to_context_asm faz:
        //   mov rsp, [context.rsp]   ; RSP = context.rsp
        //   push [context.rip]       ; RSP -= 8, escreve trampolim
        //   ret                      ; pop, RSP volta a context.rsp
        //
        // Após ret, RSP = context.rsp. O iretq vai ler a partir de RSP.
        // Então o TrapFrame deve estar a partir de context.rsp:
        //   [context.rsp + 0]  = RIP (user)
        //   [context.rsp + 8]  = CS
        //   [context.rsp + 16] = RFLAGS
        //   [context.rsp + 24] = RSP (user)
        //   [context.rsp + 32] = SS
        //
        // E context.rsp = kstack_top - 40 (para ter espaço para 5 qwords)

        let trapframe_base = ptr.offset(-5); // kstack_top - 40
        trapframe_base.offset(0).write(entry_point.as_u64()); // RIP
        trapframe_base.offset(1).write(USER_CODE_SEL); // CS
        trapframe_base.offset(2).write(RFLAGS_IF); // RFLAGS
        trapframe_base.offset(3).write(USER_STACK_TOP); // RSP
        trapframe_base.offset(4).write(USER_DATA_SEL); // SS

        // Trampolim
        let trampoline = crate::sched::context::switch::iretq_restore as u64;

        // context.rsp deve apontar para o início do TrapFrame
        // Após push+ret no asm, RSP = context.rsp
        let context_rsp = trapframe_base as u64;
        task.context.rsp = context_rsp;
        task.context.rip = trampoline;

        crate::ktrace!("(Spawn) TrapFrame base:", context_rsp);
        crate::ktrace!("(Spawn) Trampolim:", trampoline);
    }

    // 6. Marcar como pronta
    task.set_ready();
    // let pid = Pid::new(task.tid.as_u32()); // Already defined above

    // 7. Adicionar ao scheduler
    crate::sched::scheduler::enqueue(Box::pin(task));
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
