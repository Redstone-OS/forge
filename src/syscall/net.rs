//! Syscalls de Rede (Sockets).
//!
//! Atualmente são apenas stubs que retornam erro, pois a stack de rede
//! ainda não foi portada para o Kernel Forge.

use super::numbers::ENOSYS;

/// Cria um endpoint de comunicação (socket).
///
/// # Arguments
/// * `domain`: Família de protocolo (ex: AF_INET).
/// * `type_`: Tipo de comunicação (ex: SOCK_STREAM).
/// * `protocol`: Protocolo específico.
pub fn sys_socket(_domain: usize, _type_: usize, _protocol: usize) -> isize {
    // TODO: Implementar criação de socket
    ENOSYS
}

/// Inicia uma conexão em um socket.
pub fn sys_connect(_fd: usize, _addr: usize, _len: usize) -> isize {
    // TODO: Implementar conexão
    ENOSYS
}

/// Envia uma mensagem em um socket.
pub fn sys_sendto(
    _fd: usize,
    _buf: usize,
    _len: usize,
    _flags: usize,
    _addr: usize,
    _addr_len: usize,
) -> isize {
    ENOSYS
}

/// Recebe uma mensagem de um socket.
pub fn sys_recvfrom(
    _fd: usize,
    _buf: usize,
    _len: usize,
    _flags: usize,
    _addr: usize,
    _addr_len: usize,
) -> isize {
    ENOSYS
}

/// Liga um nome a um socket.
pub fn sys_bind(_fd: usize, _addr: usize, _len: usize) -> isize {
    ENOSYS
}

/// Escuta conexões em um socket.
pub fn sys_listen(_fd: usize, _backlog: usize) -> isize {
    ENOSYS
}

/// Aceita uma conexão em um socket.
pub fn sys_accept(_fd: usize, _addr: usize, _addr_len: usize) -> isize {
    ENOSYS
}
