# 📂 Sistema de Arquivos (FS)

> **Módulo**: `forge/src/fs` | **Versão**: 0.1.2 | **Status**: 🟢 Operacional  
> **Última atualização**: Janeiro 2026

O subsistema de Filesystem do **RedstoneOS** implementa uma arquitetura moderna de camadas que abstrai dispositivos de armazenamento em uma hierarquia unificada. O design prioriza **modularidade**, **extensibilidade** e **performance**.

---

## 📋 Índice

1. [Arquitetura Geral](#-arquitetura-geral)
2. [Syscalls de Filesystem](#-syscalls-de-filesystem)
3. [Virtual File System (VFS)](#-virtual-file-system-vfs)
4. [Backends de Filesystem](#-backends-de-filesystem)
5. [Hierarquia de Diretórios](#-hierarquia-de-diretórios)
6. [Estrutura do Código](#-estrutura-do-código)
7. [Fluxo de Operações](#-fluxo-de-operações)
8. [Tipos e Estruturas](#-tipos-e-estruturas)
9. [Roadmap](#-roadmap)

---

## 🏛️ Arquitetura Geral

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                    USERSPACE                                    │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐   │
│  │  Shell  │  │  Apps   │  │ Firefly │  │ Editor  │  │  Games  │  │   ...   │   │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘   │
│       │            │            │            │            │            │        │
│       └────────────┴────────────┴─────┬──────┴────────────┴────────────┘        │
│                                       ↓                                         │
│                              libredstone (librs)                                │
│                          open() read() stat() etc                               │
└───────────────────────────────────────┼─────────────────────────────────────────┘
                                        │ syscall
════════════════════════════════════════╪═════════════════════════════════════════
                                        ↓
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              KERNEL SPACE                                       │
│                                                                                 │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                      SYSCALL LAYER (syscall/fs)                          │   │
│  │                                                                          │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐  │   │
│  │  │   io    │ │  meta   │ │   dir   │ │  file   │ │  mount  │ │  ctrl  │  │   │
│  │  │ 0x60-67 │ │ 0x68-6B │ │ 0x6C-6F │ │ 0x70-73 │ │ 0x77-7A │ │ 0x7B-7F│  │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └───┬────┘  │   │
│  │       └───────────┴───────────┴─────┬─────┴───────────┴──────────┘       │   │
│  └─────────────────────────────────────┼────────────────────────────────────┘   │
│                                        ↓                                        │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                      VFS - Virtual File System                           │   │
│  │                                                                          │   │
│  │   ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐         │   │
│  │   │   Inodes   │  │   Files    │  │  Dentries  │  │   Mounts   │         │   │
│  │   │  (nodes)   │  │  (handles) │  │   (cache)  │  │  (points)  │         │   │
│  │   └─────┬──────┘  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘         │   │
│  │         └───────────────┴───────────────┴───────────────┘                │   │
│  │                                  │                                       │   │
│  │                         Path Resolution                                  │   │
│  │                      /system/core/* → InitRAMFS                          │   │
│  │                      /system/services/* → FAT                            │   │
│  │                      /apps/* → FAT                                       │   │
│  └──────────────────────────────────┼───────────────────────────────────────┘   │
│                                     ↓                                           │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                          FILESYSTEM BACKENDS                             │   │
│  │                                                                          │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │   │
│  │   │  InitRAMFS  │  │    FAT      │  │    RFS      │  │   DevFS     │     │   │
│  │   │   (TAR)     │  │  (16/32)    │  │  (futuro)   │  │  (futuro)   │     │   │
│  │   │   Estável   │  │  Read-Only  │  │   Projeto   │  │  Planejado  │     │   │ 
│  │   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘     │   │
│  └──────────────────────────────────┼───────────────────────────────────────┘   │
│                                     ↓                                           │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                     BLOCK DEVICE LAYER                                   │   │
│  │                                                                          │   │
│  │   trait BlockDevice { read_block(), write_block() }                      │   │
│  │                                                                          │   │
│  │   ┌──────────┐  ┌────────────┐  ┌──────────┐  ┌────────────┐             │   │
│  │   │  VirtIO  │  │   ATA      │  │  RAMDisk │  │   NVMe     │             │   │
│  │   │  Ativo   │  │  Planejado │  │   Ativo  │  │  Planejado │             │   │ 
│  │   └──────────┘  └────────────┘  └──────────┘  └────────────┘             │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 📞 Syscalls de Filesystem

O RedstoneOS expõe **32 syscalls** de filesystem organizadas em **7 categorias**. Esta é uma API moderna e completa.

### Visão Geral por Categoria

| Range | Categoria | Syscalls | Status |
|-------|-----------|----------|--------|
| `0x60-0x67` | **I/O Básico** | open, read, write, seek, pread, pwrite, flush, truncate | 🟢 |
| `0x68-0x6B` | **Metadados** | stat, fstat, chmod, chown | 🟡 |
| `0x6C-0x6F` | **Diretórios** | getdents, mkdir, rmdir, getcwd | 🟢 |
| `0x70-0x73` | **Manipulação** | create, unlink, rename, link | ⚪ |
| `0x74-0x76` | **Symlinks** | symlink, readlink, realpath | ⚪ |
| `0x77-0x7A` | **Montagem** | mount, umount, statfs, sync | ⚪ |
| `0x7B-0x7F` | **Controle** | ioctl, fcntl, flock, access, chdir | 🟡 |

**Legenda**: 🟢 Funcional | 🟡 Parcial | ⚪ Stub

---

### 📖 Referência Completa de Syscalls

#### 🔹 I/O Básico (0x60-0x67)

<details>
<summary><b>SYS_OPEN (0x60)</b> - Abre arquivo ou diretório</summary>

```rust
fn sys_open(path_ptr: usize, path_len: usize, flags: u32, mode: u32) -> Result<usize, SysError>
```

| Parâmetro | Tipo | Descrição |
|-----------|------|-----------|
| `path_ptr` | `*const u8` | Ponteiro para o caminho |
| `path_len` | `usize` | Tamanho do caminho em bytes |
| `flags` | `u32` | Flags de abertura (ver abaixo) |
| `mode` | `u32` | Permissões para criação (quando O_CREATE) |

**Flags Suportados:**
| Flag | Valor | Descrição |
|------|-------|-----------|
| `O_RDONLY` | 0 | Somente leitura |
| `O_WRONLY` | 1 | Somente escrita |
| `O_RDWR` | 2 | Leitura e escrita |
| `O_CREATE` | 0x0100 | Criar se não existir |
| `O_TRUNC` | 0x0200 | Truncar arquivo existente |
| `O_APPEND` | 0x0400 | Append mode |
| `O_EXCL` | 0x0800 | Falhar se existir (com O_CREATE) |
| `O_DIRECTORY` | 0x1000 | Abrir apenas diretórios |

**Retorno:** Handle do arquivo (≥ 3) ou código de erro negativo.

**Exemplo:**
```rust
let handle = syscall!(SYS_OPEN, path.as_ptr(), path.len(), O_RDONLY, 0)?;
```
</details>

<details>
<summary><b>SYS_READ (0x61)</b> - Lê dados do arquivo</summary>

```rust
fn sys_read(handle: u32, buf_ptr: usize, len: usize) -> Result<usize, SysError>
```

| Parâmetro | Tipo | Descrição |
|-----------|------|-----------|
| `handle` | `u32` | Handle retornado por open() |
| `buf_ptr` | `*mut u8` | Buffer de destino |
| `len` | `usize` | Bytes máximos a ler |

**Retorno:** Número de bytes lidos, 0 para EOF, ou erro.
</details>

<details>
<summary><b>SYS_WRITE (0x62)</b> - Escreve dados no arquivo</summary>

```rust
fn sys_write(handle: u32, buf_ptr: usize, len: usize) -> Result<usize, SysError>
```

**Status:** ⚪ Não implementado (FAT é read-only)
</details>

<details>
<summary><b>SYS_SEEK (0x63)</b> - Move cursor de leitura/escrita</summary>

```rust
fn sys_seek(handle: u32, offset: i64, whence: u32) -> Result<usize, SysError>
```

| Whence | Valor | Descrição |
|--------|-------|-----------|
| `SEEK_SET` | 0 | Do início do arquivo |
| `SEEK_CUR` | 1 | Da posição atual |
| `SEEK_END` | 2 | Do fim do arquivo |

**Retorno:** Nova posição absoluta.
</details>

<details>
<summary><b>SYS_PREAD (0x64)</b> - Lê em offset específico</summary>

```rust
fn sys_pread(handle: u32, buf_ptr: usize, len: usize, offset: u64) -> Result<usize, SysError>
```

Leitura atômica que **não move o cursor** do handle. Ideal para I/O paralelo.
</details>

<details>
<summary><b>SYS_PWRITE (0x65)</b> - Escreve em offset específico</summary>

```rust
fn sys_pwrite(handle: u32, buf_ptr: usize, len: usize, offset: u64) -> Result<usize, SysError>
```

**Status:** ⚪ Não implementado
</details>

<details>
<summary><b>SYS_FLUSH (0x66)</b> - Força flush de buffers</summary>

```rust
fn sys_flush(handle: u32) -> Result<usize, SysError>
```

Força todos os dados pendentes a serem gravados no disco.
</details>

<details>
<summary><b>SYS_TRUNCATE (0x67)</b> - Redimensiona arquivo</summary>

```rust
fn sys_truncate(handle: u32, new_size: u64) -> Result<usize, SysError>
```

**Status:** ⚪ Não implementado
</details>

---

#### 🔹 Metadados (0x68-0x6B)

<details>
<summary><b>SYS_STAT (0x68)</b> - Info de arquivo por caminho</summary>

```rust
fn sys_stat(path_ptr: usize, path_len: usize, stat_ptr: usize) -> Result<usize, SysError>
```

Preenche a estrutura `FileStat` no ponteiro fornecido.
</details>

<details>
<summary><b>SYS_FSTAT (0x69)</b> - Info de arquivo por handle</summary>

```rust
fn sys_fstat(handle: u32, stat_ptr: usize) -> Result<usize, SysError>
```

Mais eficiente que stat() quando o arquivo já está aberto.
</details>

<details>
<summary><b>SYS_CHMOD (0x6A)</b> / <b>SYS_CHOWN (0x6B)</b></summary>

**Status:** ⚪ Não implementado (requer sistema de permissões)
</details>

---

#### 🔹 Diretórios (0x6C-0x6F)

<details>
<summary><b>SYS_GETDENTS (0x6C)</b> - Lista entradas de diretório</summary>

```rust
fn sys_getdents(handle: u32, buf_ptr: usize, buf_len: usize) -> Result<usize, SysError>
```

Retorna múltiplas entradas de diretório em formato binário:

```
┌──────────────────────────────────────────────┐
│ DirEntry Header (12 bytes)                   │
├──────────────────────────────────────────────┤
│ ino (u64)      │ Número do inode (ou 0)      │
│ rec_len (u16)  │ Tamanho total desta entrada │
│ file_type (u8) │ 1=regular, 2=directory, ... │
│ name_len (u8)  │ Tamanho do nome             │
├──────────────────────────────────────────────┤
│ name[name_len] │ Nome do arquivo (sem \0)    │
│ padding        │ Alinhado em 8 bytes         │
└──────────────────────────────────────────────┘
```

**Uso:** Chamar repetidamente até retornar 0.
</details>

<details>
<summary><b>SYS_GETCWD (0x6F)</b> - Obtém diretório atual</summary>

```rust
fn sys_getcwd(buf_ptr: usize, buf_len: usize) -> Result<usize, SysError>
```

Retorna o tamanho do path incluindo null terminator.
</details>

<details>
<summary><b>SYS_MKDIR (0x6D)</b> / <b>SYS_RMDIR (0x6E)</b></summary>

**Status:** ⚪ Não implementado (requer FAT write)
</details>

---

#### 🔹 Controle (0x7B-0x7F)

<details>
<summary><b>SYS_CHDIR (0x7F)</b> - Muda diretório de trabalho</summary>

```rust
fn sys_chdir(path_ptr: usize, path_len: usize) -> Result<usize, SysError>
```

Muda o CWD do processo atual. Usado pelo comando `cd`.
</details>

<details>
<summary><b>SYS_ACCESS (0x7E)</b> - Verifica permissões</summary>

```rust
fn sys_access(path_ptr: usize, path_len: usize, mode: u32) -> Result<usize, SysError>
```

| Mode | Valor | Descrição |
|------|-------|-----------|
| `F_OK` | 0 | Verifica existência |
| `X_OK` | 1 | Verifica execução |
| `W_OK` | 2 | Verifica escrita |
| `R_OK` | 4 | Verifica leitura |
</details>

---

## 📁 Virtual File System (VFS)

O VFS é o **coração** do subsistema de arquivos. Ele unifica múltiplos backends em uma árvore única.

### Estruturas Fundamentais

```rust
/// Inode - Representa um objeto no disco
struct Inode {
    ino: u64,           // Número único
    file_type: FileType, // Regular, Directory, Symlink, etc
    mode: u16,          // Permissões (rwxrwxrwx)
    size: u64,          // Tamanho em bytes
    nlink: u32,         // Contagem de hard links
    uid: u32,           // ID do dono
    gid: u32,           // ID do grupo
    atime: u64,         // Tempo de acesso
    mtime: u64,         // Tempo de modificação
    ctime: u64,         // Tempo de criação
    ops: &'static dyn InodeOps,  // Operações específicas do backend
}

/// File Handle - Representa arquivo aberto por processo
struct FileHandle {
    path: String,       // Path completo
    file_type: FileType,
    flags: OpenFlags,   // Flags de abertura
    offset: u64,        // Cursor atual
    size: u64,          // Tamanho do arquivo
    first_cluster: u32, // (FAT) Primeiro cluster
    dir_index: usize,   // (Dir) Índice para getdents
}
```

### Roteamento de Paths

O VFS roteia requisições baseado no prefixo do caminho:

```rust
match path {
    "/system/core/*"     => InitRAMFS,  // Supervisor e core
    "/system/services/*" => FAT,        // Serviços do disco
    "/apps/*"            => FAT,        // Aplicativos
    "/devices/*"         => DevFS,      // [Futuro] Dispositivos
    "/runtime/*"         => TmpFS,      // [Futuro] Volátil
    _                    => FAT,        // Default
}
```

---

## 💾 Backends de Filesystem

### InitRAMFS (Boot)

| Aspecto | Detalhe |
|---------|---------|
| **Formato** | TAR (POSIX ustar) |
| **Propósito** | Bootstrap antes dos drivers de disco |
| **Conteúdo** | `/system/core/supervisor` |
| **Características** | Read-only, em memória, zero I/O de disco |

```rust
// Uso interno
let data = initramfs::lookup_file("/system/core/supervisor")?;
```

### FAT (Disco)

| Aspecto | Detalhe |
|---------|---------|
| **Formatos** | FAT12, FAT16, FAT32 |
| **Detecção** | Automática via BPB |
| **Partições** | Suporte a MBR |
| **Status** | Read-only (escrita planejada) |

**Capacidades Atuais:**
- ✅ Leitura de arquivos
- ✅ Navegação de diretórios
- ✅ Suporte a nomes longos (LFN)
- ✅ Detecção automática de MBR/partições
- ⚪ Escrita de arquivos
- ⚪ Criação de diretórios

```rust
// Funções públicas
fat::read_file("/apps/hello") -> Option<Vec<u8>>
fat::list_directory("/system/services") -> Option<Vec<PublicDirEntry>>
```

### RFS - Redstone File System (Futuro)

Sistema de arquivos nativo planejado com recursos enterprise:

```
┌─────────────────────────────────────────────────────┐
│                    RFS Stack                        │
├─────────────────────────────────────────────────────┤
│  ZPL (POSIX Layer)      │ Interface VFS             │
├─────────────────────────────────────────────────────┤
│  DMU (Data Management)  │ Transações, COW, Snapshots│
├─────────────────────────────────────────────────────┤
│  ARC (Cache)            │ Adaptive Replacement Cache│
├─────────────────────────────────────────────────────┤
│  SPA (Pool Allocator)   │ RAID-Z, Espelhamento     │
└─────────────────────────────────────────────────────┘
```

---

## 📂 Hierarquia de Diretórios

```
/
├── system/          🔒 Read-only - Sistema operacional
│   ├── core/        ├── Kernel e supervisor (InitRAMFS)
│   ├── services/    ├── Serviços do sistema (FAT)
│   └── manifests/   └── Metadados de pacotes
│
├── apps/            📦 Aplicativos instalados
│
├── users/           👤 Dados por usuário
│   └── <username>/  ├── Configurações e arquivos pessoais
│
├── devices/         🔌 Dispositivos virtuais (DevFS)
│   ├── fb0          ├── Framebuffer
│   ├── ttyS0        ├── Console serial
│   └── null         └── /dev/null
│
├── volumes/         💿 Pontos de montagem
│
├── runtime/         ⚡ Estado volátil (tmpfs)
│   └── Limpo a cada boot
│
├── state/           💾 Configurações persistentes
│
├── data/            📊 Dados globais de aplicativos
│
├── net/             🌐 Namespace de rede (futuro)
│
├── snapshots/       📸 Histórico do sistema (RFS)
│
└── boot/            🚀 Bootloader e kernel
```

---

## 🗂️ Estrutura do Código

### Módulo Principal (`src/fs/`)

```
fs/
├── mod.rs           # Inicialização e re-exports
├── vfs/             # Virtual File System
│   ├── mod.rs       # Roteamento e read_file()
│   ├── inode.rs     # Inodes, FileType, InodeOps
│   ├── file.rs      # File handles, FileOps
│   ├── path.rs      # Normalização de paths
│   ├── dentry.rs    # Cache de dentries
│   └── mount.rs     # Pontos de montagem
├── fat/             # Driver FAT
│   ├── mod.rs       # Montagem, read_file, list_directory
│   ├── bpb.rs       # BIOS Parameter Block parser
│   ├── dir.rs       # Navegação de diretórios
│   └── file.rs      # Leitura de arquivos
├── initramfs/       # Initial RAM filesystem
│   └── mod.rs       # Parser TAR
├── rfs/             # [Futuro] Redstone File System
│   ├── spa.rs       # Storage Pool Allocator
│   ├── dmu.rs       # Data Management Unit
│   ├── zpl.rs       # POSIX Layer
│   └── arc.rs       # Adaptive Replacement Cache
└── devices/         # [Futuro] Device filesystem
    └── mod.rs
```

### Módulo de Syscalls (`src/syscall/fs/`)

```
syscall/fs/
├── mod.rs           # Módulo principal, re-exports
├── types.rs         # OpenFlags, FileStat, DirEntry, etc
├── handle.rs        # Gerenciamento de file handles
├── io.rs            # open, read, write, seek, pread, pwrite, flush, truncate
├── meta.rs          # stat, fstat, chmod, chown
├── dir.rs           # getdents, mkdir, rmdir, getcwd
├── file.rs          # create, unlink, rename, link
├── link.rs          # symlink, readlink, realpath
├── mount.rs         # mount, umount, statfs, sync
└── ctrl.rs          # ioctl, fcntl, flock, access, chdir
```

---

## 🔄 Fluxo de Operações

### Exemplo: `ls /apps`

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. Shell chama: open("/apps", O_DIRECTORY)                      │
└────────────────────────────────┬────────────────────────────────┘
                                 ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. syscall/fs/io.rs::sys_open()                                 │
│    - Valida ponteiros                                           │
│    - Chama lookup_directory("/apps")                            │
│    - Cria FileHandle com file_type=Directory                    │
│    - Retorna handle (ex: 5)                                     │
└────────────────────────────────┬────────────────────────────────┘
                                 ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. Shell chama: getdents(5, buffer, 4096)                       │
└────────────────────────────────┬────────────────────────────────┘
                                 ↓
┌─────────────────────────────────────────────────────────────────┐
│ 4. syscall/fs/dir.rs::sys_getdents()                            │
│    - Obtém FileHandle do handle 5                               │
│    - Chama list_directory("/apps")                              │
│    - Roteia para FAT: fat::list_directory("apps")               │
└────────────────────────────────┬────────────────────────────────┘
                                 ↓
┌─────────────────────────────────────────────────────────────────┐
│ 5. fat/mod.rs::list_directory()                                 │
│    - Localiza cluster do diretório "apps"                       │
│    - Lê entradas do diretório (32 bytes cada)                   │
│    - Parseia nomes 8.3 e LFN                                    │
│    - Retorna Vec<PublicDirEntry>                                │
└────────────────────────────────┬────────────────────────────────┘
                                 ↓
┌─────────────────────────────────────────────────────────────────┐
│ 6. Shell recebe buffer com DirEntry structs                     │
│    - Formata e imprime: "hello.elf  editor.elf  game.elf"      │
└─────────────────────────────────────────────────────────────────┘
```

### Exemplo: `cat /apps/hello.txt`

```
open("/apps/hello.txt", O_RDONLY)  →  handle=6
    ↓
fstat(6, &stat)  →  stat.size = 1024
    ↓
read(6, buffer, 1024)  →  bytes_read = 1024
    ↓
    VFS::read_file("/apps/hello.txt")
        ↓
    FAT::read_file("apps/hello.txt")
        ↓
    BlockDevice::read_block(cluster_lba)
    ↓
Shell imprime conteúdo
    ↓
close(6)  →  via SYS_HANDLE_CLOSE (0x21)
```

---

## 📊 Tipos e Estruturas

### FileStat (48 bytes)

```rust
#[repr(C)]
pub struct FileStat {
    pub file_type: u8,      // 0=unknown, 1=regular, 2=directory, ...
    pub mode: u16,          // Permissões (octal)
    pub _pad: u8,
    pub size: u64,          // Tamanho em bytes
    pub nlink: u32,         // Hard links
    pub uid: u32,           // User ID
    pub gid: u32,           // Group ID
    pub _pad2: u32,
    pub atime: u64,         // Access time (ms desde epoch)
    pub mtime: u64,         // Modification time
    pub ctime: u64,         // Creation time
}
```

### DirEntry (variável, alinhado em 8 bytes)

```rust
#[repr(C, packed)]
pub struct DirEntryHeader {
    pub ino: u64,           // Número do inode
    pub rec_len: u16,       // Tamanho total desta entrada
    pub file_type: u8,      // Tipo de arquivo
    pub name_len: u8,       // Tamanho do nome
    // name: [u8; name_len]  // Nome segue imediatamente
    // padding               // Até próximo múltiplo de 8
}
```

### OpenFlags

```rust
pub struct OpenFlags(pub u32);

impl OpenFlags {
    pub const O_RDONLY: u32 = 0;
    pub const O_WRONLY: u32 = 1;
    pub const O_RDWR: u32 = 2;
    pub const O_CREATE: u32 = 0x0100;
    pub const O_TRUNC: u32 = 0x0200;
    pub const O_APPEND: u32 = 0x0400;
    pub const O_EXCL: u32 = 0x0800;
    pub const O_DIRECTORY: u32 = 0x1000;
}
```

---

## 🗺️ Roadmap

### Fase 1: Navegação Completa ✅
- [x] `open()` para arquivos e diretórios
- [x] `read()` para arquivos
- [x] `getdents()` para listagem
- [x] `stat()` / `fstat()`
- [x] `chdir()` / `getcwd()`
- [x] `seek()` / `pread()`

### Fase 2: Escrita Básica
- [ ] `write()` no FAT
- [ ] `create()` para novos arquivos
- [ ] `mkdir()` / `rmdir()`
- [ ] `unlink()` para deletar
- [ ] `truncate()` para redimensionar

### Fase 3: Recursos Avançados
- [ ] Cache de blocos em memória
- [ ] DevFS (`/devices/fb0`, `/devices/input`)
- [ ] TmpFS para `/runtime`
- [ ] Mount dinâmico de partições

### Fase 4: RFS Native
- [ ] SPA - Storage Pool Allocator
- [ ] COW (Copy-on-Write) básico
- [ ] Snapshots instantâneos
- [ ] Checksums de integridade

---

## 🔗 Ver Também

- [`SYSCALL.md`](./SYSCALL.md) - Documentação completa de syscalls
- [`DRIVERS.md`](./DRIVERS.md) - Drivers de bloco (VirtIO, ATA)
- [`MM.md`](./MM.md) - Gerenciamento de memória

---

<div align="center">
<i>Forge Kernel — RedstoneOS Filesystem Subsystem</i><br>
<i>Última atualização: Janeiro 2026</i>
</div>
