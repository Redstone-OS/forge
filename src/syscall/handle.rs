//! Syscalls de Handle (Capability-based)
//!
//! Handles são referências opacas para objetos do kernel.
//! Cada processo tem sua própria tabela de handles (HandleTable).

use super::error::{SysError, SysResult};
use crate::security::capability::CapRights;

/// Tipo de objeto para criação de handle.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleType {
    /// Porta de IPC
    Port = 1,
    /// Região de memória
    Memory = 2,
    /// Arquivo/Diretório VFS
    File = 3,
    /// Dispositivo
    Device = 4,
    /// Outra tarefa
    Task = 5,
}

/// Entrada na tabela de handles.
#[derive(Debug, Clone)]
pub struct HandleEntry {
    pub handle_type: HandleType,
    pub object_ptr: usize,
    pub rights: CapRights,
    pub active: bool,
}

impl HandleEntry {
    pub fn new(handle_type: HandleType, object_ptr: usize, rights: CapRights) -> Self {
        Self {
            handle_type,
            object_ptr,
            rights,
            active: true,
        }
    }
}

/// Cria um handle para um objeto.
///
/// # Syscall
/// `SYS_HANDLE_CREATE (0x20)` - Args: (type, object_ptr, object_len, rights)
///
/// # Nota
/// Esta syscall é principalmente para uso interno do kernel.
/// Userspace normalmente recebe handles via outras syscalls (create_port, open, etc).
pub fn sys_handle_create(
    handle_type: usize,
    _object_ptr: usize,
    _object_len: usize,
    _rights: usize,
) -> SysResult<usize> {
    // Validar tipo
    let _htype = match handle_type as u32 {
        1 => HandleType::Port,
        2 => HandleType::Memory,
        3 => HandleType::File,
        4 => HandleType::Device,
        5 => HandleType::Task,
        _ => return Err(SysError::InvalidArgument),
    };

    // TODO: Acessar HandleTable do processo atual via task
    // TODO: Inserir entrada e retornar handle

    crate::kwarn!("[Syscall] handle_create não implementado completamente");
    Err(SysError::NotImplemented)
}

/// Duplica um handle com direitos potencialmente reduzidos.
///
/// # Syscall
/// `SYS_HANDLE_DUP (0x21)` - Args: (handle, new_rights)
///
/// # Regras
/// - Novos direitos devem ser subconjunto dos direitos atuais
/// - Handle original permanece válido
pub fn sys_handle_dup(handle: usize, new_rights: usize) -> SysResult<usize> {
    if handle == 0 {
        return Err(SysError::BadHandle);
    }

    let _new_rights = CapRights::from_bits_truncate(new_rights as u32);

    // TODO: Acessar HandleTable, validar handle, verificar rights, criar cópia

    crate::kwarn!("[Syscall] handle_dup não implementado");
    Err(SysError::NotImplemented)
}

/// Fecha um handle.
///
/// # Syscall
/// `SYS_HANDLE_CLOSE (0x22)` - Args: (handle)
///
/// # Comportamento
/// - Remove handle da tabela
/// - Decrementa refcount do objeto
/// - Pode liberar objeto se refcount = 0
pub fn sys_handle_close(handle: usize) -> SysResult<usize> {
    if handle == 0 {
        return Err(SysError::BadHandle);
    }

    // TODO: Acessar HandleTable, remover entrada, cleanup

    crate::kwarn!("[Syscall] handle_close não implementado");
    Err(SysError::NotImplemented)
}

/// Verifica se um handle possui os direitos especificados.
///
/// # Syscall
/// `SYS_CHECK_RIGHTS (0x23)` - Args: (handle, rights_mask)
///
/// # Retorno
/// - 1 se handle possui todos os direitos solicitados
/// - 0 se falta algum direito
/// - Erro se handle inválido
pub fn sys_check_rights(handle: usize, rights_mask: usize) -> SysResult<usize> {
    if handle == 0 {
        return Err(SysError::BadHandle);
    }

    let _required = CapRights::from_bits_truncate(rights_mask as u32);

    // TODO: Acessar HandleTable, verificar direitos

    crate::kwarn!("[Syscall] check_rights não implementado");
    Err(SysError::NotImplemented)
}
