//! # Estrutura de Dispositivo (Device)
//!
//! Este arquivo define a representação lógica de uma unidade de hardware no kernel.
//! Cada objeto `Device` contém informações sobre sua identidade, estado atual e
//! sua posição na hierarquia do sistema.
//!
//! ## Atributos Principais:
//! - **Identidade**: ID único, nome amigável e endereço físico no barramento.
//! - **Estado**: Rastreia o ciclo de vida (Initializing, Ready, Failing, etc).
//! - **Hierarquia**: Ponteiros para pai e filhos, permitindo navegar na topologia do hardware.
//! - **Dados Privados (`Any`)**: Permite que drivers específicos armazenem estruturas
//!   complexas (ex: structs de registradores xHCI) de forma isolada.
//!
//! O `Device` é agnóstico ao barramento, servindo como uma interface comum para
//! o subsistema de arquivos, gerenciamento de energia e UI.

use super::bus::{BusAddress, BusType};
use super::driver::{DeviceType, Driver};
use crate::sync::Spinlock;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::any::Any;

/// ID único de dispositivo (64-bit para evitar colisões em sistemas massivos)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId(pub u64);

/// Estados do ciclo de vida do dispositivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Disconnected, // Hardware removido fisicamente
    Initializing, // Detectado, mas driver ainda não carregado
    Ready,        // Driver anexado e hardware operacional
    Failing,      // Erros detectados pelo monitor de saúde
    Suspended,    // Em modo de baixo consumo (D1-D3)
    Dead,         // Desativado pelo kernel por segurança (falha crítica)
}

/// Estrutura de Dispositivo Unificado
/// Representa uma instância física de hardware.
pub struct Device {
    /// Identificador único
    pub id: DeviceId,
    /// Estado atual de saúde e operação
    pub state: DeviceState,
    /// Nome amigável (limitado a 32 bytes para evitar alocações excessivas)
    pub name: [u8; 32],

    /// Classificação funcional (Storage, Network, etc)
    pub device_type: DeviceType,
    /// Tipo de conexão física (PCI, USB, etc)
    pub bus_type: BusType,
    /// Endereço físico no barramento pai
    pub bus_address: BusAddress,

    /// Driver atualmente encarregado deste hardware
    pub driver: Option<Arc<dyn Driver>>,

    /// Topologia: Dispositivo pai (ex: Root Complex ou USB Hub)
    pub parent: Option<Weak<Spinlock<Device>>>,
    /// Topologia: Dispositivos conectados a este (ex: dispositivos USB em um hub)
    pub children: Vec<Arc<Spinlock<Device>>>,

    /// Dados privados do driver (downcast seguro via Any)
    pub data: Option<Arc<dyn Any + Send + Sync>>,

    /// Estatísticas e Timestamps
    pub discovery_time: u64,
    pub last_status_change: u64,
}

impl Device {
    /// Cria um novo dispositivo.
    /// Inicia em estado `Initializing` e sem driver associado.
    pub fn new(
        id: DeviceId,
        name: &str,
        bus: BusType,
        addr: BusAddress,
        dev_type: DeviceType,
    ) -> Self {
        let mut name_buf = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(31);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        // Obter timestamp atual do sistema (se disponível)
        // let now = crate::core::time::get_timestamp(); // Placeholder para o futuro
        let now = 0;

        Self {
            id,
            state: DeviceState::Initializing,
            name: name_buf,
            device_type: dev_type,
            bus_type: bus,
            bus_address: addr,
            driver: None,
            parent: None,
            children: Vec::new(),
            data: None,
            discovery_time: now,
            last_status_change: now,
        }
    }

    /// Retorna o nome do dispositivo como uma string Rust segura.
    pub fn name_as_str(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.name[..len]).unwrap_or("unknown")
    }

    /// Altera o estado do dispositivo e atualiza o timestamp da última mudança.
    pub fn set_state(&mut self, new_state: DeviceState) {
        if self.state != new_state {
            self.state = new_state;
            self.last_status_change = 0; // Atualizar com timestamp real futuramente
        }
    }

    /// Anexa dados privados específicos do driver ao dispositivo.
    /// Útil para guardar estados internos do hardware sem expor no modelo base.
    pub fn set_data<T: Any + Send + Sync>(&mut self, data: T) {
        self.data = Some(Arc::new(data));
    }

    /// Tenta recuperar os dados privados do driver fazendo o casting para o tipo T.
    pub fn get_data<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.data.clone()?.downcast::<T>().ok()
    }

    /// Estabelece uma relação de pai e filho entre dois dispositivos.
    /// Garante que a hierarquia seja mantida corretamente (Atomicamente via Spinlock).
    pub fn link_child(parent_arc: Arc<Spinlock<Device>>, child_arc: Arc<Spinlock<Device>>) {
        // Define o pai do filho
        child_arc.lock().parent = Some(Arc::downgrade(&parent_arc));
        // Adiciona o filho à lista do pai
        parent_arc.lock().children.push(child_arc);
    }
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Atributos Dinâmicos (sysfs-like):
//    - Implementar um sistema para que drivers exponham "arquivos" de atributos
//      (ex: /devices/pci0000:00/temp) com permissões de leitura/escrita.
//
// 2. Reference Counting para Usuários:
//    - Adicionar um contador de "busy" para evitar que um dispositivo seja
//      removido ou suspenso enquanto uma aplicação (VFS) estiver usando ele.
//
// 3. Capacidades (Capabilities Flags):
//    - Criar um sistema de bitmasks para descrever capacidades comuns (ex: CAN_DMA,
//      CAN_WAKEUP, REQUIRES_ALIGNMENT).
//
// 4. Integração com a Árvore de Classes:
//    - Adicionar um ponteiro direto da struct Device para o objeto Class correspondente.
//
// 5. Hooks de Evento por Dispositivo:
//    - Permitir que outros módulos "assinem" mudanças de estado de um dispositivo
//      específico (ex: Alerta se a temperatura do disco subir demais).
