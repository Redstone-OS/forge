//! # Criação de Processos
//!
//! Orquestração do spawn de novos processos.
//!
//! Este módulo coordena:
//! 1. Leitura do binário via VFS
//! 2. Detecção de formato (ELF, script)
//! 3. Criação de Task e Address Space
//! 4. Carregamento de segmentos
//! 5. Setup de stacks (kernel e user)
//! 6. Configuração de contexto
//! 7. Enfileiramento no scheduler

use alloc::boxed::Box;
use alloc::sync::Arc;

use crate::rmm::addr::VirtAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::virt::aspace::vma::MemoryIntent;
use crate::rmm::virt::aspace::AddressSpace;
use crate::rmm::virt::hhdm;
use crate::rmm::zone::Zone;
use crate::sync::Spinlock;
use crate::sys::types::Pid;

use super::config::{KERNEL_STACK_BASE, KERNEL_STACK_SIZE, USER_STACK_SIZE, USER_STACK_TOP};
use super::context::setup_user_trap_frame;
use super::error::ExecError;
use super::fmt::{self, BinaryFormat};

/// Cria um novo processo a partir de um executável
///
/// # Argumentos
///
/// * `path` - Caminho do executável no VFS
/// * `parent_id` - TID do processo pai (None para processos iniciais)
///
/// # Retorna
///
/// * `Ok(Pid)` - PID do novo processo
/// * `Err(ExecError)` - Se houver falha na criação
///
/// # Exemplo
///
/// ```rust,ignore
/// match spawn("/bin/init", None) {
///     Ok(pid) => kinfo!("Process spawned:", pid.as_u32()),
///     Err(e) => kerror!("Spawn failed:", e.as_str()),
/// }
/// ```
pub fn spawn(path: &str, parent_id: Option<crate::sys::types::Tid>) -> Result<Pid, ExecError> {
    crate::kinfo!("(Spawn) Loading:", path.as_ptr() as u64);

    // 1. Ler arquivo via VFS
    let data = crate::fs::vfs::read_file(path).ok_or_else(|| {
        crate::kerror!("(Spawn) File not found:", path.as_ptr() as u64);
        ExecError::NotFound
    })?;

    // 2. Detectar formato
    let format = fmt::detect_format(&data);
    fmt::validate_format(format)?;

    // Apenas ELF suportado por enquanto
    if format != BinaryFormat::Elf64 {
        return Err(ExecError::UnsupportedType);
    }

    // 3. Criar Task
    let mut task = crate::sched::task::Task::new(path);
    task.parent_id = parent_id;
    let pid = Pid::new(task.tid.as_u32());
    let pid_u32 = pid.as_u32();
    let pid_u64 = pid_u32 as u64;

    crate::ktrace!("(Spawn) Created task, PID:", pid_u64);

    // 4. Criar Address Space isolado (AddressSpace::new espera u32)
    let aspace = Arc::new(Spinlock::new(
        AddressSpace::new(pid_u32).map_err(|_| ExecError::AddressSpaceError)?,
    ));
    task.aspace = Some(aspace.clone());

    // 5. Mapear Kernel Stack
    let kstack_size = KERNEL_STACK_SIZE as u64;
    let kstack_start = KERNEL_STACK_BASE + (pid_u64 * kstack_size);
    let kstack_top = kstack_start + kstack_size;

    map_kernel_stack(&aspace, kstack_start, kstack_size)?;
    task.kernel_stack = VirtAddr::new(kstack_top);

    crate::ktrace!("(Spawn) Kernel stack mapped at:", kstack_start);

    // 6. Carregar ELF
    let entry_point = super::fmt::elf::load_binary(&data, &aspace)?;

    crate::ktrace!("(Spawn) ELF loaded, entry:", entry_point.as_u64());

    // 7. Mapear User Stack
    crate::ktrace!("(Spawn) Mapeando user stack...");
    let ustack_size = USER_STACK_SIZE as usize;
    let ustack_start = USER_STACK_TOP - ustack_size as u64;

    map_user_stack(&aspace, ustack_start, ustack_size)?;
    task.user_stack = VirtAddr::new(USER_STACK_TOP);

    crate::ktrace!("(Spawn) User stack mapped at:", ustack_start);

    // 8. Configurar Trap Frame
    let target_cr3 = aspace.lock().cr3();
    let (rsp, rip) = unsafe { setup_user_trap_frame(target_cr3, kstack_top, entry_point)? };

    task.context.rsp = rsp;
    task.context.rip = rip;

    // 9. Enfileirar Task
    crate::ktrace!("(Spawn) Enfileirando task...");
    task.set_ready();
    crate::sched::core::enqueue(Box::pin(task));
    crate::ktrace!("(Spawn) Task enfileirada OK");

    crate::kinfo!("(Spawn) Process spawned successfully, PID:", pid_u64);
    crate::kdebug!(
        "(Spawn) Iniciado na CPU:",
        super::super::core::per_cpu::this_cpu_id() as u64
    );
    Ok(pid)
}

