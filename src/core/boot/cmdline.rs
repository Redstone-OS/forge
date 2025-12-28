/// Arquivo: core/boot/cmdline.rs
///
/// Propósito: Parser da Linha de Comando do Kernel.
/// Gerencia os parâmetros passados pelo Bootloader (ex: "debug", "root=/dev/sda1").
///
/// Detalhes de Implementação:
/// - Armazenamento estático (sem heap) para estar disponível muito cedo no boot.
/// - Parser simples de chave=valor separados por espaços.

/// Kernel Command Line
use crate::klib::string::strcmp; // Assumindo klib existente ou usamos u8 compare manual

/// Tamanho máximo da linha de comando
const CMDLINE_MAX_LEN: usize = 256;

pub struct CommandLine {
    buffer: [u8; CMDLINE_MAX_LEN],
    len: usize,
}

impl CommandLine {
    const fn new() -> Self {
        Self {
            buffer: [0; CMDLINE_MAX_LEN],
            len: 0,
        }
    }

    /// Inicializa a linha de comando com a string fornecida pelo bootloader.
    pub fn init(&mut self, args: &str) {
        let bytes = args.as_bytes();
        self.len = core::cmp::min(bytes.len(), CMDLINE_MAX_LEN);

        // Copia bytes
        for i in 0..self.len {
            self.buffer[i] = bytes[i];
        }

        crate::kinfo!("Linha de Comando: ");
        // crate::kinfo!(args); // TODO: klog expects &'static str or uses static buffer? log() takes &str.
    }

    /// Verifica se uma flag (chave sem valor) ou parâmetro existe.
    pub fn has(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }

    /// Obtém o valor de um parâmetro (ex: "root" -> "/dev/sda").
    /// Retorna `Option<&str>`. Se for flag ("debug"), retorna Some("").
    pub fn get(&self, key: &str) -> Option<&str> {
        self.get_value(key)
    }

    fn get_value(&self, _key: &str) -> Option<&str> {
        // Implementação simplificada de parser.
        // Como não temos `std` nem `alloc` garantido aqui (no early boot),
        // precisaríamos de um parser iterativo sobre o slice `buffer`.

        // TODO: Implementar parser real. Por enquanto retorna None.
        None
    }
}

/// Instância global da linha de comando
pub static mut CMDLINE: CommandLine = CommandLine::new();

/// Helper seguro para inicializar
pub fn init(args: &str) {
    unsafe {
        CMDLINE.init(args);
    }
}
