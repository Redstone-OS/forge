//! # Syscalls de Framebuffer
//!
//! Permite que userspace acesse o framebuffer para renderização gráfica.

use crate::drivers::video::framebuffer as fb;
use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// ============================================================================
// TIPOS COMPARTILHADOS (ABI estável)
// ============================================================================

/// Informações do framebuffer para userspace
///
/// Layout: C-compatible, packed, versão estável da ABI
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FramebufferInfo {
    /// Largura em pixels
    pub width: u32,
    /// Altura em pixels
    pub height: u32,
    /// Bytes por linha (stride)
    pub stride: u32,
    /// Bits por pixel (geralmente 32)
    pub bpp: u32,
    /// Formato de pixel (0=RGB, 1=BGR)
    pub format: u32,
}

// ============================================================================
// IMPLEMENTAÇÕES
// ============================================================================

/// SYS_FB_INFO: Obtém informações do framebuffer
///
/// Args:
///   - arg1: ponteiro para FramebufferInfo (userspace)
///
/// Retorna: 0 em sucesso, erro caso contrário
pub fn sys_fb_info(out_ptr: *mut FramebufferInfo) -> SysResult<usize> {
    // Validar ponteiro (básico - verificar se não é nulo)
    if out_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    // Obter info do driver de framebuffer
    let info = fb::get_info().ok_or(SysError::NotFound)?;

    // Converter para struct userspace
    let user_info = FramebufferInfo {
        width: info.width,
        height: info.height,
        stride: info.stride,
        bpp: info.bpp,
        format: match info.format {
            crate::core::boot::handoff::PixelFormat::Rgb => 0,
            crate::core::boot::handoff::PixelFormat::Bgr => 1,
            _ => 2, // Outro formato
        },
    };

    // Copiar para userspace
    // SAFETY: Ponteiro foi validado como não-nulo
    // TODO: Validar se ponteiro está realmente em espaço de usuário
    unsafe {
        core::ptr::write_volatile(out_ptr, user_info);
    }

    Ok(0)
}

/// SYS_FB_WRITE: Escreve pixels no framebuffer
///
/// Args:
///   - arg1: offset em bytes no framebuffer
///   - arg2: ponteiro para dados (userspace)
///   - arg3: tamanho em bytes
///
/// Retorna: bytes escritos ou erro
pub fn sys_fb_write(offset: usize, data_ptr: *const u8, len: usize) -> SysResult<usize> {
    // DEBUG: Log para verificar se está sendo chamado
    static mut FIRST_CALL: bool = true;
    static mut CALL_COUNT: u64 = 0;

    unsafe {
        CALL_COUNT += 1;
        if FIRST_CALL {
            FIRST_CALL = false;
            crate::kinfo!("(FB_WRITE) Primeira chamada!");
            crate::ktrace!("(FB_WRITE) offset:", offset as u64);
            crate::ktrace!("(FB_WRITE) len:", len as u64);
            crate::ktrace!("(FB_WRITE) data_ptr:", data_ptr as u64);
        }
    }

    // Validar ponteiro
    if data_ptr.is_null() {
        return Err(SysError::BadAddress);
    }

    // Obter info do framebuffer
    let info = fb::get_info().ok_or(SysError::NotFound)?;

    // DEBUG: Log do endereço do framebuffer na primeira chamada
    unsafe {
        static mut FB_ADDR_LOGGED: bool = false;
        if !FB_ADDR_LOGGED {
            FB_ADDR_LOGGED = true;
            crate::ktrace!("(FB_WRITE) fb_addr:", info.addr.as_u64());

            // Verificar primeiro pixel do userspace
            let first_pixel = *(data_ptr as *const u32);
            crate::ktrace!("(FB_WRITE) first_pixel:", first_pixel as u64);
        }
    }

    // Calcular tamanho máximo do framebuffer
    let fb_size = (info.height as usize) * (info.stride as usize);

    // Validar bounds
    if offset >= fb_size || offset + len > fb_size {
        crate::kerror!("(FB_WRITE) Bounds check falhou!");
        return Err(SysError::InvalidArgument);
    }

    // Copiar dados para framebuffer
    // SAFETY: Bounds foram validados
    unsafe {
        let fb_ptr = info.addr.as_mut_ptr::<u8>().add(offset);

        // Copiar byte a byte (sem SSE)
        let mut i = 0;
        while i < len {
            *fb_ptr.add(i) = *data_ptr.add(i);
            i += 1;
        }
    }

    Ok(len)
}

/// SYS_FB_CLEAR: Limpa o framebuffer com uma cor
///
/// Args:
///   - arg1: cor ARGB (32 bits)
///
/// Retorna: 0 em sucesso
pub fn sys_fb_clear(color: u32) -> SysResult<usize> {
    fb::clear(color);
    Ok(0)
}

// ============================================================================
// WRAPPERS PARA DISPATCH TABLE
// ============================================================================

pub fn sys_fb_info_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let out_ptr = args.arg1 as *mut FramebufferInfo;
    sys_fb_info(out_ptr)
}

pub fn sys_fb_write_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let offset = args.arg1;
    let data_ptr = args.arg2 as *const u8;
    let len = args.arg3;
    sys_fb_write(offset, data_ptr, len)
}

pub fn sys_fb_clear_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    let color = args.arg1 as u32;
    sys_fb_clear(color)
}
