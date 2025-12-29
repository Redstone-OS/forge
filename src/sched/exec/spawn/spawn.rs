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
/// Topo da stack de High Memory (debugging)
const HIGH_STACK_TOP: u64 = 0xFFFF_9001_0000_0000;

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
    crate::kinfo!("(Spawn) Allocating user stack...");
    {
        crate::kinfo!("(Spawn) Locking PMM...");
        let mut pmm = FRAME_ALLOCATOR.lock();
        crate::kinfo!("(Spawn) PMM locked.");
        // Removendo MapFlags::USER para testar execução em Ring 0 sem SMAP issues
        // let flags = MapFlags::PRESENT | MapFlags::WRITABLE; // | MapFlags::USER; (Testando SEM USER)
        // REATIVANDO USER e MUDANDO PARA HIGH MEMORY para evitar conflitos de Lower Half
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE | MapFlags::USER;

        // Usar endereço ALTO para stack de usuário (fake por enquanto, rodando em Ring 0)
        // Canonical High: 0xFFFF_8000_0000_0000 + ...
        // Vamos usar 0xFFFF_9001_0000_0000 (acima do Heap que começa em 9000...)
        // Usar endereço ALTO para stack de usuário (fake por enquanto, rodando em Ring 0)
        // Canonical High: 0xFFFF_8000_0000_0000 + ...
        // Vamos usar 0xFFFF_9001_0000_0000 (acima do Heap que começa em 9000...)
        // const HIGH_STACK_TOP moved to module scope
        let start_page = HIGH_STACK_TOP - USER_STACK_SIZE;
        let pages = USER_STACK_SIZE / FRAME_SIZE;
        crate::kinfo!("(Spawn) Stack pages needed:", pages);

        for i in 0..pages {
            let vaddr = start_page + i * FRAME_SIZE;
            // crate::kinfo!("(Spawn) Stack page:", i);
            if let Some(frame) = pmm.allocate_frame() {
                // TODO: Mapear no address space do processo (atualmente no kernel)
                unsafe {
                    if let Err(_e) = map_page_with_pmm(vaddr, frame.as_u64(), flags, &mut *pmm) {
                        return Err(ExecError::OutOfMemory);
                    }
                    // Zerar stack (volatile)
                    let ptr = vaddr as *mut u8;
                    for j in 0..FRAME_SIZE as usize {
                        ptr.add(j).write_volatile(0);
                    }
                }
            } else {
                return Err(ExecError::OutOfMemory);
            }
        }
        crate::kinfo!("(Spawn) Stack allocated OK.");
    }

    // 5. Alocar Kernel Stack e Configurar Contexto Ring 3 (IRETQ)
    crate::kinfo!("(Spawn) Configurando Kernel Stack e Trap Frame...");

    let pid = Pid::new(task.tid.as_u32()); // Moved up to be available for kstack calculation

    // Definir região de Kernel Stacks
    // Base: 0xFFFF_9100_0000_0000
    const KERNEL_STACK_BASE: u64 = 0xFFFF_9100_0000_0000;
    const KERNEL_STACK_SIZE: u64 = 8192; // 2 pages

    let pid_u64 = pid.as_u32() as u64;
    let kstack_start = KERNEL_STACK_BASE + (pid_u64 * KERNEL_STACK_SIZE);
    let kstack_top = kstack_start + KERNEL_STACK_SIZE;

    // Alocar frames e mapear
    {
        crate::kinfo!("(Spawn) Allocating KStack frames for PID:", pid_u64);
        let mut pmm = FRAME_ALLOCATOR.lock();
        let flags = MapFlags::PRESENT | MapFlags::WRITABLE; // Kernel acessa (sem USER)
        let pages = KERNEL_STACK_SIZE / FRAME_SIZE;

        for i in 0..pages {
            let vaddr = kstack_start + i * FRAME_SIZE;
            if let Some(frame) = pmm.allocate_frame() {
                unsafe {
                    if let Err(_e) = map_page_with_pmm(vaddr, frame.as_u64(), flags, &mut *pmm) {
                        return Err(ExecError::OutOfMemory);
                    }
                    // Zerar stack manual (volatile)
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

    // Atualizar task info
    task.kernel_stack = VirtAddr::new(kstack_top);
    task.user_stack = VirtAddr::new(USER_STACK_TOP);

    unsafe {
        // Construir Trap Frame no topo da Kernel Stack
        crate::kinfo!("(Spawn) Building TrapFrame at:", kstack_top);
        let ptr = kstack_top as *mut u64;

        // Seletores (RPL 3)
        const USER_CODE_SEL: u64 = 0x1B;
        const USER_DATA_SEL: u64 = 0x23;
        const RFLAGS_IF: u64 = 0x202;

        // Offset manual - CUIDADO com alinhamento
        // ptr aponta para o limite superior (não mapeado/fim).
        // O primeiro u64 válido é ptr-1.

        ptr.offset(-1).write(USER_DATA_SEL); // SS
        ptr.offset(-2).write(USER_STACK_TOP); // RSP  (0x7FFFFFFFF000)
        ptr.offset(-3).write(RFLAGS_IF); // RFLAGS
        ptr.offset(-4).write(USER_CODE_SEL); // CS
        ptr.offset(-5).write(entry_point.as_u64()); // RIP

        // Configurar CpuContext
        // jump_to_context carrega RSP daqui.
        // Ele DEVE apontar para o início do TrapFrame (RIP), pois
        // quando jump_to_context fizer 'ret', ele vai para 'iretq_restore',
        // e 'iretq' vai esperar que RSP aponte para RIP.

        let context_rsp = (ptr.offset(-5)) as u64;

        task.context.rsp = context_rsp;

        // Trampolim de IRETQ
        let trampoline = crate::sched::context::switch::iretq_restore as u64;
        crate::kinfo!("(Spawn) Trampoline (iretq_restore) Addr: {:x}", trampoline);
        ptr.offset(-6).write(trampoline);

        // Dummy Regs for jump_to_context (pop R15..RBX)
        task.context.rip = trampoline; // This line was removed by the user's edit, but it's essential for context switching. Re-adding it.

        crate::kinfo!("(Spawn) TrapFrame built at:", context_rsp);
        crate::kinfo!("(Spawn)  RIP =", entry_point.as_u64());
        crate::kinfo!("(Spawn)  CS  =", USER_CODE_SEL);
        crate::kinfo!("(Spawn)  RFLAGS =", RFLAGS_IF);
        crate::kinfo!("(Spawn)  RSP =", USER_STACK_TOP);
        crate::kinfo!("(Spawn)  SS  =", USER_DATA_SEL);
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
    crate::kinfo!("!!! HELLO FROM KERNEL TASK !!!");
    crate::kinfo!("Context switch worked successfully.");
    loop {
        crate::arch::Cpu::halt();
    }
}
