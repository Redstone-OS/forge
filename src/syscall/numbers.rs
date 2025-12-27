//! # Syscall Numbers Registry
//!
//! Catálogo central de todas as operações suportadas pelo kernel.
//!
//! ## ⚠️ IMUTABILIDADE
//! Uma vez atribuído, um número de syscall NUNCA muda.
//! Nova funcionalidade = novo número.
//!
//! ## Organização
//!
//! | Range     | Categoria     |
//! |-----------|---------------|
//! | 0x01-0x0F | Processo      |
//! | 0x10-0x1F | Memória       |
//! | 0x20-0x2F | Handles       |
//! | 0x30-0x3F | IPC           |
//! | 0x50-0x5F | Tempo         |
//! | 0x60-0x6F | Filesystem    |
//! | 0x80-0x8F | Events        |
//! | 0xF0-0xFF | Sistema/Debug |

// ============================================================================
// PROCESSO (0x01 - 0x0F)
// ============================================================================

/// Encerra o processo atual.
/// Args: (exit_code: i32)
/// Retorno: Nunca retorna
pub const SYS_EXIT: usize = 0x01;

/// Cria um novo processo.
/// Args: (path_ptr, path_len, args_ptr, args_len)
/// Retorno: pid ou erro
pub const SYS_SPAWN: usize = 0x02;

/// Espera um processo terminar.
/// Args: (pid, timeout_ms)
/// Retorno: exit_code ou erro
pub const SYS_WAIT: usize = 0x03;

/// Cede o restante do quantum de tempo.
/// Args: nenhum
/// Retorno: 0
pub const SYS_YIELD: usize = 0x04;

/// Obtém o PID do processo atual.
/// Args: nenhum
/// Retorno: pid
pub const SYS_GETPID: usize = 0x05;

/// Obtém informações sobre uma tarefa.
/// Args: (pid, out_ptr)
/// Retorno: 0 ou erro
pub const SYS_GETTASKINFO: usize = 0x06;

// ============================================================================
// MEMÓRIA (0x10 - 0x1F)
// ============================================================================

/// Aloca memória virtual.
/// Args: (size, flags)
/// Retorno: endereço ou erro
pub const SYS_ALLOC: usize = 0x10;

/// Libera memória alocada.
/// Args: (addr, size)
/// Retorno: 0 ou erro
pub const SYS_FREE: usize = 0x11;

/// Mapeia região de memória ou handle.
/// Args: (addr, size, flags, handle)
/// Retorno: endereço ou erro
pub const SYS_MAP: usize = 0x12;

/// Remove mapeamento de memória.
/// Args: (addr, size)
/// Retorno: 0 ou erro
pub const SYS_UNMAP: usize = 0x13;

// ============================================================================
// HANDLES (0x20 - 0x2F)
// ============================================================================

/// Duplica um handle com direitos reduzidos.
/// Args: (handle, new_rights)
/// Retorno: new_handle ou erro
pub const SYS_HANDLE_DUP: usize = 0x20;

/// Fecha um handle.
/// Args: (handle)
/// Retorno: 0 ou erro
pub const SYS_HANDLE_CLOSE: usize = 0x21;

/// Verifica direitos de um handle.
/// Args: (handle, rights_mask)
/// Retorno: 1 (tem) ou 0 (não tem)
pub const SYS_CHECK_RIGHTS: usize = 0x22;

// ============================================================================
// IPC (0x30 - 0x3F)
// ============================================================================

/// Cria uma porta de IPC.
/// Args: (capacity)
/// Retorno: handle da porta ou erro
pub const SYS_CREATE_PORT: usize = 0x30;

/// Envia mensagem para uma porta.
/// Args: (port_handle, msg_ptr, msg_len, flags)
/// Retorno: bytes enviados ou erro
pub const SYS_SEND_MSG: usize = 0x31;

/// Recebe mensagem de uma porta.
/// Args: (port_handle, buf_ptr, buf_len, timeout_ms)
/// Retorno: bytes recebidos ou erro
pub const SYS_RECV_MSG: usize = 0x32;

// ============================================================================
// TEMPO (0x50 - 0x5F)
// ============================================================================

/// Obtém tempo do sistema.
/// Args: (clock_id, out_ptr)
/// Retorno: 0 ou erro
pub const SYS_CLOCK_GET: usize = 0x50;

/// Dorme por N milissegundos.
/// Args: (ms)
/// Retorno: ms restantes (se interrompido)
pub const SYS_SLEEP: usize = 0x51;

// ============================================================================
// FILESYSTEM (0x60 - 0x6F)
// ============================================================================

/// Abre um arquivo.
/// Args: (path_ptr, path_len, flags)
/// Retorno: handle ou erro
pub const SYS_OPEN: usize = 0x60;

/// Fecha um arquivo.
/// Args: (handle)
/// Retorno: 0 ou erro
pub const SYS_CLOSE: usize = 0x61;

/// Lê dados de um arquivo.
/// Args: (handle, buf_ptr, len)
/// Retorno: bytes lidos ou erro
pub const SYS_READ: usize = 0x62;

/// Escreve dados em um arquivo.
/// Args: (handle, buf_ptr, len)
/// Retorno: bytes escritos ou erro
pub const SYS_WRITE: usize = 0x63;

/// Obtém informações de arquivo por caminho.
/// Args: (path_ptr, path_len, stat_ptr)
/// Retorno: 0 ou erro
pub const SYS_STAT: usize = 0x64;

/// Obtém informações de arquivo por handle.
/// Args: (handle, stat_ptr)
/// Retorno: 0 ou erro
pub const SYS_FSTAT: usize = 0x65;

/// Move posição de leitura/escrita.
/// Args: (handle, offset, whence)
/// Retorno: nova posição ou erro
pub const SYS_LSEEK: usize = 0x66;

// ============================================================================
// EVENTS (0x80 - 0x8F)
// ============================================================================

/// Espera eventos em múltiplos handles.
/// Args: (fds_ptr, nfds, timeout_ms)
/// Retorno: número de handles com eventos ou erro
pub const SYS_POLL: usize = 0x80;

// ============================================================================
// SISTEMA / DEBUG (0xF0 - 0xFF)
// ============================================================================

/// Obtém informações do sistema.
/// Args: (out_ptr, out_len)
/// Retorno: bytes escritos ou erro
pub const SYS_SYSINFO: usize = 0xF0;

/// Reinicia o sistema.
/// Args: nenhum
/// Retorno: nunca retorna
pub const SYS_REBOOT: usize = 0xF1;

/// Desliga o sistema.
/// Args: nenhum
/// Retorno: nunca retorna
pub const SYS_POWEROFF: usize = 0xF2;

/// Escreve na console (serial).
/// Args: (buf_ptr, len)
/// Retorno: bytes escritos
pub const SYS_CONSOLE_WRITE: usize = 0xF3;

/// Lê da console (serial).
/// Args: (buf_ptr, max_len)
/// Retorno: bytes lidos
pub const SYS_CONSOLE_READ: usize = 0xF4;

/// Comandos de debug (apenas em builds debug).
/// Args: (cmd, arg_ptr, arg_len)
/// Retorno: depende do comando
pub const SYS_DEBUG: usize = 0xFF;
