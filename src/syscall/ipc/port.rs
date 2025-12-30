//! # IPC Port Operations
//!
//! create_port, send, recv

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_create_port_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_create_port(args.arg1, args.arg2, args.arg3)
}

pub fn sys_send_msg_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_send_msg(args.arg1 as u32, args.arg2, args.arg3, args.arg4 as u32)
}

pub fn sys_recv_msg_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_recv_msg(args.arg1 as u32, args.arg2, args.arg3, args.arg4 as u64)
}

pub fn sys_futex_wait_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_futex_wait(args.arg1, args.arg2, args.arg3 as u64)
}

pub fn sys_futex_wake_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_futex_wake(args.arg1, args.arg2)
}

pub fn sys_port_connect_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_port_connect(args.arg1, args.arg2)
}

// === IMPLEMENTAÇÕES ===

/// Cria uma porta de IPC nomeada
///
/// # Args
/// - name_ptr: ponteiro para nome (UTF-8)
/// - name_len: tamanho do nome
/// - capacity: capacidade máxima de mensagens na fila
///
/// # Returns
/// Handle da porta ou erro
pub fn sys_create_port(name_ptr: usize, name_len: usize, capacity: usize) -> SysResult<usize> {
    use alloc::string::String;
    use alloc::vec::Vec;

    // Validar ponteiros (hack temporário: apenas check bounds básico)
    if name_ptr == 0 || name_len > 256 {
        return Err(SysError::InvalidArgument);
    }

    // Copiar nome do user stack/heap
    let mut name_bytes = Vec::with_capacity(name_len);
    unsafe {
        let ptr = name_ptr as *const u8;
        for i in 0..name_len {
            name_bytes.push(*ptr.add(i));
        }
    }

    let name = String::from_utf8(name_bytes).map_err(|_| SysError::InvalidArgument)?;

    crate::ipc::manager::create_port(&name, capacity).map_err(|_| SysError::AlreadyExists)
}

/// Conecta a uma porta de IPC nomeada
pub fn sys_port_connect(name_ptr: usize, name_len: usize) -> SysResult<usize> {
    use alloc::string::String;
    use alloc::vec::Vec;

    if name_ptr == 0 || name_len > 256 {
        return Err(SysError::InvalidArgument);
    }

    let mut name_bytes = Vec::with_capacity(name_len);
    unsafe {
        let ptr = name_ptr as *const u8;
        for i in 0..name_len {
            name_bytes.push(*ptr.add(i));
        }
    }

    let name = String::from_utf8(name_bytes).map_err(|_| SysError::InvalidArgument)?;

    crate::ipc::manager::connect_port(&name).map_err(|_| SysError::NotFound)
}

/// Envia mensagem para uma porta
///
/// # Args
/// - port_handle: handle da porta
/// - msg_ptr: ponteiro para dados da mensagem
/// - msg_len: tamanho da mensagem
/// - flags: flags (NONBLOCK, etc)
///
/// # Returns
/// Bytes enviados ou erro
pub fn sys_send_msg(
    port_handle: u32,
    msg_ptr: usize,
    msg_len: usize,
    _flags: u32,
) -> SysResult<usize> {
    if msg_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    use alloc::vec::Vec;
    let mut data = Vec::with_capacity(msg_len);
    unsafe {
        let ptr = msg_ptr as *const u8;
        for i in 0..msg_len {
            data.push(*ptr.add(i));
        }
    }

    crate::ipc::manager::send_msg(port_handle as usize, &data).map_err(|_| SysError::InvalidHandle)
}

/// Recebe mensagem de uma porta
///
/// # Args
/// - port_handle: handle da porta
/// - buf_ptr: buffer para receber mensagem
/// - buf_len: tamanho do buffer
/// - timeout_ms: timeout em ms (0 = bloqueante infinito)
///
/// # Returns
/// Bytes recebidos ou erro
pub fn sys_recv_msg(
    port_handle: u32,
    buf_ptr: usize,
    buf_len: usize,
    _timeout_ms: u64,
) -> SysResult<usize> {
    if buf_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    // Alocar buffer temporário no kernel (ineficiente, mas seguro sem copy_to_user ainda)
    let mut kbuf = alloc::vec![0u8; buf_len];

    match crate::ipc::manager::recv_msg(port_handle as usize, &mut kbuf) {
        Ok(len) => {
            // Copiar para user
            unsafe {
                let ptr = buf_ptr as *mut u8;
                for i in 0..len {
                    *ptr.add(i) = kbuf[i];
                }
            }
            Ok(len)
        }
        Err(_) => Err(SysError::InvalidHandle),
    }
}

/// Suspende a thread até que o valor mude (futex)
pub fn sys_futex_wait(addr: usize, expected: usize, timeout_ms: u64) -> SysResult<usize> {
    let _ = (addr, expected, timeout_ms);
    crate::kwarn!("(Syscall) sys_futex_wait não implementado");
    Err(SysError::NotImplemented)
}

/// Acorda threads esperando em um futex
pub fn sys_futex_wake(addr: usize, count: usize) -> SysResult<usize> {
    let _ = (addr, count);
    crate::kwarn!("(Syscall) sys_futex_wake não implementado");
    Err(SysError::NotImplemented)
}
