//! Ponto de entrada do kernel

use crate::core::boot::BootInfo;

/// Ponto de entrada principal do kernel.
///
/// Chamado pelo `_start` em main.rs após setup inicial.
///
/// # Ordem de Inicialização
///
/// 1. Debug/Logging
/// 2. Memória (PMM → VMM → Heap)
/// 3. Interrupções (IDT, APIC)
/// 4. Scheduler
/// 5. Syscalls
/// 6. Drivers
/// 7. Filesystem
/// 8. Init process
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Inicializar logging primeiro
    // crate::drivers::serial::init(); // Serial init may be called inside mod init or managed here if exposed
    crate::kinfo!("Forge kernel inicializando...");
    
    // 2. Inicializar memória
    unsafe { crate::mm::init(boot_info); }
    
    // 3. Inicializar interrupções
    unsafe { 
        crate::arch::x86_64::idt::init();
        crate::arch::x86_64::apic::init();
    }
    
    // 4. Inicializar scheduler
    crate::sched::init();
    
    // 5. Inicializar syscalls
    crate::syscall::init();
    
    // 6. Inicializar drivers
    crate::drivers::init();
    
    // 7. Inicializar filesystem
    crate::fs::init();
    
    // 8. Inicializar IPC
    crate::ipc::init();
    
    // 9. Inicializar módulos
    crate::module::init();
    
    crate::kinfo!("Kernel inicializado, buscando init...");
    
    // Carregar e executar init
    match crate::sched::exec::spawn("/system/core/init") {
        Ok(_pid) => {
            crate::kinfo!("Init spawned, entrando no scheduler...");
        }
        Err(e) => {
            crate::kerror!("Falha ao spawnar init:", e as u64);
            panic!("Não foi possível iniciar o processo init");
        }
    }
    
    // Entrar no loop do scheduler (nunca retorna)
    crate::sched::scheduler::run()
}
