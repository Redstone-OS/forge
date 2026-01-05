//! # Shared Memory (SHM)
//!
//! Memória compartilhada zero-copy entre processos.
//!
//! ## Visão Geral
//!
//! Este módulo implementa memória compartilhada no estilo POSIX, permitindo
//! que múltiplos processos acessem a mesma região de memória física.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                        Shared Memory (SHM)              │
//! ├─────────────────────────────────────────────────────────┤
//! │                                                         │
//! │     Processo A                         Processo B       │
//! │   ┌─────────────┐                   ┌─────────────┐     │
//! │   │ VirtAddr A  │                   │ VirtAddr B  │     │
//! │   │ 0x6_0000... │                   │ 0x6_0000... │     │
//! │   └──────┬──────┘                   └──────┬──────┘     │
//! │          │                                 │            │
//! │          │         ┌─────────────┐         │            │
//! │          └───────▶│ PhysFrames  │◀────────┘            │
//! │                    │ (shared)    │                      │
//! │                    └─────────────┘                      │
//! │                           ▲                             │
//! │                           │                             │
//! │                    ┌──────┴──────┐                      │
//! │                    │ ShmRegistry │                      │
//! │                    │ (global)    │                      │
//! │                    └─────────────┘                      │
//! │                                                         │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Componentes
//!
//! - [`SharedMemory`] - Região de memória compartilhada (frames físicos)
//! - [`ShmRegistry`] - Registry global de todas as regiões SHM
//! - [`ShmId`] - Identificador único de uma região
//! - [`ShmError`] - Erros de operações SHM
//!
//! ## Uso
//!
//! ```rust,ignore
//! use crate::ipc::shm::{ShmId, SHM_REGISTRY, ShmError};
//!
//! // Criar região compartilhada (4KB)
//! let mut registry = SHM_REGISTRY.lock();
//! let shm_id = registry.create(4096)?;
//!
//! // Mapear no address space do processo
//! if let Some(shm) = registry.get(shm_id) {
//!     let vaddr = shm.map_in_aspace(&aspace, 0x6_0000_0000)?;
//!     // Processos podem agora ler/escrever em vaddr
//! }
//!
//! // Quando terminar, liberar referência
//! registry.release(shm_id);
//! ```
//!
//! ## Ciclo de Vida
//!
//! 1. **Criação**: `ShmRegistry::create(size)` aloca frames físicos
//! 2. **Mapeamento**: `SharedMemory::map_in_aspace()` mapeia nos processos
//! 3. **Uso**: Processos leem/escrevem na memória compartilhada
//! 4. **Liberação**: `ShmRegistry::release()` decrementa refcount
//! 5. **Destruição**: Quando refcount=0, frames são liberados
//!
//! ## Integração com RMM
//!
//! O SHM usa o Redstone Memory Manager (RMM) para:
//! - Alocar frames físicos com `FrameOwner::Shared`
//! - Mapear páginas nos AddressSpaces dos processos
//! - Rastrear ownership e permitir liberação segura
//!
//! ## Segurança
//!
//! - Cada região tem um ID único
//! - Reference counting previne uso após liberação
//! - Frames são zerados na criação
//! - Mapeamento requer validação de endereço
//!
//! ## TODO
//!
//! - Adicionar controle de permissões (read / write) por processo
//! - Definir processo proprietário (owner) da região SHM
//! - Permitir mapeamento com endereço virtual automático (vaddr = 0)
//! - Validar conflitos de VA durante o mapeamento
//! - Implementar namespaces de SHM (isolamento entre serviços)
//! - Adicionar métricas e debug hooks (uso, refcount, tamanho)
//! - Suportar redimensionamento seguro de regiões (opcional)
//! - Integrar SHM com políticas de segurança do sistema
//! - Preparar base para integração com IPC por fila/eventos

mod region;

pub use region::{SharedMemory, ShmError, ShmId, ShmRegistry, SHM_REGISTRY};
