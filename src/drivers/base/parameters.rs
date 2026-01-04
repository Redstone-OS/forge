//! # Parâmetros de Runtime (Driver Parameters)
//!
//! Este módulo permite configuração **em tempo de execução** de drivers
//! sem recompilação do kernel. Similar ao `/sys/module/` do Linux.
//!
//! ## Utilidades:
//! - Ativar logs de debug para drivers específicos
//! - Ajustar timeouts de rede ou storage
//! - Configurar políticas de hardware
//! - Tuning de performance
//!
//! ## Fontes de Parâmetros:
//! 1. Linha de comando do bootloader (ex: "xhci.debug=1")
//! 2. Arquivos virtuais em /devices/parameters
//! 3. Syscalls de configuração
//!
//! ## STUB:
//! Integração com bootloader e VFS ainda não implementada.

use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// TIPOS DE VALOR
// =============================================================================

/// Valor tipado de um parâmetro.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// Valor booleano (true/false, 0/1).
    Bool(bool),

    /// Inteiro com sinal.
    Int(i64),

    /// Inteiro sem sinal.
    Uint(u64),

    /// Texto.
    Str(String),
}

impl ParameterValue {
    /// Tenta converter para bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            Self::Int(i) => Some(*i != 0),
            Self::Uint(u) => Some(*u != 0),
            _ => None,
        }
    }

    /// Tenta converter para i64.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            Self::Uint(u) => Some(*u as i64),
            Self::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Tenta converter para u64.
    pub fn as_uint(&self) -> Option<u64> {
        match self {
            Self::Uint(u) => Some(*u),
            Self::Int(i) if *i >= 0 => Some(*i as u64),
            Self::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Retorna representação como string.
    pub fn to_string(&self) -> String {
        match self {
            Self::Bool(b) => alloc::format!("{}", b),
            Self::Int(i) => alloc::format!("{}", i),
            Self::Uint(u) => alloc::format!("{}", u),
            Self::Str(s) => s.clone(),
        }
    }
}

// =============================================================================
// ESTRUTURA DE PARÂMETRO
// =============================================================================

/// Definição de um parâmetro de driver.
#[derive(Debug, Clone)]
pub struct DriverParameter {
    /// Nome único (ex: "xhci_debug_enabled").
    pub name: &'static str,

    /// Descrição para administradores.
    pub description: &'static str,

    /// Valor atual.
    pub value: ParameterValue,

    /// Valor padrão (para reset).
    pub default: ParameterValue,

    /// Permite alteração em runtime?
    pub writable: bool,

    /// Nome do driver/módulo que registrou.
    pub module: &'static str,
}

// =============================================================================
// REGISTRO DE PARÂMETROS
// =============================================================================

/// Registro global de parâmetros.
struct ParameterRegistry {
    params: Vec<DriverParameter>,
    initialized: bool,
}

static REGISTRY: Spinlock<ParameterRegistry> = Spinlock::new(ParameterRegistry {
    params: Vec::new(),
    initialized: false,
});

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o sistema de parâmetros.
pub fn init() {
    crate::kinfo!("(Parameters) Inicializando sistema de parâmetros...");

    let mut reg = REGISTRY.lock();
    reg.initialized = true;

    // TODO: Parsear linha de comando do bootloader

    crate::kinfo!("(Parameters) Sistema pronto");
}

/// Registra um novo parâmetro.
pub fn register(param: DriverParameter) {
    let mut reg = REGISTRY.lock();

    // Evita duplicatas
    if reg.params.iter().any(|p| p.name == param.name) {
        crate::kwarn!("(Parameters) Parâmetro já existe:", param.name);
        return;
    }

    crate::kinfo!(
        "(Parameters) Registrado:",
        param.name,
        "=",
        param.value.to_string().as_str()
    );

    reg.params.push(param);
}

/// Define valor de um parâmetro existente.
///
/// ## Retorno:
/// - true: Alterado com sucesso
/// - false: Parâmetro não existe ou não é writable
pub fn set(name: &str, new_value: ParameterValue) -> bool {
    let mut reg = REGISTRY.lock();

    if let Some(param) = reg.params.iter_mut().find(|p| p.name == name) {
        if !param.writable {
            crate::kwarn!("(Parameters) Parâmetro não alterável:", name);
            return false;
        }

        crate::kinfo!(
            "(Parameters) Alterado:",
            name,
            "=",
            new_value.to_string().as_str()
        );

        param.value = new_value;
        return true;
    }

    crate::kwarn!("(Parameters) Parâmetro não encontrado:", name);
    false
}

/// Obtém valor booleano de um parâmetro.
pub fn get_bool(name: &str, default: bool) -> bool {
    let reg = REGISTRY.lock();

    reg.params
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| p.value.as_bool())
        .unwrap_or(default)
}

/// Obtém valor inteiro de um parâmetro.
pub fn get_int(name: &str, default: i64) -> i64 {
    let reg = REGISTRY.lock();

    reg.params
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| p.value.as_int())
        .unwrap_or(default)
}

/// Obtém valor unsigned de um parâmetro.
pub fn get_uint(name: &str, default: u64) -> u64 {
    let reg = REGISTRY.lock();

    reg.params
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| p.value.as_uint())
        .unwrap_or(default)
}

/// Reseta um parâmetro para seu valor padrão.
pub fn reset(name: &str) -> bool {
    let mut reg = REGISTRY.lock();

    if let Some(param) = reg.params.iter_mut().find(|p| p.name == name) {
        param.value = param.default.clone();
        crate::kinfo!("(Parameters) Resetado:", name);
        return true;
    }

    false
}

/// Retorna lista de todos os parâmetros.
pub fn list_all() -> Vec<(String, String, String)> {
    let reg = REGISTRY.lock();

    reg.params
        .iter()
        .map(|p| {
            (
                String::from(p.name),
                p.value.to_string(),
                String::from(p.description),
            )
        })
        .collect()
}

/// Retorna lista de parâmetros de um módulo específico.
pub fn list_for_module(module: &str) -> Vec<DriverParameter> {
    let reg = REGISTRY.lock();

    reg.params
        .iter()
        .filter(|p| p.module == module)
        .cloned()
        .collect()
}

// =============================================================================
// MACROS DE CONVENIÊNCIA
// =============================================================================

/// Cria um parâmetro booleano.
pub fn bool_param(
    name: &'static str,
    description: &'static str,
    default: bool,
    module: &'static str,
) -> DriverParameter {
    DriverParameter {
        name,
        description,
        value: ParameterValue::Bool(default),
        default: ParameterValue::Bool(default),
        writable: true,
        module,
    }
}

/// Cria um parâmetro inteiro.
pub fn int_param(
    name: &'static str,
    description: &'static str,
    default: i64,
    module: &'static str,
) -> DriverParameter {
    DriverParameter {
        name,
        description,
        value: ParameterValue::Int(default),
        default: ParameterValue::Int(default),
        writable: true,
        module,
    }
}
