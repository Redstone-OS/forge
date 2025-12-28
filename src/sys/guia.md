# Guia de Implementação: `sys/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Tipos e definições compartilhadas do sistema. Códigos de erro, tipos fundamentais, estruturas ELF.

---

## 2. ESTRUTURA

```
sys/
├── mod.rs              ✅ JÁ EXISTE
├── error.rs            → Códigos de erro
├── types.rs            → Tipos fundamentais
└── elf.rs              → Estruturas ELF
```

---

## 3. IMPLEMENTAÇÕES

### 3.1 `error.rs`

```rust
//! Códigos de erro do kernel

/// Erro genérico do kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum KernelError {
    /// Sucesso (não é erro)
    Success = 0,
    /// Permissão negada
    PermissionDenied = -1,
    /// Não encontrado
    NotFound = -2,
    /// Já existe
    AlreadyExists = -3,
    /// Sem memória
    OutOfMemory = -4,
    /// Argumento inválido
    InvalidArgument = -5,
    /// Operação não suportada
    NotSupported = -6,
    /// Recurso ocupado
    Busy = -7,
    /// Timeout
    Timeout = -8,
    /// Handle inválido
    InvalidHandle = -9,
    /// Buffer muito pequeno
    BufferTooSmall = -10,
    /// Fim de arquivo
    EndOfFile = -11,
    /// IO Error
    IoError = -12,
    /// Interrompido
    Interrupted = -13,
    /// Novamente (tente de novo)
    Again = -14,
    /// Operação cancelada
    Cancelled = -15,
    /// Erro interno
    Internal = -99,
}

/// Result type do kernel
pub type KernelResult<T> = Result<T, KernelError>;

impl KernelError {
    /// Converte para código numérico
    pub const fn as_code(self) -> i32 {
        self as i32
    }
    
    /// Cria a partir de código
    pub const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Success,
            -1 => Self::PermissionDenied,
            -2 => Self::NotFound,
            -3 => Self::AlreadyExists,
            -4 => Self::OutOfMemory,
            -5 => Self::InvalidArgument,
            -6 => Self::NotSupported,
            -7 => Self::Busy,
            -8 => Self::Timeout,
            -9 => Self::InvalidHandle,
            -10 => Self::BufferTooSmall,
            -11 => Self::EndOfFile,
            -12 => Self::IoError,
            -13 => Self::Interrupted,
            -14 => Self::Again,
            -15 => Self::Cancelled,
            _ => Self::Internal,
        }
    }
}
```

### 3.2 `types.rs`

```rust
//! Tipos fundamentais do sistema

/// Process ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Pid(pub u32);

impl Pid {
    pub const KERNEL: Pid = Pid(0);
    pub const INIT: Pid = Pid(1);
    
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
    
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Thread ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Tid(pub u32);

impl Tid {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
    
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// User ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Uid(pub u32);

impl Uid {
    pub const ROOT: Uid = Uid(0);
    
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Group ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Gid(pub u32);

impl Gid {
    pub const ROOT: Gid = Gid(0);
    
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Offset em arquivo
pub type FileOffset = i64;

/// Tamanho
pub type Size = usize;
```

### 3.3 `elf.rs`

```rust
//! Estruturas ELF para loading de executáveis

/// Magic number ELF
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// Classe ELF
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ElfClass {
    None = 0,
    Elf32 = 1,
    Elf64 = 2,
}

/// Tipo de arquivo ELF
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum ElfType {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    SharedObject = 3,
    Core = 4,
}

/// Header ELF64
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Header {
    pub magic: [u8; 4],
    pub class: u8,
    pub endian: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub _pad: [u8; 7],
    pub elf_type: u16,
    pub machine: u16,
    pub version2: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

/// Tipo de program header
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum PhType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interp = 3,
    Note = 4,
    Phdr = 6,
    Tls = 7,
}

/// Program Header ELF64
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

/// Flags de program header
pub const PF_X: u32 = 1; // Executável
pub const PF_W: u32 = 2; // Escrevível
pub const PF_R: u32 = 4; // Legível

impl Elf64Header {
    /// Verifica se é ELF válido
    pub fn is_valid(&self) -> bool {
        self.magic == ELF_MAGIC && self.class == ElfClass::Elf64 as u8
    }
}
```

---

## 4. DEPENDÊNCIAS

Este módulo NÃO importa de nenhum outro módulo.

Outros módulos PODEM importar deste.

---

## 5. CHECKLIST

- [ ] Todos os tipos usam `#[repr(C)]` ou `#[repr(transparent)]`
- [ ] Códigos de erro são negativos
- [ ] Tipos de ID são newtypes (não aliases)
- [ ] Estruturas ELF são exatamente como especificação
