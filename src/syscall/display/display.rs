//! # Display Syscalls
//!
//! Syscalls para controle do display.
//! Mantém compatibilidade com a API legada de framebuffer.

use crate::drivers::display::DISPLAY_CRTC;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// ============================================================================
// Estrutura de resposta legada (compatibilidade com SDK)
// ============================================================================

/// Estrutura legada de informações de framebuffer (5 u32s = 20 bytes)
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct LegacyFramebufferInfo {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bpp: u32,
    pub format: u32,
}

// ============================================================================
// SYS_FB_INFO (0x40) - Compatibilidade Legado
// ============================================================================

/// Obtém informações do framebuffer.
///
/// API Legada:
/// - arg1: ponteiro para FramebufferInfo (userspace)
///
/// Returns: 0 em caso de sucesso.
pub fn sys_display_info_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut LegacyFramebufferInfo;

    if out_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    let crtc = DISPLAY_CRTC.lock();
    let info = crtc.get_info();

    // Converter para formato legado
    let legacy_info = LegacyFramebufferInfo {
        width: info.width,
        height: info.height,
        stride: info.stride / 4, // stride em pixels, não bytes
        bpp: 32,
        format: 0, // ARGB8888
    };

    unsafe {
        core::ptr::write_volatile(out_ptr, legacy_info);
    }

    Ok(0)
}

// ============================================================================
// SYS_FB_WRITE (0x41) - Compatibilidade Legado
// ============================================================================

/// Escreve dados no framebuffer.
///
/// API Legada:
/// - arg1: offset em bytes
/// - arg2: ponteiro para dados
/// - arg3: tamanho em bytes
///
/// Returns: bytes escritos ou erro.
pub fn sys_display_commit_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let offset = args.arg1;
    let data_ptr = args.arg2 as *const u8;
    let len = args.arg3;

    if data_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    if len == 0 {
        return Ok(0);
    }

    let crtc = DISPLAY_CRTC.lock();
    let info = crtc.get_info();
    let fb_size = (info.width * info.height * 4) as usize;

    // Validar bounds
    if offset >= fb_size {
        return Err(SysError::InvalidArgument);
    }

    let actual_len = len.min(fb_size - offset);

    // Debug log: log first call only
    static mut FIRST_CALL: bool = true;
    unsafe {
        if FIRST_CALL {
            FIRST_CALL = false;
            crate::kinfo!("(FB_WRITE) offset=", offset as u64);
            crate::kinfo!("(FB_WRITE) len=", len as u64);
            crate::kinfo!("(FB_WRITE) actual_len=", actual_len as u64);
            crate::kinfo!("(FB_WRITE) fb_size=", fb_size as u64);
        }
    }

    // Copiar dados para o framebuffer usando chunks de 64 bits
    unsafe {
        let fb_ptr = crtc.framebuffer_ptr();
        if fb_ptr.is_null() {
            return Err(SysError::IoError);
        }

        let dst = fb_ptr.add(offset);

        // Copiar em chunks de 8 bytes (64 bits) para performance
        let chunks = actual_len / 8;
        let remainder = actual_len % 8;

        let src_u64 = data_ptr as *const u64;
        let dst_u64 = dst as *mut u64;

        for i in 0..chunks {
            let val = core::ptr::read_volatile(src_u64.add(i));
            core::ptr::write_volatile(dst_u64.add(i), val);
        }

        // Copiar bytes restantes
        let offset_bytes = chunks * 8;
        for i in 0..remainder {
            let byte = core::ptr::read_volatile(data_ptr.add(offset_bytes + i));
            core::ptr::write_volatile(dst.add(offset_bytes + i), byte);
        }
    }

    Ok(actual_len)
}

// ============================================================================
// SYS_FB_CLEAR (0x42) - Compatibilidade Legado
// ============================================================================

/// Limpa todo o framebuffer com uma cor.
///
/// API Legada:
/// - arg1: cor ARGB
///
/// Returns: 0 em caso de sucesso.
pub fn sys_display_clear_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let color = args.arg1 as u32;

    let crtc = DISPLAY_CRTC.lock();
    crtc.clear(color);

    Ok(0)
}
