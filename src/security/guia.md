# Guia de Implementação: `security/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Segurança baseada em Capabilities. Controle de acesso via tokens, não identidade.

---

## 2. ESTRUTURA

```
security/
├── mod.rs              ✅ JÁ EXISTE
├── capability/
│   ├── mod.rs
│   ├── cap.rs          → Capability struct
│   ├── cspace.rs       → Capability Space
│   └── rights.rs       → Rights bitflags
├── credentials/
│   ├── mod.rs
│   └── cred.rs         → Credenciais de processo
├── audit/
│   ├── mod.rs
│   └── audit.rs        → Logging de segurança
└── sandbox/
    ├── mod.rs
    └── namespace.rs    → Namespaces
```

---

## 3. REGRAS

### ❌ NUNCA:
- Confiar em identidade (UID) para autorização
- Permitir acesso sem capability válida
- Expor ponteiros para userspace

### ✅ SEMPRE:
- Verificar rights antes de operação
- Usar handles opacos (u32)
- Auditar operações de segurança

---

## 4. IMPLEMENTAÇÕES

### 4.1 `capability/rights.rs`

```rust
//! Direitos de capability

/// Direitos que uma capability pode conceder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CapRights(u32);

impl CapRights {
    pub const NONE: Self = Self(0);
    
    /// Pode ler dados
    pub const READ: Self = Self(1 << 0);
    /// Pode escrever dados
    pub const WRITE: Self = Self(1 << 1);
    /// Pode executar
    pub const EXECUTE: Self = Self(1 << 2);
    /// Pode duplicar o handle
    pub const DUPLICATE: Self = Self(1 << 3);
    /// Pode transferir via IPC
    pub const TRANSFER: Self = Self(1 << 4);
    /// Pode criar capabilities derivadas
    pub const GRANT: Self = Self(1 << 5);
    /// Pode revogar capabilities derivadas
    pub const REVOKE: Self = Self(1 << 6);
    /// Pode esperar em evento
    pub const WAIT: Self = Self(1 << 7);
    /// Pode sinalizar evento
    pub const SIGNAL: Self = Self(1 << 8);
    
    /// Todos os direitos de dados
    pub const RW: Self = Self(Self::READ.0 | Self::WRITE.0);
    /// Todos os direitos
    pub const ALL: Self = Self(0x1FF);
    
    /// Verifica se tem direito específico
    #[inline]
    pub const fn has(self, right: Self) -> bool {
        (self.0 & right.0) == right.0
    }
    
    /// União de direitos
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    
    /// Interseção de direitos
    #[inline]
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
    
    /// Remove direitos
    #[inline]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}
```

### 4.2 `capability/cap.rs`

```rust
//! Capability - token de acesso

use super::rights::CapRights;

/// Tipo de objeto que a capability referencia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CapType {
    /// Slot vazio
    Null = 0,
    /// Memória virtual (VMO)
    Memory = 1,
    /// Porta IPC
    Port = 2,
    /// Canal IPC
    Channel = 3,
    /// Thread
    Thread = 4,
    /// Processo
    Process = 5,
    /// Evento
    Event = 6,
    /// Timer
    Timer = 7,
    /// IRQ
    Irq = 8,
    /// Região MMIO
    Mmio = 9,
    /// Container de capabilities (CNode)
    CNode = 10,
}

/// Uma capability é um token unforgeable de acesso
#[derive(Debug, Clone)]
pub struct Capability {
    /// Tipo do objeto referenciado
    pub cap_type: CapType,
    /// Direitos concedidos
    pub rights: CapRights,
    /// Referência ao objeto (índice interno)
    pub object_ref: u64,
    /// Badge para identificação em IPC
    pub badge: u64,
    /// Generation counter (para revocação)
    pub generation: u32,
}

impl Capability {
    /// Cria capability nula
    pub const fn null() -> Self {
        Self {
            cap_type: CapType::Null,
            rights: CapRights::NONE,
            object_ref: 0,
            badge: 0,
            generation: 0,
        }
    }
    
    /// Cria nova capability
    pub const fn new(cap_type: CapType, rights: CapRights, object_ref: u64) -> Self {
        Self {
            cap_type,
            rights,
            object_ref,
            badge: 0,
            generation: 0,
        }
    }
    
    /// Verifica se é válida
    pub const fn is_valid(&self) -> bool {
        !matches!(self.cap_type, CapType::Null)
    }
    
    /// Cria capability derivada com menos direitos
    pub fn derive(&self, new_rights: CapRights) -> Option<Self> {
        // Só pode reduzir direitos
        if !self.rights.has(CapRights::GRANT) {
            return None;
        }
        
        // Nova capability tem apenas direitos que origem tinha
        let reduced_rights = new_rights.intersect(self.rights);
        
        Some(Self {
            cap_type: self.cap_type,
            rights: reduced_rights.without(CapRights::GRANT),
            object_ref: self.object_ref,
            badge: self.badge,
            generation: self.generation,
        })
    }
}

/// Handle opaco para userspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CapHandle(u32);

impl CapHandle {
    pub const INVALID: Self = Self(0);
    
    pub const fn new(index: u32) -> Self {
        Self(index)
    }
    
    pub const fn as_u32(self) -> u32 {
        self.0
    }
    
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}
```

### 4.3 `capability/cspace.rs`

