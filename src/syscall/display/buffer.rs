//! # Buffer Syscalls
//!
//! Syscalls para gerenciamento de buffers de display.

use crate::drivers::display::BUFFER_MANAGER;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};
use gfx_types::{BufferDescriptor, BufferHandle, PixelFormat};

// ============================================================================
// SYS_BUFFER_CREATE
// ============================================================================

/// Cria novo buffer de display.
///
/// # Args
/// - arg1: largura em pixels
/// - arg2: altura em pixels
/// - arg3: formato (PixelFormat)
///
/// # Returns
/// BufferHandle.0 em caso de sucesso.
pub fn sys_buffer_create(width: u32, height: u32, format: u32) -> SysResult<usize> {
    let pixel_format = match format {
        0 => PixelFormat::ARGB8888,
        1 => PixelFormat::XRGB8888,
        2 => PixelFormat::RGB565,
        3 => PixelFormat::BGRA8888,
        _ => return Err(SysError::InvalidArgument),
    };

    let desc = BufferDescriptor::new(width, height, pixel_format);

    let mut mgr = BUFFER_MANAGER.lock();
    let handle = mgr.create(desc).map_err(|_| SysError::OutOfMemory)?;

    Ok(handle.0 as usize)
}

/// Wrapper para dispatch table.
pub fn sys_buffer_create_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_buffer_create(args.arg1 as u32, args.arg2 as u32, args.arg3 as u32)
}

// ============================================================================
// SYS_BUFFER_MAP
// ============================================================================

/// Mapeia buffer para o espaço de endereçamento do processo.
///
/// # Args
/// - arg1: BufferHandle
/// - arg2: endereço virtual desejado (0 = auto)
///
/// # Returns
/// Endereço virtual onde o buffer foi mapeado.
pub fn sys_buffer_map(handle: u64, hint_addr: u64) -> SysResult<usize> {
    let buffer_handle = BufferHandle(handle);

    // Endereço padrão se não especificado
    let vaddr = if hint_addr == 0 {
        // Usar região fixa de userspace para buffers gráficos
        0x0000_4000_0000_0000u64 + (handle * 0x1000_0000)
    } else {
        hint_addr
    };

    let mut mgr = BUFFER_MANAGER.lock();
    let mapped_addr = mgr.map(buffer_handle, vaddr).map_err(|e| match e {
        crate::drivers::display::buffer::BufferError::InvalidHandle => SysError::InvalidArgument,
        crate::drivers::display::buffer::BufferError::AlreadyMapped => SysError::AlreadyExists,
        _ => SysError::IoError,
    })?;

    Ok(mapped_addr.as_u64() as usize)
}

/// Wrapper para dispatch table.
pub fn sys_buffer_map_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_buffer_map(args.arg1 as u64, args.arg2 as u64)
}

// ============================================================================
// SYS_BUFFER_DESTROY
// ============================================================================

/// Libera um buffer.
///
/// # Args
/// - arg1: BufferHandle
///
/// # Returns
/// 0 em caso de sucesso.
pub fn sys_buffer_destroy(handle: u64) -> SysResult<usize> {
    let buffer_handle = BufferHandle(handle);

    let mut mgr = BUFFER_MANAGER.lock();
    mgr.destroy(buffer_handle)
        .map_err(|_| SysError::InvalidArgument)?;

    Ok(0)
}

/// Wrapper para dispatch table.
pub fn sys_buffer_destroy_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_buffer_destroy(args.arg1 as u64)
}

// ============================================================================
// SYS_BUFFER_INFO
// ============================================================================

/// Obtém informações de um buffer.
///
/// # Args
/// - arg1: BufferHandle
/// - arg2: ponteiro para BufferDescriptor (userspace)
///
/// # Returns
/// 0 em caso de sucesso.
pub fn sys_buffer_info(handle: u64, out_ptr: *mut BufferDescriptor) -> SysResult<usize> {
    if out_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    let buffer_handle = BufferHandle(handle);

    let mgr = BUFFER_MANAGER.lock();
    let buffer = mgr.get(buffer_handle).ok_or(SysError::InvalidArgument)?;

    unsafe {
        core::ptr::write_volatile(out_ptr, buffer.desc);
    }

    Ok(0)
}

/// Wrapper para dispatch table.
pub fn sys_buffer_info_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_buffer_info(args.arg1 as u64, args.arg2 as *mut BufferDescriptor)
}
