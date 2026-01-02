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
//! | Range     | Categoria        |
//! |-----------|------------------|
//! | 0x01-0x0F | Processo         |
//! | 0x10-0x1F | Memória          |
//! | 0x20-0x2F | Handles          |
//! | 0x30-0x3F | IPC              |
//! | 0x40-0x4F | Gráficos/Input   |
//! | 0x50-0x5F | Tempo            |
//! | 0x60-0x7F | Filesystem       |
//! | 0x80-0x8F | Events           |
//! | 0xF0-0xFF | Sistema/Debug    |

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

/// Obtém o TID da thread atual.
/// Args: nenhum
/// Retorno: tid
pub const SYS_GETTID: usize = 0x07;

/// Cria uma nova thread no processo atual.
/// Args: (entry_ptr, stack_ptr, arg)
/// Retorno: tid ou erro
pub const SYS_THREAD_CREATE: usize = 0x08;

/// Encerra a thread atual.
/// Args: (exit_code)
/// Retorno: Nunca retorna
pub const SYS_THREAD_EXIT: usize = 0x09;

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

/// Altera as proteções de uma região de memória.
/// Args: (addr, size, new_flags)
/// Retorno: 0 ou erro
pub const SYS_MPROTECT: usize = 0x14;

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

/// Suspende a thread até que o valor mude (futex).
/// Args: (addr, expected, timeout_ms)
/// Retorno: 0 ou erro
pub const SYS_FUTEX_WAIT: usize = 0x33;

/// Acorda threads esperando em um futex.
/// Args: (addr, count)
/// Retorno: número de threads acordadas ou erro
pub const SYS_FUTEX_WAKE: usize = 0x34;

/// Cria uma região de memória compartilhada.
/// Args: (size: usize)
/// Retorno: shm_id ou erro
pub const SYS_SHM_CREATE: usize = 0x35;

/// Mapeia uma região SHM no espaço do processo.
/// Args: (shm_id: u64, suggested_addr: usize)
/// Retorno: endereço mapeado ou erro
pub const SYS_SHM_MAP: usize = 0x36;

/// Conecta a uma porta nomeada.
/// Args: (name_ptr, name_len)
/// Retorno: port_id ou erro
pub const SYS_PORT_CONNECT: usize = 0x37;

/// Obtém o tamanho de uma região SHM.
/// Args: (shm_id: u64)
/// Retorno: tamanho em bytes ou erro
pub const SYS_SHM_GET_SIZE: usize = 0x38;

// ============================================================================
// GRÁFICOS / INPUT (0x40 - 0x4F)
// ============================================================================

/// Obtém informações do framebuffer.
/// Args: (out_ptr: *mut FramebufferInfo)
/// Retorno: 0 ou erro
pub const SYS_FB_INFO: usize = 0x40;

/// Escreve pixels no framebuffer.
/// Args: (offset: usize, data_ptr: *const u8, len: usize)
/// Retorno: bytes escritos ou erro
pub const SYS_FB_WRITE: usize = 0x41;

/// Limpa todo o framebuffer com uma cor.
/// Args: (color: u32)
/// Retorno: 0 ou erro
pub const SYS_FB_CLEAR: usize = 0x42;

/// Lê estado do mouse (posição e botões).
/// Args: (out_ptr: *mut MouseState)
/// Retorno: 0 ou erro
pub const SYS_MOUSE_READ: usize = 0x48;

/// Lê eventos de teclado.
/// Args: (out_ptr: *mut KeyEvent, max_events: usize)
/// Retorno: número de eventos ou erro
pub const SYS_KEYBOARD_READ: usize = 0x49;

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

/// Cria um timer do sistema.
/// Retorno: handle do timer ou erro
pub const SYS_TIMER_CREATE: usize = 0x52;

/// Configura/Inicia um timer.
/// Args: (handle, initial_ms, interval_ms)
pub const SYS_TIMER_SET: usize = 0x53;

