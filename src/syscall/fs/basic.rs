//! # Filesystem Basic Operations
//!
//! open, close, read, write, stat, fstat, lseek

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_open_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_open(args.arg1, args.arg2, args.arg3 as u32)
}

pub fn sys_close_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_close(args.arg1 as u32)
}

pub fn sys_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_read(args.arg1 as u32, args.arg2, args.arg3)
}

pub fn sys_write_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_write(args.arg1 as u32, args.arg2, args.arg3)
}

pub fn sys_stat_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_stat(args.arg1, args.arg2, args.arg3)
}

pub fn sys_fstat_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_fstat(args.arg1 as u32, args.arg2)
}

pub fn sys_lseek_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_lseek(args.arg1 as u32, args.arg2 as i64, args.arg3 as u32)
}

// === IMPLEMENTAÇÕES ===

/// Abre um arquivo
///
/// # Args
/// - path_ptr: ponteiro para caminho
/// - path_len: tamanho do caminho
/// - flags: flags de abertura (O_RDONLY, O_WRONLY, etc)
///
/// # Returns
/// Handle do arquivo ou erro
pub fn sys_open(path_ptr: usize, path_len: usize, flags: u32) -> SysResult<usize> {
    // TODO: Validar ponteiro (copy_from_user)
    // TODO: Converter para &str
    // TODO: Chamar VFS.lookup()
    // TODO: Criar handle na tabela do processo
    // TODO: Retornar handle

    let _ = (path_ptr, path_len, flags);
    crate::kwarn!("(Syscall) sys_open não implementado");
    Err(SysError::NotImplemented)
}

/// Fecha um arquivo
///
/// # Args
/// - handle: handle do arquivo
///
/// # Returns
/// 0 ou erro
pub fn sys_close(handle: u32) -> SysResult<usize> {
    // TODO: Validar handle
    // TODO: Decrementar refcount
    // TODO: Se refcount == 0, fechar recurso

    let _ = handle;
    crate::kwarn!("(Syscall) sys_close não implementado");
    Err(SysError::NotImplemented)
}

/// Lê dados de um arquivo
///
/// # Args
/// - handle: handle do arquivo
/// - buf_ptr: buffer de destino
/// - len: tamanho máximo a ler
///
/// # Returns
/// Bytes lidos ou erro
pub fn sys_read(handle: u32, buf_ptr: usize, len: usize) -> SysResult<usize> {
    // TODO: Validar handle (rights: READ)
    // TODO: Validar buffer (copy_to_user)
    // TODO: Chamar VFS.read()
    // TODO: Retornar bytes lidos

    let _ = (handle, buf_ptr, len);
    crate::kwarn!("(Syscall) sys_read não implementado");
    Err(SysError::NotImplemented)
}

/// Escreve dados em um arquivo
///
/// # Args
/// - handle: handle do arquivo
/// - buf_ptr: buffer de origem
/// - len: tamanho a escrever
///
/// # Returns
/// Bytes escritos ou erro
pub fn sys_write(handle: u32, buf_ptr: usize, len: usize) -> SysResult<usize> {
    // TODO: Validar handle (rights: WRITE)
    // TODO: Validar buffer (copy_from_user)
    // TODO: Chamar VFS.write()
    // TODO: Retornar bytes escritos

    let _ = (handle, buf_ptr, len);
    crate::kwarn!("(Syscall) sys_write não implementado");
    Err(SysError::NotImplemented)
}

/// Obtém informações de arquivo por caminho
///
/// # Args
/// - path_ptr: ponteiro para caminho
/// - path_len: tamanho do caminho
/// - stat_ptr: ponteiro para struct Stat de saída
///
/// # Returns
/// 0 ou erro
pub fn sys_stat(path_ptr: usize, path_len: usize, stat_ptr: usize) -> SysResult<usize> {
    // TODO: Validar ponteiros
    // TODO: Lookup no VFS
    // TODO: Preencher Stat
    // TODO: copy_to_user

    let _ = (path_ptr, path_len, stat_ptr);
    crate::kwarn!("(Syscall) sys_stat não implementado");
    Err(SysError::NotImplemented)
}

/// Obtém informações de arquivo por handle
///
/// # Args
/// - handle: handle do arquivo
/// - stat_ptr: ponteiro para struct Stat de saída
///
/// # Returns
/// 0 ou erro
pub fn sys_fstat(handle: u32, stat_ptr: usize) -> SysResult<usize> {
    // TODO: Validar handle (rights: STAT)
    // TODO: Obter info do VFS node
    // TODO: Preencher Stat
    // TODO: copy_to_user

    let _ = (handle, stat_ptr);
    crate::kwarn!("(Syscall) sys_fstat não implementado");
    Err(SysError::NotImplemented)
}

/// Move posição de leitura/escrita
///
/// # Args
/// - handle: handle do arquivo
/// - offset: deslocamento
/// - whence: referência (SET, CUR, END)
///
/// # Returns
/// Nova posição ou erro
pub fn sys_lseek(handle: u32, offset: i64, whence: u32) -> SysResult<usize> {
    // TODO: Validar handle (rights: SEEK)
    // TODO: Calcular nova posição
    // TODO: Atualizar posição no handle
    // TODO: Retornar nova posição

    let _ = (handle, offset, whence);
    crate::kwarn!("(Syscall) sys_lseek não implementado");
    Err(SysError::NotImplemented)
}
