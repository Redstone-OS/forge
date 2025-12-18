//! ProcFS - Process Filesystem
//!
//! Sistema de arquivos para informações de processos (/proc).
//!
//! # Implementação Básica
//! Fornece informações estáticas do sistema para boot inicial.

#![allow(dead_code)]

/// Entrada do ProcFS
pub struct ProcEntry {
    /// Nome da entrada
    pub name: &'static str,
    /// Conteúdo (texto estático)
    pub content: &'static str,
}

impl ProcEntry {
    /// Cria uma nova entrada
    pub const fn new(name: &'static str, content: &'static str) -> Self {
        Self { name, content }
    }
}

/// ProcFS - Process Filesystem
pub struct ProcFS {
    /// Entradas estáticas
    entries: [ProcEntry; 4],
}

impl ProcFS {
    /// Cria uma nova instância de ProcFS
    pub fn new() -> Self {
        Self {
            entries: [
                ProcEntry::new("cpuinfo", "processor\t: 0\nvendor_id\t: GenuineIntel\n"),
                ProcEntry::new("meminfo", "MemTotal:\t1048576 kB\nMemFree:\t524288 kB\n"),
                ProcEntry::new("uptime", "0.00 0.00\n"),
                ProcEntry::new("version", "Redstone 0.1.0\n"),
            ],
        }
    }

    /// Lê informações de uma entrada
    pub fn read(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.content)
    }

    /// Lista entradas disponíveis
    pub fn list(&self) -> &[ProcEntry] {
        &self.entries
    }
}

impl Default for ProcFS {
    fn default() -> Self {
        Self::new()
    }
}

// TODO(prioridade=média, versão=v1.0): Implementar entradas dinâmicas
// - /proc/[pid]/ - Informações de processos
// - /proc/[pid]/cmdline - Linha de comando
// - /proc/[pid]/status - Status do processo
// - /proc/[pid]/maps - Mapeamento de memória

// TODO(prioridade=baixa, versão=v2.0): Integrar com scheduler
// - Listar processos reais
// - Informações dinâmicas de CPU/memória
