//! Segurança do Kernel
//!
//! Modelo híbrido: DAC + Capabilities
//!
//! # Componentes
//! - DAC: Discretionary Access Control (permissões Unix)
//! - Capabilities: Capabilities básicas
//! - Audit: Log de operações sensíveis
//!
//! # TODOs
//! - TODO(prioridade=média, versão=v1.0): Implementar DAC
//! - TODO(prioridade=média, versão=v1.0): Implementar capabilities básicas
//! - TODO(prioridade=alta, versão=v1.0): Implementar audit log
//! - TODO(prioridade=baixa, versão=v2.0): Adicionar SELinux-like MAC

pub mod dac;
pub mod capability;
pub mod audit;
