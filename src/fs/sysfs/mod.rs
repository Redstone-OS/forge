//! SysFS - System Filesystem
//!
//! Sistema de arquivos para informações do sistema (/sys).
//!
//! # Implementação Básica
//! Fornece informações estáticas de dispositivos e kernel.

#![allow(dead_code)]

/// Tipo de entrada do SysFS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysEntryType {
    /// Dispositivo
    Device,
    /// Kernel
    Kernel,
    /// Módulo
    Module,
}

/// Entrada do SysFS
pub struct SysEntry {
    /// Nome da entrada
    pub name: &'static str,
    /// Tipo de entrada
    pub entry_type: SysEntryType,
    /// Conteúdo (texto estático)
    pub content: &'static str,
}

impl SysEntry {
    /// Cria uma nova entrada
    pub const fn new(name: &'static str, entry_type: SysEntryType, content: &'static str) -> Self {
        Self {
            name,
            entry_type,
            content,
        }
    }
}

/// SysFS - System Filesystem
pub struct SysFS {
    /// Entradas estáticas
    entries: [SysEntry; 3],
}

impl SysFS {
    /// Cria uma nova instância de SysFS
    pub fn new() -> Self {
        Self {
            entries: [
                SysEntry::new("kernel/version", SysEntryType::Kernel, "0.1.0\n"),
                SysEntry::new("kernel/ostype", SysEntryType::Kernel, "Redstone\n"),
                SysEntry::new("devices/system", SysEntryType::Device, "cpu0\nmem0\n"),
            ],
        }
    }

    /// Lê informações de uma entrada
    pub fn read(&self, path: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.name == path)
            .map(|e| e.content)
    }

    /// Lista entradas disponíveis
    pub fn list(&self) -> &[SysEntry] {
        &self.entries
    }

    /// Lista entradas por tipo
    pub fn list_by_type(&self, entry_type: SysEntryType) -> impl Iterator<Item = &SysEntry> {
        self.entries
            .iter()
            .filter(move |e| e.entry_type == entry_type)
    }
}

impl Default for SysFS {
    fn default() -> Self {
        Self::new()
    }
}

// TODO(prioridade=média, versão=v1.0): Implementar hierarquia de dispositivos
// - /sys/devices/ - Árvore de dispositivos
// - /sys/bus/ - Barramentos do sistema
// - /sys/class/ - Classes de dispositivos

// TODO(prioridade=baixa, versão=v2.0): Integrar com sistema de drivers
// - Registrar dispositivos dinamicamente
// - Expor atributos de dispositivos