// ============================================================================
// FILESYSTEM - BÁSICO (0x60 - 0x67)
// Operações fundamentais de I/O
// ============================================================================

/// Abre um arquivo ou diretório.
/// Args: (path_ptr, path_len, flags, mode)
/// Retorno: handle ou erro
/// Flags: O_RDONLY, O_WRONLY, O_RDWR, O_CREATE, O_TRUNC, O_APPEND
pub const SYS_OPEN: usize = 0x60;

/// Lê dados de um arquivo na posição atual.
/// Args: (handle, buf_ptr, len)
/// Retorno: bytes lidos ou erro
pub const SYS_READ: usize = 0x61;

/// Escreve dados em um arquivo na posição atual.
/// Args: (handle, buf_ptr, len)
/// Retorno: bytes escritos ou erro
pub const SYS_WRITE: usize = 0x62;

/// Move posição de leitura/escrita (cursor).
/// Args: (handle, offset, whence)
/// Whence: SEEK_SET=0, SEEK_CUR=1, SEEK_END=2
/// Retorno: nova posição ou erro
pub const SYS_SEEK: usize = 0x63;

/// Lê dados em offset específico (atômico, não move cursor).
/// Args: (handle, buf_ptr, len, offset)
/// Retorno: bytes lidos ou erro
pub const SYS_PREAD: usize = 0x64;

/// Escreve dados em offset específico (atômico, não move cursor).
/// Args: (handle, buf_ptr, len, offset)
/// Retorno: bytes escritos ou erro
pub const SYS_PWRITE: usize = 0x65;

/// Força flush de buffers do handle para o disco.
/// Args: (handle)
/// Retorno: 0 ou erro
pub const SYS_FLUSH: usize = 0x66;

/// Redimensiona um arquivo.
/// Args: (handle, new_size)
/// Retorno: 0 ou erro
pub const SYS_TRUNCATE: usize = 0x67;

// ============================================================================
// FILESYSTEM - METADADOS (0x68 - 0x6B)
// Informações sobre arquivos e permissões
// ============================================================================

/// Obtém informações de arquivo por caminho.
/// Args: (path_ptr, path_len, stat_ptr)
/// Retorno: 0 ou erro
pub const SYS_STAT: usize = 0x68;

/// Obtém informações de arquivo por handle.
/// Args: (handle, stat_ptr)
/// Retorno: 0 ou erro
pub const SYS_FSTAT: usize = 0x69;

/// Altera permissões de um arquivo/diretório.
/// Args: (path_ptr, path_len, mode)
/// Retorno: 0 ou erro
pub const SYS_CHMOD: usize = 0x6A;

/// Altera dono/grupo de um arquivo/diretório.
/// Args: (path_ptr, path_len, uid, gid)
/// Retorno: 0 ou erro
pub const SYS_CHOWN: usize = 0x6B;

// ============================================================================
// FILESYSTEM - DIRETÓRIOS (0x6C - 0x6F)
// Navegação e listagem de diretórios
// ============================================================================

/// Lista entradas de um diretório (batch).
/// Args: (handle, buf_ptr, buf_len)
/// Retorno: bytes escritos no buffer ou erro
/// Buffer contém structs DirEntry sequenciais
pub const SYS_GETDENTS: usize = 0x6C;

/// Cria um diretório.
/// Args: (path_ptr, path_len, mode)
/// Retorno: 0 ou erro
pub const SYS_MKDIR: usize = 0x6D;

/// Remove um diretório vazio.
/// Args: (path_ptr, path_len)
/// Retorno: 0 ou erro
pub const SYS_RMDIR: usize = 0x6E;

/// Obtém diretório de trabalho atual do processo.
/// Args: (buf_ptr, buf_len)
/// Retorno: tamanho do path ou erro
pub const SYS_GETCWD: usize = 0x6F;

