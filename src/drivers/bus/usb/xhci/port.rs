//! # Gerenciamento de Portas xHCI
//!
//! Lógica para detecção, reset e status de portas físicas.

use super::controller::XhciController;
use super::regs;
use super::types::UsbPort;
use alloc::vec::Vec;

impl XhciController {
    /// Escaneia todas as portas e detecta conexões
    pub fn scan_ports(&mut self) -> Vec<UsbPort> {
        let mut ports = Vec::new();
        let max_ports = self.max_ports;
        for port_num in 1..=max_ports {
            unsafe {
                let mut detected = false;

                // Sequência única e robusta para detecção de porta
                // 1. Limpar bits de mudança de status (W1C)
                let mut portsc = self.read_port32(port_num, regs::port::PORTSC);
                let clear_bits = portsc & 0x00FE_0000;
                self.write_port32(port_num, regs::port::PORTSC, portsc | clear_bits);

                // 2. Tentar resetar se algo estiver conectado (CCS bit 0)
                portsc = self.read_port32(port_num, regs::port::PORTSC);
                if (portsc & regs::portsc::CCS) != 0 {
                    crate::core::debug::display::log_hex(
                        "(xHCI) Porta conectada, resetando:",
                        port_num as u64,
                    );
                    self.write_port32(port_num, regs::port::PORTSC, portsc | regs::portsc::PR);
                    for _ in 0..100_000 {
                        portsc = self.read_port32(port_num, regs::port::PORTSC);
                        if (portsc & regs::portsc::PR) == 0 {
                            break;
                        }
                        core::hint::spin_loop();
                    }
                    // Limpar PRC (bit 21)
                    self.write_port32(port_num, regs::port::PORTSC, portsc | regs::portsc::PRC);
                } else {
                    // Garantir PP (Port Power bit 9)
                    self.write_port32(port_num, regs::port::PORTSC, portsc | regs::portsc::PP);
                }

                // 3. Esperar por conexão estável (bit 0 = CCS)
                for _ in 0..200_000 {
                    portsc = self.read_port32(port_num, regs::port::PORTSC);
                    if (portsc & regs::portsc::CCS) != 0 {
                        detected = true;
                        // Limpar CSC (bit 17)
                        self.write_port32(port_num, regs::port::PORTSC, portsc | regs::portsc::CSC);
                        break;
                    }
                    core::hint::spin_loop();
                }

                if !detected {
                    let status = self.read_port32(port_num, regs::port::PORTSC);
                    if status != 0 {}
                }
            }

            if let Some(port_info) = self.read_port_status(port_num) {
                if port_info.connected {
                    crate::core::debug::display::log_hex(
                        "(xHCI) Porta DETECTADA:",
                        port_num as u64,
                    );
                    ports.push(port_info);
                }
            }
        }
        ports
    }

    /// Tenta resetar uma porta específica
    pub fn reset_port(&mut self, port_num: u8) -> bool {
        let mut portsc = self.read_port32(port_num, regs::port::PORTSC);

        // Trigger RESET
        portsc |= regs::portsc::PR;
        self.write_port32(port_num, regs::port::PORTSC, portsc);

        // Aguardar bit PR limpar ou PRC setar
        for _ in 0..2_000_000 {
            let status = self.read_port32(port_num, regs::port::PORTSC);
            if (status & regs::portsc::PRC) != 0 || (status & regs::portsc::PR) == 0 {
                // Limpar PRC
                self.write_port32(port_num, regs::port::PORTSC, status | regs::portsc::PRC);
                return true;
            }
            core::hint::spin_loop();
        }
        false
    }
}
