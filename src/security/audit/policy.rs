//! # Política de Auditoria
//!
//! Define quais eventos são logados.

use super::events::{AuditCategory, AuditEvent, AuditResult};

/// Política de auditoria.
#[derive(Debug, Clone)]
pub struct AuditPolicy {
    /// Auditoria habilitada.
    pub enabled: bool,

    /// Logar apenas falhas.
    pub failures_only: bool,

    /// Máscara de categorias habilitadas.
    pub categories: u8,
}

impl AuditPolicy {
    /// Política padrão (tudo habilitado).
    pub const fn default_policy() -> Self {
        Self {
            enabled: true,
            failures_only: false,
            categories: 0xFF, // Todas as categorias
        }
    }

    /// Política mínima (apenas falhas de segurança).
    pub const fn minimal() -> Self {
        Self {
            enabled: true,
            failures_only: true,
            categories: (1 << AuditCategory::Access as u8) | (1 << AuditCategory::Capability as u8),
        }
    }

    /// Política desabilitada.
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            failures_only: false,
            categories: 0,
        }
    }

    /// Verifica se evento deve ser logado.
    pub fn should_log(&self, event: &AuditEvent, result: &AuditResult) -> bool {
        if !self.enabled {
            return false;
        }

        // Verificar categoria
        let cat = event.category() as u8;
        if (self.categories & (1 << cat)) == 0 {
            return false;
        }

        // Se só falhas, ignorar sucesso
        if self.failures_only && matches!(result, AuditResult::Success) {
            return false;
        }

        true
    }

    /// Habilita categoria.
    pub fn enable_category(&mut self, cat: AuditCategory) {
        self.categories |= 1 << (cat as u8);
    }

    /// Desabilita categoria.
    pub fn disable_category(&mut self, cat: AuditCategory) {
        self.categories &= !(1 << (cat as u8));
    }

    /// Verifica se categoria está habilitada.
    pub fn is_category_enabled(&self, cat: AuditCategory) -> bool {
        (self.categories & (1 << (cat as u8))) != 0
    }
}
