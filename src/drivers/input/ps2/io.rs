//! # Funções de I/O do Controlador PS/2
//!
//! Wrappers para comunicação com o controlador 8042.

use super::ports::{CMD_PORT, DATA_PORT, STATUS_PORT};
use crate::arch::x86_64::ports::{inb, outb};

/// Timeout para operações (iterações)
const TIMEOUT: u32 = 100_000;

/// Espera o buffer de entrada ficar vazio (pronto para escrita)
pub fn wait_write() -> bool {
    for _ in 0..TIMEOUT {
        if (inb(STATUS_PORT) & 0x02) == 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// Espera o buffer de saída ter dados (pronto para leitura)
pub fn wait_read() -> bool {
    for _ in 0..TIMEOUT {
        if (inb(STATUS_PORT) & 0x01) != 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// Lê um byte da porta de dados
#[inline]
pub fn read_data() -> u8 {
    inb(DATA_PORT)
}

/// Escreve um byte na porta de dados
#[inline]
pub fn write_data(value: u8) {
    outb(DATA_PORT, value);
}

/// Lê o status do controlador
#[inline]
pub fn read_status() -> u8 {
    inb(STATUS_PORT)
}

/// Envia um comando para o controlador
pub fn send_command(cmd: u8) -> bool {
    if !wait_write() {
        return false;
    }
    outb(CMD_PORT, cmd);
    true
}

/// Envia um comando seguido de um dado
pub fn send_command_with_data(cmd: u8, data: u8) -> bool {
    if !send_command(cmd) {
        return false;
    }
    if !wait_write() {
        return false;
    }
    write_data(data);
    true
}

/// Lê o Configuration Byte
pub fn read_config() -> Option<u8> {
    if !send_command(super::ports::commands::READ_CONFIG) {
        return None;
    }
    if !wait_read() {
        return None;
    }
    Some(read_data())
}

/// Escreve o Configuration Byte
pub fn write_config(config: u8) -> bool {
    send_command_with_data(super::ports::commands::WRITE_CONFIG, config)
}

/// Envia um byte para o dispositivo auxiliar (mouse)
pub fn write_aux(byte: u8) -> Option<u8> {
    if !send_command(super::ports::commands::WRITE_AUX) {
        return None;
    }
    if !wait_write() {
        return None;
    }
    write_data(byte);
    if !wait_read() {
        return None;
    }
    Some(read_data()) // ACK ou resposta
}

/// Limpa o buffer de saída do controlador
pub fn flush_output() {
    while (read_status() & 0x01) != 0 {
        let _ = read_data();
    }
}

/// Verifica se há dados do mouse disponíveis (bit AUX no status)
#[inline]
pub fn is_mouse_data() -> bool {
    (read_status() & 0x20) != 0
}

/// Verifica se há dados disponíveis
#[inline]
pub fn has_data() -> bool {
    (read_status() & 0x01) != 0
}
