//! # Eventos de Auditoria
//!
//! Define os tipos de eventos que são logados.

/// Categoria de evento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AuditCategory {
    /// Eventos de capability (grant, deny, revoke).
    Capability = 0,
    /// Eventos de processo (create, exit, exec).
    Process = 1,
    /// Eventos de acesso (denied, violation).
    Access = 2,
    /// Eventos de sistema (module load, config).
    System = 3,
    /// Eventos de IPC.
    Ipc = 4,
    /// Eventos de memória.
    Memory = 5,
}

/// Evento específico de auditoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum AuditEvent {
    // =========================================================================
    // Capability (0x01xx)
    // =========================================================================
    /// Capability concedida.
    CapGranted = 0x0100,
    /// Acesso negado por falta de capability.
    CapDenied = 0x0101,
    /// Capability revogada.
    CapRevoked = 0x0102,
    /// Capability transferida via IPC.
    CapTransferred = 0x0103,
    /// Capability duplicada.
    CapDuplicated = 0x0104,
    /// Capability derivada.
    CapDerived = 0x0105,

    // =========================================================================
    // Process (0x02xx)
    // =========================================================================
    /// Processo criado.
    ProcessCreated = 0x0200,
    /// Processo terminado.
    ProcessExited = 0x0201,
    /// Processo executou novo binário.
    ProcessExec = 0x0202,
    /// Fork de processo.
    ProcessFork = 0x0203,

    // =========================================================================
    // Access (0x03xx)
    // =========================================================================
    /// Acesso negado.
    AccessDenied = 0x0300,
    /// Violação de segurança.
    AccessViolation = 0x0301,
    /// Handle inválido usado.
    InvalidHandle = 0x0302,
    /// Tipo de capability incorreto.
    TypeMismatch = 0x0303,
    /// Direitos insuficientes.
    InsufficientRights = 0x0304,

    // =========================================================================
    // System (0x04xx)
    // =========================================================================
    /// Módulo de kernel carregado.
    ModuleLoaded = 0x0400,
    /// Módulo de kernel descarregado.
    ModuleUnloaded = 0x0401,
    /// Configuração de segurança alterada.
    ConfigChanged = 0x0402,
    /// Política de audit alterada.
    PolicyChanged = 0x0403,

    // =========================================================================
    // IPC (0x05xx)
    // =========================================================================
    /// Mensagem enviada.
    IpcSend = 0x0500,
    /// Mensagem recebida.
    IpcReceive = 0x0501,
    /// Canal criado.
    IpcChannelCreated = 0x0502,
    /// Canal fechado.
    IpcChannelClosed = 0x0503,

    // =========================================================================
    // Memory (0x06xx)
    // =========================================================================
    /// VMO criado.
    VmoCreated = 0x0600,
    /// Memória mapeada.
    MemoryMapped = 0x0601,
    /// Violação de memória.
    MemoryViolation = 0x0602,
}

impl AuditEvent {
    /// Retorna categoria do evento.
    pub const fn category(&self) -> AuditCategory {
        match (*self as u16) >> 8 {
            0x01 => AuditCategory::Capability,
            0x02 => AuditCategory::Process,
            0x03 => AuditCategory::Access,
            0x04 => AuditCategory::System,
            0x05 => AuditCategory::Ipc,
            0x06 => AuditCategory::Memory,
            _ => AuditCategory::System,
        }
    }

    /// Retorna nome do evento para logs.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::CapGranted => "CAP_GRANTED",
            Self::CapDenied => "CAP_DENIED",
            Self::CapRevoked => "CAP_REVOKED",
            Self::CapTransferred => "CAP_TRANSFERRED",
            Self::CapDuplicated => "CAP_DUPLICATED",
            Self::CapDerived => "CAP_DERIVED",

            Self::ProcessCreated => "PROC_CREATED",
            Self::ProcessExited => "PROC_EXITED",
            Self::ProcessExec => "PROC_EXEC",
            Self::ProcessFork => "PROC_FORK",

            Self::AccessDenied => "ACCESS_DENIED",
            Self::AccessViolation => "ACCESS_VIOLATION",
            Self::InvalidHandle => "INVALID_HANDLE",
            Self::TypeMismatch => "TYPE_MISMATCH",
            Self::InsufficientRights => "INSUFFICIENT_RIGHTS",

            Self::ModuleLoaded => "MODULE_LOADED",
            Self::ModuleUnloaded => "MODULE_UNLOADED",
            Self::ConfigChanged => "CONFIG_CHANGED",
            Self::PolicyChanged => "POLICY_CHANGED",

            Self::IpcSend => "IPC_SEND",
            Self::IpcReceive => "IPC_RECEIVE",
            Self::IpcChannelCreated => "IPC_CHANNEL_CREATED",
            Self::IpcChannelClosed => "IPC_CHANNEL_CLOSED",

            Self::VmoCreated => "VMO_CREATED",
            Self::MemoryMapped => "MEMORY_MAPPED",
            Self::MemoryViolation => "MEMORY_VIOLATION",
        }
    }

    /// Retorna código numérico.
    pub const fn code(&self) -> u16 {
        *self as u16
    }
}

/// Resultado de operação auditada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditResult {
    /// Operação bem sucedida.
    Success,
    /// Acesso negado.
    Denied,
    /// Erro com código.
    Error(i32),
}
