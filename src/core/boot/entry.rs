//! Entry Point do Kernel
//!
//! Função `kernel_main` - primeiro código Rust após o trampolim assembly.

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use crate::core::boot::handoff::{BootInfo, BOOT_INFO_VERSION, BOOT_MAGIC};
use crate::drivers::serial;
use alloc::vec::Vec;

/// Função principal do Kernel.
/// Chamada pelo `_start` (assembly) com a stack configurada.
/// Não retorna.
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Validação de sanidade
    if boot_info.magic != BOOT_MAGIC || boot_info.version != BOOT_INFO_VERSION {
        Cpu::hang();
    }

    serial::write("[FORGE] Kernel v0.1.0\n");

    // Arquitetura (GDT/IDT/TSS)
    unsafe {
        crate::arch::platform::gdt::init();
        crate::arch::platform::idt::init();
    }

    // Memória (PMM/VMM/Heap)
    unsafe {
        crate::mm::init(boot_info);
    }

    // Drivers (PIC/PIT)
    crate::drivers::pic::init();
    crate::drivers::pic::unmask(0);
    crate::drivers::timer::init(250);

    // Subsistemas
    crate::ipc::init();
    crate::fs::init(boot_info);

    unsafe {
        crate::drivers::video::init(&boot_info.framebuffer);
    }

    // Scheduler
    crate::sched::scheduler::init();

    // Syscall
    crate::syscall::init();

    // Carregar init
    spawn_init_process();

    // Habilitar interrupções
    unsafe {
        Cpu::enable_interrupts();
    }

    // Idle loop
    loop {
        Cpu::halt();
    }
}

/// Carrega e executa /system/core/init
fn spawn_init_process() {
    use crate::fs::vfs::ROOT_VFS;

    let vfs = ROOT_VFS.lock();

    if let Ok(node) = vfs.lookup("/system/core/init") {
        if let Ok(handle) = node.open() {
            let size = node.size() as usize;
            let mut buffer = Vec::with_capacity(size);
            unsafe {
                buffer.set_len(size);
            }

            if let Ok(bytes_read) = handle.read(&mut buffer, 0) {
                match unsafe { crate::sys::elf::load(&buffer[..bytes_read]) } {
                    Ok(entry_point) => {
                        let user_stack_size = 16 * 1024;
                        let user_stack_base = 0x8000_0000 - user_stack_size as u64;
                        let user_stack_top = 0x8000_0000;

                        {
                            use crate::mm::pmm::FRAME_ALLOCATOR;
                            use crate::mm::vmm::{self, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};

                            let mut addr = user_stack_base;
                            while addr < user_stack_top {
                                let frame = FRAME_ALLOCATOR
                                    .lock()
                                    .allocate_frame()
                                    .expect("No frames for user stack");
                                unsafe {
                                    vmm::map_page(
                                        addr,
                                        frame.addr(),
                                        PAGE_PRESENT | PAGE_USER | PAGE_WRITABLE,
                                    )
                                    .expect("Failed to map user stack");
                                    core::arch::asm!("invlpg [{0}]", in(reg) addr, options(nostack, preserves_flags));
                                }
                                addr += 4096;
                            }
                        }

                        let cr3 = unsafe { crate::arch::platform::memory::cr3() };
                        let task =
                            crate::sched::task::Task::new_user(entry_point, user_stack_top, cr3);
                        crate::sched::scheduler::SCHEDULER.lock().add_task(task);
                    }
                    Err(_) => {}
                }
            }
        }
    } else {
        crate::sched::scheduler::SCHEDULER
            .lock()
            .add_task(crate::sched::task::Task::new_kernel(dummy_init));
    }
}

/// Tarefa dummy quando não há init
extern "C" fn dummy_init() {
    loop {
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }
}
