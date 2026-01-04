//! # Parâmetros Dinâmicos (Driver Parameters)
//!
//! Permite a configuração "on-the-fly" de drivers sem a necessidade de recompilar
//! o kernel. Funciona de forma similar ao `/sys/module/` no Linux.
//!
//! ## Utilidade:
//! - Ativar logs de debug para drivers específicos.
//! - Ajustar timeouts de rede ou escalonamento de disco.
//! - Configurar politícas de hardware em tempo real.
//!
//! Os parâmetros podem ser alterados em tempo de execução via syscalls,
//! arquivos virtuais no `/devices/parameters` ou durante o boot via linha de comando.

use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec::Vec;

/// Valor tipado de um parâmetro de hardware
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Bool(bool),
    Int(i64),
    Uint(u64),
    Str(String),
}

/// Definição de um parâmetro de driver
pub struct DriverParameter {
    /// Nome único (ex: "xhci_debug_enabled")
    pub name: &'static str,
    /// Descrição amigável para o administrador
    pub description: &'static str,
    /// Valor atual
    pub value: ParameterValue,
    /// Permite alteração em tempo de execução?
    pub writable: bool,
}

struct ParameterRegistry {
    params: Vec<DriverParameter>,
}

static REGISTRY: Spinlock<ParameterRegistry> =
    Spinlock::new(ParameterRegistry { params: Vec::new() });

/// Registra um novo parâmetro de driver no sistema global.
pub fn register(param: DriverParameter) {
    let mut reg = REGISTRY.lock();
    // Evita duplicatas
    if !reg.params.iter().any(|p| p.name == param.name) {
        reg.params.push(param);
    }
}

/// Altera o valor de um parâmetro existente.
/// Retorna true se o parâmetro foi encontrado e alterado.
pub fn set(name: &str, new_value: ParameterValue) -> bool {
    let mut reg = REGISTRY.lock();
    if let Some(param) = reg.params.iter_mut().find(|p| p.name == name) {
        if param.writable {
            param.value = new_value;
            return true;
        }
    }
    false
}

/// Busca um parâmetro booleano. Se não encontrado, retorna o valor padrão fornecido.
pub fn get_bool(name: &str, default: bool) -> bool {
    let reg = REGISTRY.lock();
    reg.params
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            ParameterValue::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(default)
}

/// Busca um parâmetro inteiro.
pub fn get_int(name: &str, default: i64) -> i64 {
    let reg = REGISTRY.lock();
    reg.params
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            ParameterValue::Int(i) => Some(*i),
            _ => None,
        })
        .unwrap_or(default)
}

/// Retorna uma lista de todos os parâmetros para ferramentas de depuração.
pub fn list_all() -> Vec<(String, String)> {
    let reg = REGISTRY.lock();
    reg.params
        .iter()
        .map(|p| (String::from(p.name), format!("{:?}", p.value)))
        .collect()
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Integração com Bootloader Command Line:
//    - Analisar a string de boot (ex: "xhci.debug=1") e aplicar os parâmetros
//      antes da inicialização dos drivers.
//
// 2. Exportação para /devices/parameters (VFS):
//    - Criar arquivos virtuais no sistema de arquivos para que o administrador
//      possa ler/escrever parâmetros via Shell (`echo 1 > /devices/params/xhci_debug`).
//
// 3. Callbacks de Notificação:
//    - Adicionar um hook no `DriverParameter` para avisar o driver sempre que
//      o parâmetro for alterado, permitindo que ele reconfigure o hardware na hora.
//
// 4. Validação de Intervalos:
//    - Adicionar campos de Min/Max para parâmetros numéricos para evitar que
//      valores perigosos sejam configurados no hardware.
//
// 5. Parâmetros por Instância:
//    - Permitir parâmetros específicos para um único dispositivo (ex: "sda.timeout")
//      em vez de globais por módulo.
