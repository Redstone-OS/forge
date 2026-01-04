//! # Estrutura de Dispositivo (Device)
//!
//! Este arquivo define a **representação lógica** de uma unidade de hardware
//! no kernel. Cada dispositivo físico detectado pelo sistema tem um objeto
//! `Device` correspondente que armazena:
//!
//! - **Identidade**: ID único, nome, endereço no barramento
//! - **Estado**: Ciclo de vida (Initializing → Ready → Failing → Dead)
//! - **Hierarquia**: Relação pai-filho (ex: USB device → USB hub → PCI controller)
//! - **Driver**: Referência ao driver que controla este dispositivo
//! - **Dados privados**: Estruturas específicas do driver (via Any)
//!
//! ## Filosofia:
//! O `Device` é **agnóstico ao barramento**. Não importa se o teclado veio
//! via USB ou PS/2 - para o restante do kernel, ambos são "Input Devices".
//!
//! ## Persistência:
//! O objeto Device **persiste** mesmo quando o driver é recarregado.
//! Isso permite hot-reload sem perder o contexto do hardware.

use super::bus::{BusAddress, BusType};
use super::driver::{DeviceType, Driver};
use crate::sync::Spinlock;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::any::Any;

// =============================================================================
// IDENTIFICADOR DE DISPOSITIVO
// =============================================================================

/// Identificador único de dispositivo.
///
/// Cada dispositivo no sistema tem um ID único de 64 bits que é atribuído
/// durante o registro. Este ID:
/// - Nunca é reutilizado (mesmo após remoção do dispositivo)
/// - Começa em 1 (0 é reservado para "inválido")
/// - É usado como chave em buscas e logs
///
/// Wrapper newtype para type safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(pub u64);

impl DeviceId {
    /// ID inválido/nulo (usado como placeholder)
    pub const INVALID: DeviceId = DeviceId(0);

    /// Verifica se o ID é válido (não-zero)
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

// =============================================================================
// ESTADOS DO DISPOSITIVO
// =============================================================================

/// Estados do ciclo de vida de um dispositivo.
///
/// O dispositivo transita entre estes estados conforme eventos ocorrem.
/// O RecoveryManager pode forçar transições em caso de falhas.
///
/// ## Diagrama de Transições:
/// ```text
///                    ┌──────────────┐
///      (hotplug) ──> │ Initializing │
///                    └──────┬───────┘
///                           │ probe() sucesso
///                           ▼
///                    ┌──────────────┐
///      <──────────── │    Ready     │ <──────┐
///      │             └──────┬───────┘        │
///      │                    │ erro           │ recovery
///      │                    ▼                │
///      │             ┌──────────────┐        │
///      │             │   Failing    │ ───────┘
///      │             └──────┬───────┘
///      │                    │ muitos erros
///      │                    ▼
///      │             ┌──────────────┐
///      │             │    Dead      │
///      │             └──────────────┘
///      │
///      │ suspend()   ┌──────────────┐
///      └───────────> │  Suspended   │
///                    └──────────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    /// Hardware removido fisicamente ou desconectado.
    /// Dispositivo não responde e não deve ser acessado.
    Disconnected,

    /// Hardware detectado, aguardando driver compatível.
    /// Estado inicial após descoberta pelo bus.
    Initializing,

    /// Driver anexado e hardware operacional.
    /// Estado normal de funcionamento.
    Ready,

    /// Erros detectados, mas ainda tentando operar.
    /// Monitor de saúde detectou problemas.
    Failing,

    /// Em modo de baixo consumo (D1, D2, D3).
    /// Hardware parado para economizar energia.
    Suspended,

    /// Desativado permanentemente pelo kernel.
    /// Muitos erros ou falha irrecuperável.
    /// Não será mais usado nesta sessão.
    Dead,
}

