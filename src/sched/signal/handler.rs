//! Registro de Signal Handlers

/// Ação padrão para um sinal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalDisposition {
    Ignore,
    Terminate,
    Core,
    Stop,
    Continue,
    Handler(u64), // Endereço função user-space
}

/// Tabela de ações para sinais
pub struct SignalHandlers {
    actions: [SignalDisposition; 32],
}

impl SignalHandlers {
    pub const fn new() -> Self {
        Self {
            actions: [SignalDisposition::Terminate; 32], // Default é Terminate para maioria (simplificado)
        }
    }

    pub fn get_action(&self, signum: i32) -> SignalDisposition {
        if signum > 0 && signum < 32 {
            self.actions[signum as usize]
        } else {
            SignalDisposition::Ignore
        }
    }

    pub fn set_action(&mut self, signum: i32, action: SignalDisposition) {
        if signum > 0 && signum < 32 {
            // SIGKILL e SIGSTOP não podem ser capturados/ignorados
            if signum != super::SIGKILL && signum != super::SIGSTOP {
                self.actions[signum as usize] = action;
            }
        }
    }
}
