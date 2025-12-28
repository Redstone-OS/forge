# Guia de Implementação: `ipc/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Comunicação entre processos. Portas, canais, pipes, shared memory.

---

## 2. ESTRUTURA

```
ipc/
├── mod.rs              ✅ JÁ EXISTE
├── message/
│   ├── mod.rs
│   └── message.rs      → Envelope de mensagem
├── port/
│   ├── mod.rs
│   ├── port.rs         → Port implementation
│   └── rights.rs       → Port rights
├── channel/
│   ├── mod.rs
│   └── channel.rs      → Canais bidirecionais
├── pipe/
│   ├── mod.rs
│   └── pipe.rs         → Pipes unidirecionais
├── shm/
│   ├── mod.rs
│   └── shm.rs          → Shared memory
└── futex/
    ├── mod.rs
    └── futex.rs        → Fast userspace mutex
```

---

## 3. REGRAS

### ❌ NUNCA:
- Copiar dados desnecessariamente
- Permitir acesso sem handle válido
- Bloquear com spinlock (usar mutex/waitqueue)

### ✅ SEMPRE:
- Validar handles antes de usar
- Usar capabilities para controle de acesso
- Integrar blocking com scheduler

---

## 4. IMPLEMENTAÇÕES

### 4.1 `message/message.rs`

```rust
//! Mensagem IPC

use alloc::vec::Vec;
use crate::core::object::Handle;

/// Header da mensagem
#[repr(C)]
pub struct MessageHeader {
    /// Tamanho total do payload
    pub size: u32,
    /// Número de handles anexados
    pub handle_count: u32,
    /// Tipo de mensagem (definido pelo usuário)
    pub msg_type: u32,
    /// Reservado
    pub reserved: u32,
}

/// Mensagem completa
pub struct Message {
    /// Header
    pub header: MessageHeader,
    /// Dados
    pub payload: Vec<u8>,
    /// Handles transferidos
    pub handles: Vec<Handle>,
}

impl Message {
    /// Cria mensagem vazia
    pub fn new(msg_type: u32) -> Self {
        Self {
            header: MessageHeader {
                size: 0,
                handle_count: 0,
                msg_type,
                reserved: 0,
            },
            payload: Vec::new(),
            handles: Vec::new(),
        }
    }
    
    /// Cria com dados
    pub fn with_data(msg_type: u32, data: &[u8]) -> Self {
        Self {
            header: MessageHeader {
                size: data.len() as u32,
                handle_count: 0,
                msg_type,
                reserved: 0,
            },
            payload: data.to_vec(),
            handles: Vec::new(),
        }
    }
    
    /// Anexa handle
    pub fn attach_handle(&mut self, handle: Handle) {
        self.handles.push(handle);
        self.header.handle_count += 1;
    }
}
```

### 4.2 `port/port.rs`

```rust
//! Porta de comunicação

use alloc::collections::VecDeque;
use crate::sync::Mutex;
use crate::sched::wait::WaitQueue;
use super::super::message::Message;

/// ID de porta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(u64);

static NEXT_PORT_ID: crate::sync::AtomicCounter = crate::sync::AtomicCounter::new(1);

/// Status da porta
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortStatus {
    Open,
    Closed,
}

/// Porta IPC
pub struct Port {
    id: PortId,
    status: PortStatus,
    queue: Mutex<VecDeque<Message>>,
    wait: WaitQueue,
    capacity: usize,
}

impl Port {
    /// Cria nova porta
    pub fn new(capacity: usize) -> Self {
        Self {
            id: PortId(NEXT_PORT_ID.inc()),
            status: PortStatus::Open,
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            wait: WaitQueue::new(),
            capacity,
        }
    }
    
    /// Envia mensagem (não bloqueante)
    pub fn send(&self, msg: Message) -> Result<(), IpcError> {
        if self.status != PortStatus::Open {
            return Err(IpcError::PortClosed);
        }
        
        let mut queue = self.queue.lock();
        if queue.len() >= self.capacity {
            return Err(IpcError::QueueFull);
        }
        
        queue.push_back(msg);
        drop(queue);
        
        // Acordar um waiter
        self.wait.wake_one();
        
        Ok(())
    }
    
    /// Recebe mensagem (não bloqueante)
    pub fn try_recv(&self) -> Result<Message, IpcError> {
        if self.status != PortStatus::Open {
            return Err(IpcError::PortClosed);
        }
        
        self.queue.lock()
            .pop_front()
            .ok_or(IpcError::Empty)
    }
    
    /// Recebe mensagem (bloqueante)
    pub fn recv(&self) -> Result<Message, IpcError> {
        loop {
            match self.try_recv() {
                Ok(msg) => return Ok(msg),
                Err(IpcError::Empty) => {
                    // Dormir até ter mensagem
                    self.wait.wait();
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    /// Fecha a porta
    pub fn close(&mut self) {
        self.status = PortStatus::Closed;
        self.wait.wake_all();
    }
}

/// Erro de IPC
#[derive(Debug, Clone, Copy)]
pub enum IpcError {
    PortClosed,
    QueueFull,
    Empty,
    InvalidHandle,
    PermissionDenied,
}

/// Handle para porta
#[derive(Debug, Clone, Copy)]
pub struct PortHandle(crate::core::object::Handle);
```

### 4.3 `channel/channel.rs`

