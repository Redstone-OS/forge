//! # Contexto Persistente de Driver (DriverContext)
//!
//! Este módulo implementa o **DriverContext**, uma estrutura que sobrevive
//! a reloads de drivers. Quando um driver precisa ser atualizado ou
//! reiniciado após uma falha, o contexto preserva:
//!
//! - **Estado**: Filas de requisições pendentes
//! - **Configuração**: Parâmetros que o driver havia aplicado
//! - **Histórico**: Informações sobre falhas anteriores
//!
//! ## Problema que Resolve:
//! Sem contexto persistente, hot-reload de drivers perderia:
//! - Requisições I/O em andamento
//! - Conexões de rede estabelecidas
//! - Estados internos do hardware
//!
//! ## Filosofia:
//! O Device persiste (é o hardware). O DriverContext persiste (é o estado).
//! Apenas o código do Driver é substituído durante reload.
//!
//! ## Políticas de Recuperação:
//! Cada tipo de driver tem uma política diferente para lidar com falhas.
//! O contexto armazena qual política aplicar.

use super::device::DeviceId;
use super::driver::DriverError;
// TODO: Revisar uso no futuro
#[allow(unused_imports)]
use crate::sync::Spinlock;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

// =============================================================================
// POLÍTICAS DE RECUPERAÇÃO
// =============================================================================

/// Política de recuperação a ser aplicada quando o driver falha.
///
/// Cada categoria de driver tem comportamento diferente:
/// - Storage: Dados são críticos - pausa tudo, tenta não perder nada
/// - Network: Perda de pacotes é aceitável - reconexão é esperada
/// - Display: Usuário precisa ver algo - fallback rápido
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPolicy {
    /// Pausa todas as operações e aguarda recuperação.
    /// Usado para Storage - não pode perder dados.
    /// Após recuperação, resume todas as operações pendentes.
    PauseAndRetry,

    /// Descarta operações pendentes e notifica os solicitantes.
    /// Usado para Network - TCP/IP lida com retransmissões.
    /// Mais rápido de recuperar, aceita perda parcial.
    DropAndNotify,

    /// Tenta fallback para driver alternativo imediatamente.
    /// Usado para Display - usuário precisa ver a tela.
    /// Prioriza continuidade visual sobre features.
    CriticalFallback,

    /// Não pode esperar - age imediatamente.
    /// Usado para Timer/IRQ - são críticos para o scheduler.
    /// Falha parcial é melhor que sistema travado.
    Immediate,
}

impl RecoveryPolicy {
    /// Retorna política padrão baseada no tipo de dispositivo.
    pub fn default_for_type(dev_type: super::driver::DeviceType) -> Self {
        use super::driver::DeviceType;
        match dev_type {
            DeviceType::Storage => Self::PauseAndRetry,
            DeviceType::Network => Self::DropAndNotify,
            DeviceType::Display => Self::CriticalFallback,
            DeviceType::Timer | DeviceType::Controller => Self::Immediate,
            _ => Self::DropAndNotify, // Padrão seguro
        }
    }

    /// Retorna nome legível da política.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PauseAndRetry => "PauseAndRetry",
            Self::DropAndNotify => "DropAndNotify",
            Self::CriticalFallback => "CriticalFallback",
            Self::Immediate => "Immediate",
        }
    }
}

// =============================================================================
// REQUISIÇÃO PENDENTE
// =============================================================================

/// Representa uma requisição que estava em andamento quando o driver falhou.
///
/// Armazenada no contexto para ser retomada após recuperação.
#[derive(Debug, Clone)]
pub struct PendingRequest {
    /// ID único da requisição.
    pub id: u64,

    /// Tipo da operação (read, write, etc).
    pub operation: RequestType,

    /// Timestamp de quando a requisição foi submetida.
    pub submitted_at: u64,

    /// Dados da requisição (se aplicável).
    pub data: Option<Vec<u8>>,

    /// Status atual.
    pub status: RequestStatus,
}

/// Tipo de operação da requisição.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    Read,
    Write,
    Control,
    Status,
    Other,
}

/// Status de uma requisição pendente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    /// Aguardando processamento.
    Pending,

    /// Em processamento pelo driver.
    InProgress,

    /// Pausada devido a falha do driver.
    Paused,

    /// Será descartada.
    Dropping,

    /// Concluída com sucesso.
    Completed,

    /// Falhou.
    Failed,
}

// =============================================================================
// ESTRUTURA PRINCIPAL: DRIVER CONTEXT
// =============================================================================

/// Contexto persistente associado a um dispositivo.
///
/// Esta estrutura NÃO é destruída quando o driver é recarregado.
/// Ela mantém o estado que precisa sobreviver entre reloads.
pub struct DriverContext {
    /// ID do dispositivo ao qual este contexto pertence.
    pub device_id: DeviceId,

    /// Política de recuperação a ser aplicada.
    pub recovery_policy: RecoveryPolicy,

    /// Fila de requisições pendentes.
    /// Usada para retomar operações após recuperação.
    pub pending_requests: VecDeque<PendingRequest>,

    /// Próximo ID de requisição.
    next_request_id: u64,

    /// Contador de falhas desde última inicialização bem-sucedida.
    pub failure_count: u32,

    /// Último erro ocorrido.
    pub last_error: Option<DriverError>,

    /// Timestamp da última falha.
    pub last_failure_time: u64,

    /// Flag indicando se o contexto está pausado.
    pub is_paused: bool,

