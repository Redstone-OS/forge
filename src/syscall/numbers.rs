//! # Syscall Numbers Registry
//!
//! Catálogo central de todas as operações suportadas pelo kernel.
//!
//! ## 🎯 Propósito
//! - **Central Authority:** Garante que cada número de syscall seja único e imutável.
//! - **Categorization:** Agrupa syscalls por domínio funcional (Processo, Memória, IO).
//!
//! ## ⚠️ Compatibility Warning
//! **ESTE SISTEMA NÃO É LINUX.**
//! A numeração de syscalls não segue a tabela x86_64 do Linux.
//! Portar software requer recompilação ou uma camada de emulação (Linuxlator) que traduza os números.
//!
//! ### ⚠️ Pontos de Atenção
//! - **Missing Features:** A lista atual é muito curta para um OS real. Faltam syscalls críticas de rede (`socket`, `bind`), sinais (`kill`, `sigaction`) e filesystem (`openat`, `fstat`).
//!
//! ## 🛠️ TODOs
//! - [ ] **TODO: (Architecture)** Reservar range para **Vendor Extensions**.
//!   - *Meta:* Permitir que drivers específicos registrem syscalls dinâmicas (ex: GPU driver).
//! - [ ] **TODO: (Feature)** Adicionar bloco de **Network Syscalls** (0x60-0x7F).
//!
//! --------------------------------------------------------------------------------
//!
//! **ATENÇÃO**: Esta numeração é EXCLUSIVA do Redstone OS.
//! NÃO é compatível com Linux, POSIX ou qualquer outro sistema.
//!
//! # Organização
//!
//! | Range     | Categoria     |
//! |-----------|---------------|
//! | 0x01-0x0F | Processo      |
//! | 0x10-0x1F | Memória       |
//! | 0x20-0x2F | Handles       |
//! | 0x30-0x3F | IPC           |
//! | 0x40-0x4F | IO            |
//! | 0x50-0x5F | Tempo         |
//! | 0xE0-0xEF | Async IO      |
//! | 0xF0-0xFF | Sistema/Debug |

// ============================================================================
// PROCESSO (0x01 - 0x0F)
// ============================================================================

/// Encerra o processo atual.
/// Args: (exit_code: i32)
/// Retorno: Nunca retorna
pub const SYS_EXIT: usize = 0x01;

/// Cria um novo processo.
/// Args: (image_ptr, image_len, args_ptr, args_len)
/// Retorno: task_id ou erro
pub const SYS_SPAWN: usize = 0x02;

/// Espera um processo terminar.
/// Args: (task_id, timeout_ms)
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
/// Args: (task_id, out_ptr)
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

/// Cria um handle para um objeto.
/// Args: (type, object_ptr, object_len, rights)
/// Retorno: handle ou erro
pub const SYS_HANDLE_CREATE: usize = 0x20;

/// Duplica um handle com direitos reduzidos.
/// Args: (handle, new_rights)
/// Retorno: new_handle ou erro
pub const SYS_HANDLE_DUP: usize = 0x21;

/// Fecha um handle.
/// Args: (handle)
/// Retorno: 0 ou erro
pub const SYS_HANDLE_CLOSE: usize = 0x22;

/// Verifica direitos de um handle.
/// Args: (handle, rights_mask)
/// Retorno: 1 (tem) ou 0 (não tem)
pub const SYS_CHECK_RIGHTS: usize = 0x23;

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

/// Verifica mensagem sem remover.
/// Args: (port_handle, buf_ptr, buf_len)
/// Retorno: bytes disponíveis ou erro
pub const SYS_PEEK_MSG: usize = 0x33;

// ============================================================================
// IO (0x40 - 0x4F)
// ============================================================================

/// Leitura vetorizada de um handle.
/// Args: (handle, iov_ptr, iov_cnt, flags)
/// Retorno: bytes lidos ou erro
pub const SYS_READV: usize = 0x40;

/// Escrita vetorizada em um handle.
/// Args: (handle, iov_ptr, iov_cnt, flags)
/// Retorno: bytes escritos ou erro
pub const SYS_WRITEV: usize = 0x41;

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

/// Obtém tempo monotônico em ticks.
/// Args: nenhum
/// Retorno: ticks desde boot
pub const SYS_MONOTONIC: usize = 0x52;

// ============================================================================
// ASYNC IO - RESERVADO PARA FUTURO (0xE0 - 0xEF)
// ============================================================================

/// Cria um ring de IO assíncrono.
/// Args: (flags, params_ptr)
/// Retorno: ring_id ou erro
pub const SYS_CREATE_RING: usize = 0xE0;

/// Submete operações ao ring.
/// Args: (ring_id, submit_ptr, submit_cnt)
/// Retorno: quantidade submetida ou erro
pub const SYS_SUBMIT_IO: usize = 0xE1;

/// Espera completions do ring.
/// Args: (ring_id, timeout_ms, min_complete)
/// Retorno: quantidade completada ou erro
pub const SYS_WAIT_IO: usize = 0xE2;

/// Fecha um ring de IO.
/// Args: (ring_id)
/// Retorno: 0 ou erro
pub const SYS_CLOSE_RING: usize = 0xE3;

// ============================================================================
// SISTEMA / DEBUG (0xF0 - 0xFF)
// ============================================================================

/// Obtém informações do sistema.
/// Args: (out_ptr, out_len)
/// Retorno: bytes escritos ou erro
pub const SYS_SYSINFO: usize = 0xF0;

/// Comandos de debug (apenas em builds debug).
/// Args: (cmd, arg_ptr, arg_len)
/// Retorno: depende do comando
pub const SYS_DEBUG: usize = 0xFF;

// ============================================================================
// DEBUG COMMANDS
// ============================================================================

/// Comandos para SYS_DEBUG
pub mod debug_cmd {
    /// Imprime string no log do kernel
    pub const KPRINT: u32 = 0x01;
    /// Dump de registradores
    pub const DUMP_REGS: u32 = 0x02;
    /// Dump de memória
    pub const DUMP_MEM: u32 = 0x03;
    /// Breakpoint (para debugger)
    pub const BREAKPOINT: u32 = 0x04;
}