```rust
//! Canal bidirecional (par de portas)

use super::super::port::{Port, PortHandle, IpcError};
use super::super::message::Message;

/// Canal bidirecional
pub struct Channel {
    /// Porta de envio
    send_port: Port,
    /// Porta de recepção
    recv_port: Port,
}

/// Par de canais (endpoints)
pub struct ChannelPair {
    pub endpoint0: ChannelEndpoint,
    pub endpoint1: ChannelEndpoint,
}

/// Um lado do canal
pub struct ChannelEndpoint {
    /// Porta para enviar
    send: *const Port,
    /// Porta para receber
    recv: *const Port,
}

impl Channel {
    /// Cria par de canais conectados
    pub fn create_pair() -> ChannelPair {
        let port_a = Port::new(16);
        let port_b = Port::new(16);
        
        // TODO: armazenar em algum lugar e retornar handles
        unimplemented!()
    }
}

impl ChannelEndpoint {
    /// Envia mensagem
    pub fn send(&self, msg: Message) -> Result<(), IpcError> {
        // SAFETY: Port é válida enquanto Channel existe
        unsafe { (*self.send).send(msg) }
    }
    
    /// Recebe mensagem
    pub fn recv(&self) -> Result<Message, IpcError> {
        // SAFETY: Port é válida enquanto Channel existe
        unsafe { (*self.recv).recv() }
    }
}
```

### 4.4 `shm/shm.rs`

```rust
//! Memória compartilhada

use crate::mm::{PhysFrame, VirtAddr};
use crate::sync::Spinlock;
use alloc::vec::Vec;

/// ID de região compartilhada
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShmId(u64);

/// Região de memória compartilhada
pub struct SharedMemory {
    id: ShmId,
    /// Frames físicos (compartilhados entre processos)
    frames: Vec<PhysFrame>,
    /// Tamanho em bytes
    size: usize,
    /// Contagem de referência
    ref_count: Spinlock<u32>,
}

impl SharedMemory {
    /// Cria região compartilhada
    pub fn create(size: usize) -> Result<Self, ShmError> {
        // Calcular número de frames necessários
        let page_size = crate::arch::PAGE_SIZE;
        let num_frames = (size + page_size - 1) / page_size;
        
        // Alocar frames
        let mut frames = Vec::with_capacity(num_frames);
        for _ in 0..num_frames {
            // TODO: alocar do PMM
            // frames.push(crate::mm::pmm::alloc_frame()?);
        }
        
        Ok(Self {
            id: ShmId(0), // TODO: gerar ID único
            frames,
            size,
            ref_count: Spinlock::new(1),
        })
    }
    
    /// Mapeia no address space
    pub fn map(&self, _vaddr: VirtAddr) -> Result<(), ShmError> {
        // TODO: mapear frames no VMM
        Ok(())
    }
    
    /// Desmapeia
    pub fn unmap(&self, _vaddr: VirtAddr) -> Result<(), ShmError> {
        // TODO: desmapear
        Ok(())
    }
}

#[derive(Debug)]
pub enum ShmError {
    OutOfMemory,
    InvalidAddress,
    NotMapped,
}
```

### 4.5 `futex/futex.rs`

```rust
//! Fast Userspace Mutex

use crate::sync::Spinlock;
use crate::sched::wait::WaitQueue;
use crate::mm::VirtAddr;
use alloc::collections::BTreeMap;

/// Tabela global de futexes
static FUTEX_TABLE: Spinlock<BTreeMap<u64, WaitQueue>> = 
    Spinlock::new(BTreeMap::new());

/// Futex - primitiva de sincronização userspace
pub struct Futex;

impl Futex {
    /// Wait: dorme se *addr == expected
    pub fn wait(addr: VirtAddr, expected: u32) -> Result<(), FutexError> {
        // Ler valor atual
        let current = unsafe { *(addr.as_ptr::<u32>()) };
        
        if current != expected {
            return Err(FutexError::WouldBlock);
        }
        
        // Adicionar à wait queue
        let mut table = FUTEX_TABLE.lock();
        let queue = table.entry(addr.as_u64())
            .or_insert_with(WaitQueue::new);
        
        // Dormir
        queue.wait();
        
        Ok(())
    }
    
    /// Wake: acorda até N threads esperando em addr
    pub fn wake(addr: VirtAddr, count: u32) -> u32 {
        let mut table = FUTEX_TABLE.lock();
        
        if let Some(queue) = table.get(&addr.as_u64()) {
            let mut woken = 0;
            for _ in 0..count {
                queue.wake_one();
                woken += 1;
            }
            woken
        } else {
            0
        }
    }
}

#[derive(Debug)]
pub enum FutexError {
    WouldBlock,
    InvalidAddress,
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `message/message.rs` - Estrutura de mensagem
2. `port/port.rs` - Port básica
3. `channel/channel.rs` - Canais
4. `pipe/pipe.rs` - Pipes
5. `shm/shm.rs` - Shared memory
6. `futex/futex.rs` - Futex

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::sync`
- `crate::mm`
- `crate::sched` (wait queues)
- `crate::core::object`

---

## 7. CHECKLIST

- [ ] Port integra com WaitQueue
- [ ] Messages podem transferir handles
- [ ] SharedMemory usa frames do PMM
- [ ] Futex acorda corretamente
