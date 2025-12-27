//! # Syscall Table
//!
//! Tabela estática de handlers indexada por número de syscall.

use super::super::abi::SyscallArgs;
use super::super::error::SysResult;
use super::super::numbers::*;

/// Tipo de handler de syscall
pub type SyscallHandler = fn(&SyscallArgs) -> SysResult<usize>;

/// Tamanho da tabela (256 syscalls possíveis)
pub const TABLE_SIZE: usize = 256;

/// Tabela de syscalls
///
/// Inicializada estaticamente com todos os handlers.
/// None = syscall não implementada.
pub static SYSCALL_TABLE: [Option<SyscallHandler>; TABLE_SIZE] = {
    let mut table: [Option<SyscallHandler>; TABLE_SIZE] = [None; TABLE_SIZE];

    // === PROCESSO (0x01-0x0F) ===
    table[SYS_EXIT] = Some(super::super::process::sys_exit_wrapper);
    table[SYS_SPAWN] = Some(super::super::process::sys_spawn_wrapper);
    table[SYS_WAIT] = Some(super::super::process::sys_wait_wrapper);
    table[SYS_YIELD] = Some(super::super::process::sys_yield_wrapper);
    table[SYS_GETPID] = Some(super::super::process::sys_getpid_wrapper);

    // === MEMÓRIA (0x10-0x1F) ===
    table[SYS_ALLOC] = Some(super::super::memory::sys_alloc_wrapper);
    table[SYS_FREE] = Some(super::super::memory::sys_free_wrapper);
    table[SYS_MAP] = Some(super::super::memory::sys_map_wrapper);
    table[SYS_UNMAP] = Some(super::super::memory::sys_unmap_wrapper);

    // === HANDLES (0x20-0x2F) ===
    table[SYS_HANDLE_DUP] = Some(super::super::handle::sys_handle_dup_wrapper);
    table[SYS_HANDLE_CLOSE] = Some(super::super::handle::sys_handle_close_wrapper);
    table[SYS_CHECK_RIGHTS] = Some(super::super::handle::sys_check_rights_wrapper);

    // === IPC (0x30-0x3F) ===
    table[SYS_CREATE_PORT] = Some(super::super::ipc::sys_create_port_wrapper);
    table[SYS_SEND_MSG] = Some(super::super::ipc::sys_send_msg_wrapper);
    table[SYS_RECV_MSG] = Some(super::super::ipc::sys_recv_msg_wrapper);

    // === FILESYSTEM (0x60-0x6F) ===
    table[SYS_OPEN] = Some(super::super::fs::sys_open_wrapper);
    table[SYS_CLOSE] = Some(super::super::fs::sys_close_wrapper);
    table[SYS_READ] = Some(super::super::fs::sys_read_wrapper);
    table[SYS_WRITE] = Some(super::super::fs::sys_write_wrapper);
    table[SYS_STAT] = Some(super::super::fs::sys_stat_wrapper);
    table[SYS_FSTAT] = Some(super::super::fs::sys_fstat_wrapper);
    table[SYS_LSEEK] = Some(super::super::fs::sys_lseek_wrapper);

    // === EVENTS (0x80-0x8F) ===
    table[SYS_POLL] = Some(super::super::event::sys_poll_wrapper);

    // === TEMPO (0x50-0x5F) ===
    table[SYS_CLOCK_GET] = Some(super::super::time::sys_clock_get_wrapper);
    table[SYS_SLEEP] = Some(super::super::time::sys_sleep_wrapper);

    // === SISTEMA (0xF0-0xFF) ===
    table[SYS_SYSINFO] = Some(super::super::system::sys_sysinfo_wrapper);
    table[SYS_DEBUG] = Some(super::super::system::sys_debug_wrapper);

    table
};