// ============================================================================
// FILESYSTEM - MANIPULAÇÃO (0x70 - 0x73)
// Criar, remover e mover arquivos
// ============================================================================

/// Cria um arquivo vazio.
/// Args: (path_ptr, path_len, mode)
/// Retorno: 0 ou erro
pub const SYS_CREATE: usize = 0x70;

/// Remove (desvincula) um arquivo.
/// Args: (path_ptr, path_len)
/// Retorno: 0 ou erro
pub const SYS_UNLINK: usize = 0x71;

/// Renomeia ou move um arquivo/diretório.
/// Args: (old_ptr, old_len, new_ptr, new_len)
/// Retorno: 0 ou erro
pub const SYS_RENAME: usize = 0x72;

/// Cria um hard link.
/// Args: (target_ptr, target_len, link_ptr, link_len)
/// Retorno: 0 ou erro
pub const SYS_LINK: usize = 0x73;

// ============================================================================
// FILESYSTEM - LINKS SIMBÓLICOS (0x74 - 0x76)
// Operações com symlinks
// ============================================================================

/// Cria um link simbólico.
/// Args: (target_ptr, target_len, link_ptr, link_len)
/// Retorno: 0 ou erro
pub const SYS_SYMLINK: usize = 0x74;

/// Lê o destino de um link simbólico.
/// Args: (path_ptr, path_len, buf_ptr, buf_len)
/// Retorno: tamanho do target ou erro
pub const SYS_READLINK: usize = 0x75;

/// Resolve caminho para forma canônica (absoluta, sem symlinks).
/// Args: (path_ptr, path_len, buf_ptr, buf_len)
/// Retorno: tamanho do path resolvido ou erro
pub const SYS_REALPATH: usize = 0x76;

// ============================================================================
// FILESYSTEM - MONTAGEM (0x77 - 0x7A)
// Operações de sistema de arquivos
// ============================================================================

/// Monta um filesystem.
/// Args: (source_ptr, source_len, target_ptr, target_len, fstype_ptr, fstype_len, flags)
/// Retorno: 0 ou erro
pub const SYS_MOUNT: usize = 0x77;

/// Desmonta um filesystem.
/// Args: (target_ptr, target_len, flags)
/// Retorno: 0 ou erro
pub const SYS_UMOUNT: usize = 0x78;

/// Obtém informações do filesystem (espaço livre, total, etc).
/// Args: (path_ptr, path_len, statfs_ptr)
/// Retorno: 0 ou erro
pub const SYS_STATFS: usize = 0x79;

/// Sincroniza todos os buffers de filesystem para disco.
/// Args: nenhum
/// Retorno: 0 ou erro
pub const SYS_SYNC: usize = 0x7A;

// ============================================================================
// FILESYSTEM - AVANÇADO (0x7B - 0x7F)
// Controle e operações especiais
// ============================================================================

/// Controle de dispositivo (operações específicas de driver).
/// Args: (handle, cmd, arg_ptr)
/// Retorno: depende do comando
pub const SYS_IOCTL: usize = 0x7B;

/// Controle de handle (flags, duplicação, etc).
/// Args: (handle, cmd, arg)
/// Retorno: depende do comando
pub const SYS_FCNTL: usize = 0x7C;

/// Lock/unlock de arquivo.
/// Args: (handle, operation)
/// Operation: LOCK_SH, LOCK_EX, LOCK_UN, LOCK_NB
/// Retorno: 0 ou erro
pub const SYS_FLOCK: usize = 0x7D;

/// Verifica permissões de acesso sem abrir.
/// Args: (path_ptr, path_len, mode)
/// Mode: R_OK, W_OK, X_OK, F_OK
/// Retorno: 0 (permitido) ou erro
pub const SYS_ACCESS: usize = 0x7E;

/// Altera diretório de trabalho do processo.
/// Args: (path_ptr, path_len)
/// Retorno: 0 ou erro
pub const SYS_CHDIR: usize = 0x7F;

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
