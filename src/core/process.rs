//! Process Management
//!
//! High-level abstractions for process lifecycle.

pub fn spawn_init() {
    // Tenta spawnar o init process.
    let init_path = "/system/core/init";

    // Caminho para a função spawn via sched -> exec -> spawn (mod) -> spawn (file) -> spawn (func)
    match crate::sched::exec::spawn(init_path) {
        Ok(pid) => {
            // Access public field .0 since Pid is tuple struct
            crate::kinfo!("Init process spawned. PID:", pid.0 as u64);
        }
        Err(crate::sched::ExecError::NotFound) => {
            core::panic!(
                "Failed to spawn init process: Init executable not found at {}",
                init_path
            );
        }
        Err(_e) => {
            core::panic!("Failed to spawn init process! Error: {:?}", _e);
        }
    }
}
