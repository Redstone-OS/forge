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
    table[SYS_GETTID] = Some(super::super::process::sys_gettid_wrapper);
    table[SYS_THREAD_CREATE] = Some(super::super::process::sys_thread_create_wrapper);
    table[SYS_THREAD_EXIT] = Some(super::super::process::sys_thread_exit_wrapper);

    // === MEMÓRIA (0x10-0x1F) ===
    table[SYS_ALLOC] = Some(super::super::memory::sys_alloc_wrapper);
    table[SYS_FREE] = Some(super::super::memory::sys_free_wrapper);
    table[SYS_MAP] = Some(super::super::memory::sys_map_wrapper);
    table[SYS_UNMAP] = Some(super::super::memory::sys_unmap_wrapper);
    table[SYS_MPROTECT] = Some(super::super::memory::sys_mprotect_wrapper);

    // === HANDLES (0x20-0x2F) ===
    table[SYS_HANDLE_DUP] = Some(super::super::handle::sys_handle_dup_wrapper);
    table[SYS_HANDLE_CLOSE] = Some(super::super::handle::sys_handle_close_wrapper);
    table[SYS_CHECK_RIGHTS] = Some(super::super::handle::sys_check_rights_wrapper);

    // === IPC (0x30-0x3F) ===
    table[SYS_CREATE_PORT] = Some(super::super::ipc::sys_create_port_wrapper);
    table[SYS_SEND_MSG] = Some(super::super::ipc::sys_send_msg_wrapper);
    table[SYS_RECV_MSG] = Some(super::super::ipc::sys_recv_msg_wrapper);
    table[SYS_FUTEX_WAIT] = Some(super::super::ipc::sys_futex_wait_wrapper);
    table[SYS_FUTEX_WAKE] = Some(super::super::ipc::sys_futex_wake_wrapper);

    // === GRÁFICOS / INPUT (0x40-0x4F) ===
    table[SYS_FB_INFO] = Some(super::super::gfx::sys_fb_info_wrapper);
    table[SYS_FB_WRITE] = Some(super::super::gfx::sys_fb_write_wrapper);
    table[SYS_FB_CLEAR] = Some(super::super::gfx::sys_fb_clear_wrapper);
    table[SYS_MOUSE_READ] = Some(super::super::gfx::sys_mouse_read_wrapper);
    table[SYS_KEYBOARD_READ] = Some(super::super::gfx::sys_keyboard_read_wrapper);

    // === TEMPO (0x50-0x5F) ===
    table[SYS_CLOCK_GET] = Some(super::super::time::sys_clock_get_wrapper);
    table[SYS_SLEEP] = Some(super::super::time::sys_sleep_wrapper);
    table[SYS_TIMER_CREATE] = Some(super::super::time::sys_timer_create_wrapper);
    table[SYS_TIMER_SET] = Some(super::super::time::sys_timer_set_wrapper);

    // === FILESYSTEM (0x60-0x6F) ===
    table[SYS_OPEN] = Some(super::super::fs::sys_open_wrapper);
    table[SYS_CLOSE] = Some(super::super::fs::sys_close_wrapper);
    table[SYS_READ] = Some(super::super::fs::sys_read_wrapper);
    table[SYS_WRITE] = Some(super::super::fs::sys_write_wrapper);
    table[SYS_STAT] = Some(super::super::fs::sys_stat_wrapper);
    table[SYS_FSTAT] = Some(super::super::fs::sys_fstat_wrapper);
    table[SYS_LSEEK] = Some(super::super::fs::sys_lseek_wrapper);
    table[SYS_MKDIR] = Some(super::super::fs::sys_mkdir_wrapper);
    table[SYS_RMDIR] = Some(super::super::fs::sys_rmdir_wrapper);
    table[SYS_UNLINK] = Some(super::super::fs::sys_unlink_wrapper);
    table[SYS_READDIR] = Some(super::super::fs::sys_readdir_wrapper);
    table[SYS_CHMOD] = Some(super::super::fs::sys_chmod_wrapper);

    // === EVENTS (0x80-0x8F) ===
    table[SYS_POLL] = Some(super::super::event::sys_poll_wrapper);

    // === SISTEMA (0xF0-0xFF) ===
    table[SYS_SYSINFO] = Some(super::super::system::sys_sysinfo_wrapper);
    table[SYS_REBOOT] = Some(super::super::system::sys_reboot_wrapper);
    table[SYS_POWEROFF] = Some(super::super::system::sys_poweroff_wrapper);
    table[SYS_CONSOLE_WRITE] = Some(super::super::system::sys_console_write_wrapper);
    table[SYS_CONSOLE_READ] = Some(super::super::system::sys_console_read_wrapper);
    table[SYS_DEBUG] = Some(super::super::system::sys_debug_wrapper);

    table
};
