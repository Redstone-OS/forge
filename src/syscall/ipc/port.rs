//! # IPC Port Operations
//!
//! create_port, send, recv

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_create_port_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_create_port(args.arg1)
}

pub fn sys_send_msg_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_send_msg(args.arg1 as u32, args.arg2, args.arg3, args.arg4 as u32)
}

pub fn sys_recv_msg_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_recv_msg(args.arg1 as u32, args.arg2, args.arg3, args.arg4 as u64)
}

// === IMPLEMENTAÇÕES ===

/// Cria uma porta de IPC
///
/// # Args
/// - capacity: capacidade máxima de mensagens na fila
///
/// # Returns
/// Handle da porta ou erro
pub fn sys_create_port(capacity: usize) -> SysResult<usize> {
    // TODO: Validar capacity
    // TODO: Criar Port no subsistema IPC
    // TODO: Criar handle para o Port
    // TODO: Retornar handle

    let _ = capacity;
    crate::kwarn!("(Syscall) sys_create_port não implementado");
    Err(SysError::NotImplemented)
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
    flags: u32,
) -> SysResult<usize> {
    // TODO: Validar handle (rights: WRITE)
    // TODO: Validar ponteiro (copy_from_user)
    // TODO: Enfileirar mensagem na porta
    // TODO: Acordar receivers se houver

    let _ = (port_handle, msg_ptr, msg_len, flags);
    crate::kwarn!("(Syscall) sys_send_msg não implementado");
    Err(SysError::NotImplemented)
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
    timeout_ms: u64,
) -> SysResult<usize> {
    // TODO: Validar handle (rights: READ)
    // TODO: Validar buffer (copy_to_user depois)
    // TODO: Se fila vazia e !NONBLOCK, bloquear
    // TODO: Copiar mensagem para buffer do usuário

    let _ = (port_handle, buf_ptr, buf_len, timeout_ms);
    crate::kwarn!("(Syscall) sys_recv_msg não implementado");
    Err(SysError::NotImplemented)
}