/// Mapeia a stack de kernel para um processo
fn map_kernel_stack(
    aspace: &Arc<Spinlock<AddressSpace>>,
    kstack_start: u64,
    kstack_size: u64,
) -> Result<(), ExecError> {
    let page_size = PAGE_SIZE as u64;
    let pages = kstack_size / page_size;
    let target_cr3 = aspace.lock().cr3();

    for i in 0..pages {
        let vaddr = kstack_start + i * page_size;

        // Alocar frame zerado (kernel owner)
        let frame = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
            .ok_or(ExecError::OutOfMemory)?;

        // Mapear no address space do processo (kernel flags, sem USER)
        map_kernel_page(target_cr3, vaddr, frame.as_u64())?;
    }

    Ok(())
}

/// Mapeia a stack de usuário para um processo
fn map_user_stack(
    aspace: &Arc<Spinlock<AddressSpace>>,
    ustack_start: u64,
    ustack_size: usize,
) -> Result<(), ExecError> {
    // 1. Registrar VMA (stack usa MemoryIntent::Stack que já configura flags corretas)
    {
        let mut as_guard = aspace.lock();
        as_guard
            .map_region(
                VirtAddr::new(ustack_start),
                ustack_size,
                crate::rmm::virt::aspace::vma::Protection::RW,
                MemoryIntent::Stack, // RMM configura GROWABLE e STACK automaticamente
            )
            .map_err(|_| ExecError::MappingFailed)?;
    }

    // 2. Alocar e mapear páginas
    let page_size = PAGE_SIZE as u64;
    let pages = ustack_size as u64 / page_size;
    let target_cr3 = aspace.lock().cr3();

    for i in 0..pages {
        let vaddr = ustack_start + i * page_size;

        let frame = phys::alloc(
            FrameOwner::Process { pid: 0 },
            Zone::Normal,
            AllocFlags::ZERO,
        )
        .ok_or(ExecError::OutOfMemory)?;

        map_user_page(target_cr3, vaddr, frame.as_u64())?;
    }

    Ok(())
}

/// Mapeia uma página no espaço de kernel (sem USER flag)
fn map_kernel_page(target_cr3: u64, vaddr: u64, phys_frame: u64) -> Result<(), ExecError> {
    // Flags: Present, Writable (sem User)
    map_page_internal(target_cr3, vaddr, phys_frame, 0x3)
}

/// Mapeia uma página no espaço de usuário (com USER flag)
fn map_user_page(target_cr3: u64, vaddr: u64, phys_frame: u64) -> Result<(), ExecError> {
    // Flags: Present, Writable, User
    map_page_internal(target_cr3, vaddr, phys_frame, 0x7)
}

/// Implementação interna de mapeamento de página
fn map_page_internal(
    target_cr3: u64,
    vaddr: u64,
    phys_frame: u64,
    flags: u64,
) -> Result<(), ExecError> {
    // Flags para tabelas intermediárias (sempre writable e user para permitir acesso)
    let table_flags: u64 = 0x7;

    unsafe {
        let pml4_phys = target_cr3 & !0xFFF;
        let pml4 = hhdm::phys_to_virt(pml4_phys) as *mut u64;

        let pml4_idx = ((vaddr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((vaddr >> 30) & 0x1FF) as usize;
        let pd_idx = ((vaddr >> 21) & 0x1FF) as usize;
        let pt_idx = ((vaddr >> 12) & 0x1FF) as usize;

        // Ensure PDPT exists
        if (*pml4.add(pml4_idx)) & 1 == 0 {
            let new_pdpt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ExecError::OutOfMemory)?;
            *pml4.add(pml4_idx) = new_pdpt.as_u64() | table_flags;
        }

        let pdpt_phys = (*pml4.add(pml4_idx)) & 0x000F_FFFF_FFFF_F000;
        let pdpt = hhdm::phys_to_virt(pdpt_phys) as *mut u64;

        // Ensure PD exists
        if (*pdpt.add(pdpt_idx)) & 1 == 0 {
            let new_pd = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ExecError::OutOfMemory)?;
            *pdpt.add(pdpt_idx) = new_pd.as_u64() | table_flags;
        }

        let pd_phys = (*pdpt.add(pdpt_idx)) & 0x000F_FFFF_FFFF_F000;
        let pd = hhdm::phys_to_virt(pd_phys) as *mut u64;

        // Ensure PT exists
        if (*pd.add(pd_idx)) & 1 == 0 {
            let new_pt = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
                .ok_or(ExecError::OutOfMemory)?;
            *pd.add(pd_idx) = new_pt.as_u64() | table_flags;
        }

        let pt_phys = (*pd.add(pd_idx)) & 0x000F_FFFF_FFFF_F000;
        let pt = hhdm::phys_to_virt(pt_phys) as *mut u64;

        // Map the page with specified flags
        *pt.add(pt_idx) = phys_frame | flags;
    }

    Ok(())
}
