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
    table[SYS_CREATE_PORT] = Some(super::super::ipc::port::sys_create_port_wrapper);
    table[SYS_SEND_MSG] = Some(super::super::ipc::port::sys_send_msg_wrapper);
    table[SYS_RECV_MSG] = Some(super::super::ipc::port::sys_recv_msg_wrapper);
    table[SYS_FUTEX_WAIT] = Some(super::super::ipc::port::sys_futex_wait_wrapper);
    table[SYS_FUTEX_WAKE] = Some(super::super::ipc::port::sys_futex_wake_wrapper);
    table[SYS_SHM_CREATE] = Some(super::super::ipc::shm::sys_shm_create_wrapper);
    table[SYS_SHM_MAP] = Some(super::super::ipc::shm::sys_shm_map_wrapper);
    table[SYS_PORT_CONNECT] = Some(super::super::ipc::port::sys_port_connect_wrapper);
    table[SYS_SHM_GET_SIZE] = Some(super::super::ipc::shm::sys_shm_get_size_wrapper);

    // === DISPLAY (0x40-0x4F) ===
    table[SYS_FB_INFO] = Some(super::super::display::sys_display_info_wrapper);
    table[SYS_FB_WRITE] = Some(super::super::display::sys_display_commit_wrapper);
    table[SYS_FB_CLEAR] = Some(super::super::display::sys_display_clear_wrapper);
    // Input syscalls
    table[SYS_MOUSE_READ] = Some(super::super::display::sys_mouse_read_wrapper);
    table[SYS_KEYBOARD_READ] = Some(super::super::display::sys_keyboard_read_wrapper);

    // === TEMPO (0x50-0x5F) ===
    table[SYS_CLOCK_GET] = Some(super::super::time::sys_clock_get_wrapper);
    table[SYS_SLEEP] = Some(super::super::time::sys_sleep_wrapper);
    table[SYS_TIMER_CREATE] = Some(super::super::time::sys_timer_create_wrapper);
    table[SYS_TIMER_SET] = Some(super::super::time::sys_timer_set_wrapper);

    // =========================================================================
    // FILESYSTEM (0x60-0x7F)
    // =========================================================================

    // --- BÁSICO (0x60-0x67) ---
    table[SYS_OPEN] = Some(super::super::fs::sys_open_wrapper);
    table[SYS_READ] = Some(super::super::fs::sys_read_wrapper);
    table[SYS_WRITE] = Some(super::super::fs::sys_write_wrapper);
    table[SYS_SEEK] = Some(super::super::fs::sys_seek_wrapper);
    table[SYS_PREAD] = Some(super::super::fs::sys_pread_wrapper);
    table[SYS_PWRITE] = Some(super::super::fs::sys_pwrite_wrapper);
    table[SYS_FLUSH] = Some(super::super::fs::sys_flush_wrapper);
    table[SYS_TRUNCATE] = Some(super::super::fs::sys_truncate_wrapper);

    // --- METADADOS (0x68-0x6B) ---
    table[SYS_STAT] = Some(super::super::fs::sys_stat_wrapper);
    table[SYS_FSTAT] = Some(super::super::fs::sys_fstat_wrapper);
    table[SYS_CHMOD] = Some(super::super::fs::sys_chmod_wrapper);
    table[SYS_CHOWN] = Some(super::super::fs::sys_chown_wrapper);

    // --- DIRETÓRIOS (0x6C-0x6F) ---
    table[SYS_GETDENTS] = Some(super::super::fs::sys_getdents_wrapper);
    table[SYS_MKDIR] = Some(super::super::fs::sys_mkdir_wrapper);
    table[SYS_RMDIR] = Some(super::super::fs::sys_rmdir_wrapper);
    table[SYS_GETCWD] = Some(super::super::fs::sys_getcwd_wrapper);

    // --- MANIPULAÇÃO (0x70-0x73) ---
    table[SYS_CREATE] = Some(super::super::fs::sys_create_wrapper);
    table[SYS_UNLINK] = Some(super::super::fs::sys_unlink_wrapper);
    table[SYS_RENAME] = Some(super::super::fs::sys_rename_wrapper);
    table[SYS_LINK] = Some(super::super::fs::sys_link_wrapper);

    // --- SYMLINKS (0x74-0x76) ---
    table[SYS_SYMLINK] = Some(super::super::fs::sys_symlink_wrapper);
    table[SYS_READLINK] = Some(super::super::fs::sys_readlink_wrapper);
    table[SYS_REALPATH] = Some(super::super::fs::sys_realpath_wrapper);

    // --- MONTAGEM (0x77-0x7A) ---
    table[SYS_MOUNT] = Some(super::super::fs::sys_mount_wrapper);
    table[SYS_UMOUNT] = Some(super::super::fs::sys_umount_wrapper);
    table[SYS_STATFS] = Some(super::super::fs::sys_statfs_wrapper);
    table[SYS_SYNC] = Some(super::super::fs::sys_sync_wrapper);

    // --- AVANÇADO (0x7B-0x7F) ---
    table[SYS_IOCTL] = Some(super::super::fs::sys_ioctl_wrapper);
    table[SYS_FCNTL] = Some(super::super::fs::sys_fcntl_wrapper);
    table[SYS_FLOCK] = Some(super::super::fs::sys_flock_wrapper);
    table[SYS_ACCESS] = Some(super::super::fs::sys_access_wrapper);
    table[SYS_CHDIR] = Some(super::super::fs::sys_chdir_wrapper);

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
