//! # Module System Tests
//!
//! Testes do sistema de módulos.

/// Executa todos os testes do sistema de módulos
pub fn run_module_tests() {
    crate::kinfo!("[Module Test] Iniciando testes...");
    
    test_supervisor_creation();
    test_capability_system();
    test_watchdog();
    
    crate::kinfo!("[Module Test] Todos os testes passaram!");
}

fn test_supervisor_creation() {
    crate::ktrace!("[Module Test] test_supervisor_creation");
    
    // Verificar que supervisor existe
    let supervisor = super::SUPERVISOR.lock();
    assert!(supervisor.list_modules().is_empty());
    
    crate::ktrace!("[Module Test] OK: Supervisor criado");
}

fn test_capability_system() {
    crate::ktrace!("[Module Test] test_capability_system");
    
    use super::capability::{ModuleCapabilityManager, ModuleCapType};
    
    let mut manager = ModuleCapabilityManager::new();
    
    // Testar solicitação de capability
    // Nota: DMA requer IOMMU, então testamos Timer
    let result = manager.request(1, ModuleCapType::Timer { id: 0 });
    assert!(result.is_ok());
    
    crate::ktrace!("[Module Test] OK: Capability system");
}

fn test_watchdog() {
    crate::ktrace!("[Module Test] test_watchdog");
    
    use super::watchdog::{ModuleWatchdog, HealthStatus};
    use super::ModuleId;
    
    let mut watchdog = ModuleWatchdog::new();
    watchdog.init();
    
    let id = ModuleId::new(999);
    watchdog.register(id);
    
    assert_eq!(watchdog.get_status(id), Some(HealthStatus::Healthy));
    
    watchdog.heartbeat(id);
    assert_eq!(watchdog.get_status(id), Some(HealthStatus::Healthy));
    
    watchdog.unregister(id);
    assert_eq!(watchdog.get_status(id), None);
    
    crate::ktrace!("[Module Test] OK: Watchdog");
}