```rust
//! Capability Space - tabela por processo

use super::{Capability, CapHandle, CapType, CapRights};
use crate::sync::Spinlock;

/// Tamanho máximo do CSpace
const CSPACE_SIZE: usize = 256;

/// CSpace - tabela de capabilities por processo
pub struct CSpace {
    /// Slots de capabilities
    slots: [Option<Capability>; CSPACE_SIZE],
    /// Próximo slot livre
    next_free: usize,
    /// Generation counter global
    generation: u32,
}

impl CSpace {
    /// Cria CSpace vazio
    pub const fn new() -> Self {
        const NONE: Option<Capability> = None;
        Self {
            slots: [NONE; CSPACE_SIZE],
            next_free: 1, // Slot 0 é reservado (INVALID)
            generation: 1,
        }
    }
    
    /// Insere capability e retorna handle
    pub fn insert(&mut self, cap: Capability) -> Option<CapHandle> {
        // Procurar slot livre
        for i in self.next_free..CSPACE_SIZE {
            if self.slots[i].is_none() {
                self.slots[i] = Some(cap);
                self.next_free = i + 1;
                return Some(CapHandle::new(i as u32));
            }
        }
        
        // Tentar desde o início
        for i in 1..self.next_free {
            if self.slots[i].is_none() {
                self.slots[i] = Some(cap);
                return Some(CapHandle::new(i as u32));
            }
        }
        
        None // CSpace cheio
    }
    
    /// Busca capability por handle
    pub fn lookup(&self, handle: CapHandle) -> Option<&Capability> {
        let index = handle.as_u32() as usize;
        if index >= CSPACE_SIZE {
            return None;
        }
        self.slots[index].as_ref()
    }
    
    /// Busca capability mutável
    pub fn lookup_mut(&mut self, handle: CapHandle) -> Option<&mut Capability> {
        let index = handle.as_u32() as usize;
        if index >= CSPACE_SIZE {
            return None;
        }
        self.slots[index].as_mut()
    }
    
    /// Remove capability
    pub fn remove(&mut self, handle: CapHandle) -> Option<Capability> {
        let index = handle.as_u32() as usize;
        if index >= CSPACE_SIZE {
            return None;
        }
        
        let cap = self.slots[index].take();
        if cap.is_some() && index < self.next_free {
            self.next_free = index;
        }
        cap
    }
    
    /// Duplica capability
    pub fn duplicate(&mut self, handle: CapHandle) -> Option<CapHandle> {
        let cap = self.lookup(handle)?.clone();
        
        if !cap.rights.has(CapRights::DUPLICATE) {
            return None;
        }
        
        self.insert(cap)
    }
    
    /// Verifica se handle tem direito específico
    pub fn check_rights(&self, handle: CapHandle, required: CapRights) -> bool {
        match self.lookup(handle) {
            Some(cap) => cap.rights.has(required),
            None => false,
        }
    }
}

/// Erro de capability
#[derive(Debug, Clone, Copy)]
pub enum CapError {
    InvalidHandle,
    InsufficientRights,
    TypeMismatch,
    CSpaceFull,
    NotTransferable,
}
```

### 4.4 `credentials/cred.rs`

```rust
//! Credenciais de processo

use crate::sys::types::{Uid, Gid};

/// Credenciais do processo
#[derive(Debug, Clone)]
pub struct Credentials {
    /// UID real
    pub uid: Uid,
    /// GID real
    pub gid: Gid,
    /// UID efetivo
    pub euid: Uid,
    /// GID efetivo
    pub egid: Gid,
    /// GIDs suplementares
    pub groups: [Gid; 16],
    pub num_groups: usize,
}

impl Credentials {
    /// Credenciais root
    pub const fn root() -> Self {
        Self {
            uid: Uid::ROOT,
            gid: Gid::ROOT,
            euid: Uid::ROOT,
            egid: Gid::ROOT,
            groups: [Gid::ROOT; 16],
            num_groups: 0,
        }
    }
    
    /// Credenciais padrão de usuário
    pub const fn user(uid: Uid, gid: Gid) -> Self {
        Self {
            uid,
            gid,
            euid: uid,
            egid: gid,
            groups: [Gid::ROOT; 16],
            num_groups: 0,
        }
    }
    
    /// Verifica se é privilegiado
    pub fn is_privileged(&self) -> bool {
        self.euid.0 == 0
    }
}
```

### 4.5 `audit/audit.rs`

```rust
//! Log de auditoria de segurança

use crate::sys::types::{Pid, Uid};

/// Tipo de evento de auditoria
#[derive(Debug, Clone, Copy)]
pub enum AuditEvent {
    CapabilityGranted,
    CapabilityDenied,
    CapabilityRevoked,
    ProcessCreated,
    ProcessTerminated,
    AccessDenied,
    PrivilegeEscalation,
}

/// Entrada de log de auditoria
pub struct AuditEntry {
    pub timestamp: u64,
    pub event: AuditEvent,
    pub pid: Pid,
    pub uid: Uid,
    pub details: [u8; 64],
}

/// Loga evento de auditoria
pub fn log_event(event: AuditEvent, pid: Pid, uid: Uid) {
    // TODO: adicionar ao buffer de log
    crate::kdebug!("Audit:", event as u64);
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `capability/rights.rs` - Bitflags de direitos
2. `capability/cap.rs` - Capability struct
3. `capability/cspace.rs` - Tabela por processo
4. `credentials/cred.rs` - Credenciais
5. `audit/audit.rs` - Logging

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::sync`
- `crate::sys::types`
- `crate::klib`

---

## 7. CHECKLIST

- [ ] CapRights usa bitflags corretamente
- [ ] CSpace limita tamanho
- [ ] Derivação só reduz direitos
- [ ] Handle 0 é sempre inválido
