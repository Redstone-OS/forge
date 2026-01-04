//! # Gerenciamento de Energia (Power Management Layer)
//!
//! Coordena o consumo elétrico dos periféricos no RedstoneOS.
//! Essencial para eficiência em notebooks (economia de bateria) e servidores.
//!
//! ## Conceitos:
//! - **D-States (Device States)**: Estados de energia do dispositivo (D0 a D3).
//! - **Hierarquia de Suspensão**: Dispositivos filhos devem ser suspensos antes dos pais.
//! - **Hierarquia de Retomada**: Dispositivos pais devem ser acordados antes dos filhos.
//!
//! O `PowerManager` garante a ordem correta de transição de energia para evitar
//! que um dispositivo tente acessar um barramento que já foi desligado.

use super::device::{DeviceId, DeviceState};
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Estados de energia padrão (Baseados na especificação ACPI/PCI)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PowerState {
    /// Full Power: Dispositivo totalmente operacional e respondendo.
    D0,
    /// Light Sleep: Baixa latência de retorno, economia mínima.
    D1,
    /// Deep Sleep: Maior economia, latência significativa para acordar.
    D2,
    /// Off: Desligado. O estado do hardware pode ser perdido.
    D3,
}

/// Eventos de transição de energia
pub enum PowerEvent {
    PrepareSuspend,
    Suspend,
    PrepareResume,
    Resume,
}

/// Interface que drivers devem implementar para suportar economia de energia.
pub trait PowerManaged: Send + Sync {
    /// Altera o estado de energia do hardware.
    fn set_power_state(&self, state: PowerState) -> bool;

    /// Retorna o estado atual reportado pelo hardware ou driver.
    fn get_power_state(&self) -> PowerState;

    /// Chamado antes da suspensão para salvar contextos (ex: registradores).
    fn save_context(&self) -> bool {
        true
    }

    /// Chamado após o retorno para restaurar contextos.
    fn restore_context(&self) -> bool {
        true
    }
}

/// Registro global de capacidades de energia.
/// Futuramente integrado ao DriverManager para percorrer a árvore.
pub struct PowerManager {
    current_system_state: PowerState,
}

static POWER_MANAGER: Spinlock<PowerManager> = Spinlock::new(PowerManager {
    current_system_state: PowerState::D0,
});

/// Tenta suspender um dispositivo específico.
/// Deve ser chamado pelo DriverManager respeitando a topologia.
pub fn suspend_device(id: DeviceId) -> bool {
    crate::kinfo!("(Power) Solicitando suspensão para ID:", id.0);
    // TODO: Recuperar o device do registry e invocar driver.suspend()
    true
}

/// Acorda o subsistema de hardware na ordem correta (Barramentos primeiro).
pub fn resume_all() {
    crate::kinfo!("(Power) Iniciando retomada global de energia...");
    let mut mgr = POWER_MANAGER.lock();
    mgr.current_system_state = PowerState::D0;
    // TODO: Percorrer a árvore de dispositivos em ordem BFS (Nível de Barramento primeiro)
}

/// Retorna o estado de energia global do sistema.
pub fn get_system_power_state() -> PowerState {
    POWER_MANAGER.lock().current_system_state
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Suspensão em cascata (Ordered Shutdown):
//    - Implementar algoritmo que percorre a árvore de dispositivos do
//      DriverManager e garante o desligamento de folhas (periféricos)
//      antes de troncos (barramentos/hubs).
//
// 2. Latency Tolerance Reporting (LTR):
//    - Drivers podem informar ao kernel quanto tempo levam para acordar,
//      permitindo que o sistema decida se vale a pena entrar em D2 ou D3.
//
// 3. Wake-on-Interrupt:
//    - Configurar interrupções específicas que podem acordar o sistema do
//      estado suspenso (ex: teclado, pacote de rede mágico).
//
// 4. Integração ACPI:
//    - Mapear os estados D0-D3 do RedstoneOS para os estados S0-S5 da BIOS/ACPI
//      através de chamadas AML.
//
// 5. Runtime Power Management:
//    - Desligar automaticamente dispositivos individuais que não estão sendo
//      usados (ex: desligar a GPU se apenas o console serial estiver ativo).
