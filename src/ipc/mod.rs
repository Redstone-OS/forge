//! # Inter-Process Communication (IPC) Subsystem
//!
//! O subsistema `ipc` implementa o mecanismo de troca de mensagens entre processos,
//! fundamental para a arquitetura micro-modular do Redstone OS.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Message Passing:** Comunicação desacoplada via mensagens tipadas (Structs/Bytes).
//! - **Portas (Endpoints):** Filas de mensagens (`VecDeque`) protegidas, atuando como "caixas de correio".
//! - **Capabilities:** Transporte seguro de permissões (Handles) entre processos.
//!
//! ## 🏗️ Arquitetura dos Módulos
//!
//! | Módulo    | Responsabilidade | Estado Atual |
//! |-----------|------------------|--------------|
//! | `message` | Define o envelope da mensagem (`MessageHeader`, dados, caps). | **Alloc-heavy:** Usa `Vec<u8>` para payload, gerando pressão no Heap. |
//! | `port`    | Implementa a fila de mensagens e lógica de envio/recebimento. | **Síncrono/Polling:** `recv` retorna `Empty` em vez de bloquear a thread. |
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Simplicidade (KISS):** A implementação inicial é fácil de auditar e livre de *deadlocks* complexos (apenas um Mutex por porta).
//! - **Segurança de Tipos:** O uso de `PortHandle` e `Message` encapsula bem a lógica bruta de ponteiros.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Falta de Bloqueio (Scheduler Integration):** O método `recv` não coloca a thread em *Sleep* se a fila estiver vazia.
//!   - *Consequência:* Consumidores precisam fazer *busy wait* ou polling manual, desperdiçando CPU.
//! - **Alocação Dinâmica Excessiva:** Cada `Message::new` aloca um `Vec`. Num sistema de alta frequência, isso fragmentará o Heap.
//! - **Cópia de Dados:** O payload é copiado da Userland para o Kernel (Sender) e do Kernel para a Userland (Receiver). `memcpy` duplo.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Performance)** Implementar **Zero-Copy** para mensagens grandes (Shared Memory).
//!   - *Motivo:* Evitar overhead de `memcpy` para transferências de arquivos ou buffers de vídeo.
//! - [ ] **TODO: (Scheduler)** Integrar `Port::recv()` com `Thread::park()`.
//!   - *Objetivo:* Se `Empty`, a thread deve dormir e ser acordada apenas quando houver `send()`.
//! - [ ] **TODO: (Optimization)** Substituir `Vec<u8>` por um **Slab Allocator** ou Pool de Mensagens fixas.
//!   - *Impacto:* Reduzir latência de alocação e fragmentação de memória.
//! - [ ] **TODO: (Security)** Implementar verificação rigorosa de limites de portas por processo.
//!   - *Risco:* Um processo malicioso pode criar infinitas portas e exaurir a memória do kernel (DoS).

pub mod message;
pub mod port;
pub mod test;

pub use message::Message;
pub use port::{Port, PortHandle, PortStatus};

/// Inicializa o subsistema de IPC.
pub fn init() {
    crate::kinfo!("(IPC) Inicializando subsistema de mensagens...");
    crate::kdebug!("(IPC) init: Protocolo assíncrono baseado em capacidades ativo");
    // Futuro: Criar portas globais do sistema (ex: NameService)
    crate::kinfo!("(IPC) Inicializado");
}
