// (FASE2) src/syscall/mod.rs
//! Interface de Chamadas de Sistema (Userspace -> Kernel).

pub mod dispatcher;
pub mod fs;
pub mod memory;
pub mod net;
pub mod numbers;
pub mod process;

// Handler de baixo nível (chamado pelo Assembly `int 0x80` ou `syscall`)
#[no_mangle]
pub extern "C" fn syscall_handler(num: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let result = dispatcher::dispatch(num as usize, arg1 as usize, arg2 as usize, arg3 as usize);
    result as u64
}