impl DeviceState {
    /// Retorna nome legível do estado.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Initializing => "Initializing",
            Self::Ready => "Ready",
            Self::Failing => "Failing",
            Self::Suspended => "Suspended",
            Self::Dead => "Dead",
        }
    }

    /// Verifica se o dispositivo está operacional.
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Ready | Self::Failing)
    }

    /// Verifica se o dispositivo aceita novos comandos.
    pub fn can_accept_commands(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

// =============================================================================
// ESTRUTURA PRINCIPAL: DEVICE
// =============================================================================

/// Representação de um dispositivo de hardware no kernel.
///
/// Esta estrutura contém todas as informações necessárias para:
/// - Identificar o dispositivo
/// - Rastrear seu estado
/// - Navegar na hierarquia de hardware
/// - Associar com um driver
/// - Armazenar dados privados do driver
pub struct Device {
    // =========================================================================
    // IDENTIFICAÇÃO
    // =========================================================================
    /// Identificador único atribuído pelo DriverManager.
    /// Nunca muda após criação.
    pub id: DeviceId,

    /// Nome amigável do dispositivo (ex: "Intel e1000 Network Adapter").
    /// Buffer fixo de 32 bytes para evitar alocações dinâmicas.
    /// Útil para logs e interface de usuário.
    pub name: [u8; 32],

    // =========================================================================
    // CLASSIFICAÇÃO
    // =========================================================================
    /// Tipo funcional do dispositivo (Storage, Network, etc).
    /// Determina em qual classe o dispositivo será registrado.
    pub device_type: DeviceType,

    /// Tipo de barramento físico (PCI, USB, etc).
    /// Indica como o dispositivo está conectado.
    pub bus_type: BusType,

    /// Endereço no barramento pai.
    /// Formato depende do tipo de barramento.
    pub bus_address: BusAddress,

    // =========================================================================
    // IDENTIFICADORES DE HARDWARE
    // =========================================================================
    /// Vendor ID do fabricante (PCI/USB vendor ID).
    /// Usado para matching de drivers.
    pub vendor_id: u16,

    /// Device ID do produto específico.
    /// Junto com vendor_id, identifica univocamente o modelo.
    pub device_id: u16,

    /// Classe do dispositivo (conforme spec do barramento).
    /// Ex: PCI class 0x01 = Storage, 0x02 = Network.
    pub class_code: u8,

    /// Subclasse para refinamento da classificação.
    pub subclass_code: u8,

    /// Revisão do hardware.
    pub revision: u8,

    // =========================================================================
    // ESTADO
    // =========================================================================
    /// Estado atual do dispositivo no ciclo de vida.
    pub state: DeviceState,

    /// Driver atualmente controlando este hardware.
    /// None se aguardando driver compatível.
    pub driver: Option<Arc<dyn Driver>>,

    // =========================================================================
    // HIERARQUIA
    // =========================================================================
    /// Dispositivo pai na árvore de hardware.
    /// Weak para evitar reference cycles.
    /// Ex: USB device aponta para USB hub, que aponta para xHCI controller.
    pub parent: Option<Weak<Spinlock<Device>>>,

    /// Dispositivos filhos conectados a este.
    /// Ex: USB hub tem lista de devices conectados.
    pub children: Vec<Arc<Spinlock<Device>>>,

    // =========================================================================
    // DADOS DO DRIVER
    // =========================================================================
    /// Dados privados do driver.
    /// Usa Any para permitir downcast seguro para tipo específico.
    /// O driver pode armazenar structs complexas aqui.
    pub data: Option<Arc<dyn Any + Send + Sync>>,

    // =========================================================================
    // ESTATÍSTICAS
    // =========================================================================
    /// Timestamp de quando o dispositivo foi descoberto.
    /// Em ticks do timer do sistema.
    pub discovery_time: u64,

    /// Timestamp da última mudança de estado.
    pub last_state_change: u64,

    /// Contador de erros desde última inicialização.
    pub error_count: u32,
}

impl Device {
    /// Cria um novo dispositivo com valores padrão.
    ///
    /// O ID ainda não é atribuído (será feito pelo DriverManager).
    /// O estado inicial é `Initializing`.
    ///
    /// ## Parâmetros:
    /// - `name`: Nome legível (máximo 31 caracteres, truncado se maior)
    /// - `bus`: Tipo de barramento
    /// - `addr`: Endereço no barramento
    /// - `dev_type`: Classificação funcional
    pub fn new(name: &str, bus: BusType, addr: BusAddress, dev_type: DeviceType) -> Self {
        // Copia nome para buffer fixo (evita alocação)
        let mut name_buf = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(31); // Deixa espaço para null terminator
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            id: DeviceId::INVALID, // Será atribuído pelo DriverManager
            name: name_buf,
            device_type: dev_type,
            bus_type: bus,
            bus_address: addr,
            vendor_id: 0,
            device_id: 0,
            class_code: 0,
            subclass_code: 0,
            revision: 0,
            state: DeviceState::Initializing,
            driver: None,
            parent: None,
            children: Vec::new(),
            data: None,
            discovery_time: 0, // TODO: Obter timestamp real
            last_state_change: 0,
            error_count: 0,
        }
    }

    /// Retorna o nome do dispositivo como string Rust.
    ///
    /// Converte o buffer de bytes para &str, parando no primeiro null.
    pub fn name_as_str(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.name[..len]).unwrap_or("unknown")
    }

    /// Altera o estado do dispositivo.
    ///
    /// Atualiza também o timestamp da última mudança e emite log.
    pub fn set_state(&mut self, new_state: DeviceState) {
        if self.state != new_state {
            let old_state = self.state;
            self.state = new_state;
            self.last_state_change = 0; // TODO: Timestamp real

            crate::kinfo!(
                "(Device) Estado alterado:",
                self.name_as_str(),
                old_state.as_str(),
                "->",
                new_state.as_str()
            );
        }
    }

    /// Define os IDs de hardware do dispositivo.
    ///
    /// Chamado pelo driver de barramento durante a enumeração.
    pub fn set_ids(&mut self, vendor: u16, device: u16, class: u8, subclass: u8, rev: u8) {
        self.vendor_id = vendor;
        self.device_id = device;
        self.class_code = class;
        self.subclass_code = subclass;
        self.revision = rev;
    }

    /// Anexa dados privados do driver ao dispositivo.
    ///
    /// O driver pode armazenar qualquer struct que implemente
    /// `Any + Send + Sync` para uso posterior.
    pub fn set_data<T: Any + Send + Sync>(&mut self, data: T) {
        self.data = Some(Arc::new(data));
    }

    /// Recupera dados privados do driver com downcast para tipo T.
    ///
    /// Retorna None se:
    /// - Não há dados armazenados
    /// - O tipo T não corresponde ao tipo armazenado
    pub fn get_data<T: Any + Send + Sync + Clone>(&self) -> Option<Arc<T>> {
        self.data.clone()?.downcast::<T>().ok()
    }

    /// Limpa os dados privados do driver.
    ///
    /// Chamado durante remove() para liberar recursos.
    pub fn clear_data(&mut self) {
        self.data = None;
    }

    /// Incrementa contador de erros.
    ///
    /// Usado pelo monitor de saúde para rastrear problemas.
    pub fn record_error(&mut self) {
        self.error_count = self.error_count.saturating_add(1);
    }

    /// Reseta contador de erros.
    ///
    /// Chamado após recuperação bem-sucedida.
    pub fn clear_errors(&mut self) {
        self.error_count = 0;
    }

    /// Estabelece relação pai-filho entre dois dispositivos.
    ///
    /// ## Parâmetros:
    /// - `parent_arc`: Arc do dispositivo pai
    /// - `child_arc`: Arc do dispositivo filho
    ///
    /// Esta função atualiza ambos os dispositivos atomicamente.
    pub fn link_child(parent_arc: Arc<Spinlock<Device>>, child_arc: Arc<Spinlock<Device>>) {
        // Define o pai do filho (weak para evitar cycle)
        child_arc.lock().parent = Some(Arc::downgrade(&parent_arc));

        // Adiciona filho à lista do pai
        parent_arc.lock().children.push(child_arc);
    }

    /// Verifica se este dispositivo tem um pai específico.
    pub fn has_parent(&self, parent_id: DeviceId) -> bool {
        if let Some(weak_parent) = &self.parent {
            if let Some(parent) = weak_parent.upgrade() {
                return parent.lock().id == parent_id;
            }
        }
        false
    }

    /// Retorna quantidade de dispositivos filhos.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

// =============================================================================
// IMPLEMENTAÇÕES DE TRAIT PADRÃO
// =============================================================================

impl core::fmt::Debug for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Device")
            .field("id", &self.id.0)
            .field("name", &self.name_as_str())
            .field("type", &self.device_type)
            .field("state", &self.state)
            .field("bus", &self.bus_type)
            .field("vendor", &format_args!("{:#06x}", self.vendor_id))
            .field("device", &format_args!("{:#06x}", self.device_id))
            .finish()
    }
}
