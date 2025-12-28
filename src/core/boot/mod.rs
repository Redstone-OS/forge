//! Inicialização do kernel

pub mod cmdline;
pub mod entry;
pub mod handoff;
pub mod initcall;
pub mod panic;

pub use entry::kernel_main;
pub use handoff::BootInfo;
