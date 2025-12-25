//! # IPC Syscalls (Message Passing)
//!
//! O sistema nervoso central do Redstone OS. Tudo que não é computação pura é comunicação.
//!
//! ## 🎯 Propósito
//! - **Isolamento:** Processos não compartilham memória (exceto setup explícito). Eles trocam mensagens.
//! - **Synchronization:** O ato de enviar/receber mensagem também sincroniza os processos (Rendezvous).
//!
//! ## 🏗️ Arquitetura: Port-Based IPC
//! - **Ports:** São filas de mensagens no kernel.
//! - **Handles:** Quem tem o Handle da porta pode escrever (Send) ou ler (Recv), dependendo dos direitos.
//! - **Copy-Semantics:** Mensagens pequenas são copiadas. Mensagens grandes usam *Memory Grant* (fase 2).
//!
//! ## 🔍 Análise Crítica
//!
//! ### ✅ Pontos Fortes
//! - **Desacoplamento:** O remetente não precisa saber quem é o destinatário, apenas ter o handle da porta de serviço. Isso facilita *Service Swapping*.
//!
//! ### ⚠️ Pontos de Atenção
//! - **Overhead de Cópia:** `sys_send_msg` copia dados User->Kernel, e `sys_recv_msg` copia Kernel->User. (2 cópias).
//!   - *Correção:* Para payloads grandes, precisamos de *Zero-Copy* (transferência de páginas de memória).
//! - **Queue Flooding:** Se um servidor for lento, a fila enche. O sender bloqueia ou falha? Precisamos de *Backpressure*.
//!
//! ## 🛠️ TODOs
//! - [ ] **TODO: (Performance)** Implementar **Zero-Copy IPC** para mensagens > 4KiB.
//! - [ ] **TODO: (Architecture)** Definir **IPC Namespace** para descoberta de serviços (Name Server).
//!
//! --------------------------------------------------------------------------------
//!
//! Comunicação entre processos via portas de mensagens.
//! Modelo: portas são endpoints para filas de mensagens.

use super::error::{SysError, SysResult};
use crate::ipc::PortHandle;

/// Capacidade padrão de uma porta.
const DEFAULT_PORT_CAPACITY: usize = 32;

/// Cria uma porta de IPC.
///
/// # Syscall
/// `SYS_CREATE_PORT (0x30)` - Args: (capacity)
///
/// # Argumentos
/// - `capacity`: Tamanho máximo da fila (0 = default)
///
/// # Retorno
/// Handle da porta criada
pub fn sys_create_port(capacity: usize) -> SysResult<usize> {
    let cap = if capacity == 0 {
        DEFAULT_PORT_CAPACITY
    } else {
        capacity.min(1024) // Limite máximo
    };

    // Criar porta
    let _port = PortHandle::new(cap);

    // TODO: Registrar porta na HandleTable do processo
    // TODO: Retornar handle

    crate::kwarn!("(Syscall) sys_create_port: Porta criada mas HandleTable não implementada");
    Err(SysError::NotImplemented)
}

/// Envia mensagem para uma porta.
///
/// # Syscall
/// `SYS_SEND_MSG (0x31)` - Args: (port_handle, msg_ptr, msg_len, flags)
///
/// # Argumentos
/// - `port_handle`: Handle da porta (com direito WRITE)
/// - `msg_ptr`: Ponteiro para dados da mensagem
/// - `msg_len`: Tamanho dos dados
/// - `flags`: msg_flags (NONBLOCK, URGENT)
///
/// # Retorno
/// Bytes enviados ou erro
pub fn sys_send_msg(
    port_handle: usize,
    msg_ptr: usize,
    msg_len: usize,
    _flags: usize,
) -> SysResult<usize> {
    if port_handle == 0 {
        return Err(SysError::BadHandle);
    }

    if msg_ptr == 0 && msg_len > 0 {
        return Err(SysError::BadAddress);
    }

    if msg_len > crate::ipc::message::MAX_MESSAGE_SIZE {
        return Err(SysError::MessageTooLarge);
    }

    // TODO: Obter PortHandle da HandleTable
    // TODO: Verificar direito WRITE
    // TODO: Copiar dados e enviar

    crate::kwarn!("(Syscall) sys_send_msg não implementado");
    Err(SysError::NotImplemented)
}

/// Recebe mensagem de uma porta.
///
/// # Syscall
/// `SYS_RECV_MSG (0x32)` - Args: (port_handle, buf_ptr, buf_len, timeout_ms)
///
/// # Argumentos
/// - `port_handle`: Handle da porta (com direito READ)
/// - `buf_ptr`: Buffer de destino
/// - `buf_len`: Tamanho do buffer
/// - `timeout_ms`: Timeout (0 = não bloquear, u64::MAX = infinito)
///
/// # Retorno
/// Bytes recebidos ou erro
pub fn sys_recv_msg(
    port_handle: usize,
    buf_ptr: usize,
    buf_len: usize,
    _timeout_ms: usize,
) -> SysResult<usize> {
    if port_handle == 0 {
        return Err(SysError::BadHandle);
    }

    if buf_ptr == 0 && buf_len > 0 {
        return Err(SysError::BadAddress);
    }

    // TODO: Obter PortHandle da HandleTable
    // TODO: Verificar direito READ
    // TODO: Receber mensagem (bloqueante ou não)

    crate::kwarn!("(Syscall) sys_recv_msg não implementado");
    Err(SysError::NotImplemented)
}

/// Verifica mensagem sem remover da fila.
///
/// # Syscall
/// `SYS_PEEK_MSG (0x33)` - Args: (port_handle, buf_ptr, buf_len)
///
/// # Retorno
/// Tamanho da próxima mensagem ou erro
pub fn sys_peek_msg(port_handle: usize, _buf_ptr: usize, _buf_len: usize) -> SysResult<usize> {
    if port_handle == 0 {
        return Err(SysError::BadHandle);
    }

    // TODO: Obter PortHandle
    // TODO: Verificar se há mensagem
    // TODO: Copiar para buf sem remover

    crate::kwarn!("(Syscall) sys_peek_msg não implementado");
    Err(SysError::NotImplemented)
}
