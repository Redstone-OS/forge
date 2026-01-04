//! # Portas e Constantes do Controlador PS/2 8042
//!
//! Definições para o controlador de teclado/mouse PS/2.

/// Porta de dados do controlador PS/2
pub const DATA_PORT: u16 = 0x60;

/// Porta de status (leitura) / comando (escrita)
pub const STATUS_PORT: u16 = 0x64;

/// Porta de comando (mesmo endereço que STATUS)
pub const CMD_PORT: u16 = 0x64;

/// Bits do Status Register
pub mod status {
    /// Output buffer full - dados disponíveis para leitura
    pub const OUTPUT_FULL: u8 = 0x01;
    /// Input buffer full - aguardando processamento
    pub const INPUT_FULL: u8 = 0x02;
    /// System flag - passou no self-test
    pub const SYSTEM_FLAG: u8 = 0x04;
    /// Command/Data - 0=dados para PS/2, 1=comando para controlador
    pub const COMMAND_DATA: u8 = 0x08;
    /// Keyboard lock - teclado bloqueado
    pub const KEYBOARD_LOCK: u8 = 0x10;
    /// Auxiliary output buffer full - dados do mouse
    pub const AUX_OUTPUT_FULL: u8 = 0x20;
    /// Timeout error
    pub const TIMEOUT_ERROR: u8 = 0x40;
    /// Parity error
    pub const PARITY_ERROR: u8 = 0x80;
}

/// Comandos do Controlador 8042
pub mod commands {
    /// Ler Command Byte
    pub const READ_CONFIG: u8 = 0x20;
    /// Escrever Command Byte
    pub const WRITE_CONFIG: u8 = 0x60;
    /// Desabilitar porta auxiliar (mouse)
    pub const DISABLE_AUX: u8 = 0xA7;
    /// Habilitar porta auxiliar (mouse)
    pub const ENABLE_AUX: u8 = 0xA8;
    /// Testar porta auxiliar
    pub const TEST_AUX: u8 = 0xA9;
    /// Self-test do controlador
    pub const SELF_TEST: u8 = 0xAA;
    /// Testar porta do teclado
    pub const TEST_KEYBOARD: u8 = 0xAB;
    /// Desabilitar teclado
    pub const DISABLE_KEYBOARD: u8 = 0xAD;
    /// Habilitar teclado
    pub const ENABLE_KEYBOARD: u8 = 0xAE;
    /// Escrever próximo byte no dispositivo auxiliar
    pub const WRITE_AUX: u8 = 0xD4;
}

/// Bits do Configuration Byte
pub mod config {
    /// Habilitar interrupção do teclado (IRQ 1)
    pub const KEYBOARD_IRQ: u8 = 0x01;
    /// Habilitar interrupção do mouse (IRQ 12)
    pub const AUX_IRQ: u8 = 0x02;
    /// System flag
    pub const SYSTEM_FLAG: u8 = 0x04;
    /// Ignorar keyboard lock
    pub const IGNORE_LOCK: u8 = 0x08;
    /// Desabilitar teclado
    pub const DISABLE_KEYBOARD: u8 = 0x10;
    /// Desabilitar mouse
    pub const DISABLE_AUX: u8 = 0x20;
    /// Traduzir scan codes para set 1
    pub const TRANSLATE: u8 = 0x40;
}