    /// Dados de estado salvos pelo driver (opaque).
    /// O driver pode salvar seu estado interno aqui antes do reload.
    pub saved_state: Option<Vec<u8>>,
}

impl DriverContext {
    /// Cria um novo contexto para um dispositivo.
    pub fn new(device_id: DeviceId) -> Self {
        Self {
            device_id,
            recovery_policy: RecoveryPolicy::DropAndNotify, // Padrão seguro
            pending_requests: VecDeque::new(),
            next_request_id: 1,
            failure_count: 0,
            last_error: None,
            last_failure_time: 0,
            is_paused: false,
            saved_state: None,
        }
    }

    /// Define a política de recuperação.
    pub fn set_policy(&mut self, policy: RecoveryPolicy) {
        self.recovery_policy = policy;
        crate::kinfo!(
            "(Context) Política definida para device",
            self.device_id.0,
            ":",
            policy.as_str()
        );
    }

    /// Adiciona uma requisição à fila de pendentes.
    ///
    /// Retorna o ID da requisição para tracking.
    pub fn enqueue_request(&mut self, op: RequestType, data: Option<Vec<u8>>) -> u64 {
        let id = self.next_request_id;
        self.next_request_id += 1;

        let request = PendingRequest {
            id,
            operation: op,
            submitted_at: 0, // TODO: Timestamp real
            data,
            status: RequestStatus::Pending,
        };

        self.pending_requests.push_back(request);
        id
    }

    /// Marca uma requisição como concluída.
    pub fn complete_request(&mut self, id: u64) {
        if let Some(req) = self.pending_requests.iter_mut().find(|r| r.id == id) {
            req.status = RequestStatus::Completed;
        }
        // Remove completadas
        self.pending_requests
            .retain(|r| r.status != RequestStatus::Completed);
    }

    /// Pausa todas as requisições pendentes.
    ///
    /// Chamado quando o driver falha e a política é PauseAndRetry.
    pub fn pause_all(&mut self) {
        self.is_paused = true;
        for req in self.pending_requests.iter_mut() {
            if req.status == RequestStatus::Pending || req.status == RequestStatus::InProgress {
                req.status = RequestStatus::Paused;
            }
        }
        crate::kinfo!(
            "(Context) Requisições pausadas para device",
            self.device_id.0
        );
    }

    /// Resume todas as requisições pausadas.
    ///
    /// Chamado após recuperação bem-sucedida.
    pub fn resume_all(&mut self) {
        self.is_paused = false;
        for req in self.pending_requests.iter_mut() {
            if req.status == RequestStatus::Paused {
                req.status = RequestStatus::Pending;
            }
        }
        crate::kinfo!(
            "(Context) Requisições resumidas para device",
            self.device_id.0
        );
    }

    /// Descarta todas as requisições pendentes.
    ///
    /// Chamado quando a política é DropAndNotify.
    pub fn drop_all(&mut self) {
        let count = self.pending_requests.len();
        self.pending_requests.clear();
        crate::kwarn!(
            "(Context) Descartadas",
            count,
            "requisições do device",
            self.device_id.0
        );
    }

    /// Registra uma falha no contexto.
    pub fn record_failure(&mut self, err: DriverError) {
        self.failure_count += 1;
        self.last_error = Some(err);
        self.last_failure_time = 0; // TODO: Timestamp real

        crate::kerror!(
            "(Context) Falha registrada para device",
            self.device_id.0,
            "total:",
            self.failure_count
        );
    }

    /// Reseta contadores de falha após recuperação bem-sucedida.
    pub fn reset_failures(&mut self) {
        self.failure_count = 0;
        self.last_error = None;
        crate::kinfo!(
            "(Context) Contadores resetados para device",
            self.device_id.0
        );
    }

    /// Salva estado do driver para recuperação posterior.
    pub fn save_state(&mut self, state: Vec<u8>) {
        self.saved_state = Some(state);
        crate::kinfo!("(Context) Estado salvo para device", self.device_id.0);
    }

    /// Recupera estado salvo do driver.
    pub fn restore_state(&mut self) -> Option<Vec<u8>> {
        self.saved_state.take()
    }

    /// Retorna número de requisições pendentes.
    pub fn pending_count(&self) -> usize {
        self.pending_requests.len()
    }
}

// =============================================================================
// FUNÇÕES GLOBAIS
// =============================================================================

/// Aplica a política de recuperação apropriada para um contexto.
///
/// Chamado pelo RecoveryManager quando um driver falha.
pub fn apply_recovery_policy(ctx: &mut DriverContext, err: DriverError) {
    ctx.record_failure(err);

    match ctx.recovery_policy {
        RecoveryPolicy::PauseAndRetry => {
            crate::kinfo!("(Context) Aplicando PauseAndRetry para", ctx.device_id.0);
            ctx.pause_all();
        }
        RecoveryPolicy::DropAndNotify => {
            crate::kinfo!("(Context) Aplicando DropAndNotify para", ctx.device_id.0);
            ctx.drop_all();
        }
        RecoveryPolicy::CriticalFallback => {
            crate::kinfo!("(Context) Aplicando CriticalFallback para", ctx.device_id.0);
            ctx.drop_all();
            // Fallback é tratado pelo fallback.rs
        }
        RecoveryPolicy::Immediate => {
            crate::kinfo!("(Context) Aplicando Immediate para", ctx.device_id.0);
            // Não pausa, não descarta - age imediatamente
        }
    }
}
