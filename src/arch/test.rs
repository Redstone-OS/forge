/// Arquivo: arch/test.rs
///
/// Propósito: Testes de unidade e integração para a camada de arquitetura (HAL).
///
/// Detalhes de Implementação:
/// - Executado apenas quando a feature "self_test" está ativa.
/// - Verifica o contrato básico do trait CPU.
/// - Verifica se as portas de IO não causam exceções.

#[cfg(feature = "self_test")]
pub fn run_tests() {
    crate::kinfo!("Iniciando testes de arquitetura...");
    test_cpu_interrupts();
    test_io_ports();
    crate::kinfo!("Testes de arquitetura concluídos com SUCESSO.");
}

#[cfg(feature = "self_test")]
fn test_cpu_interrupts() {
    // Nota: Usamos caminhos explícitos para garantir que estamos testando
    // a implementação correta independentemente de re-exports no mod.rs
    use crate::arch::x86_64::cpu::Cpu;
    // Ajuste: O usuário renomeou traits para _traits
    use crate::arch::_traits::cpu::CpuTrait;
    
    crate::klog!("Testando controle de interrupções...");

    // 1. Desabilitar e verificar
    Cpu::disable_interrupts();
    assert!(!Cpu::interrupts_enabled(), "Interrupções deveriam estar desabilitadas após cli");
    
    // 2. Habilitar e verificar
    Cpu::enable_interrupts();
    assert!(Cpu::interrupts_enabled(), "Interrupções deveriam estar habilitadas após sti");
    
    // 3. Deixar desabilitado no final (estado seguro para outros testes)
    Cpu::disable_interrupts();
}

#[cfg(feature = "self_test")]
fn test_io_ports() {
    use crate::arch::x86_64::ports;
    
    crate::klog!("Testando IO Ports (0x80)...");
    
    // Testar porta 0x80 (POST codes, sempre funciona e é inofensiva)
    // Escrever 0xAA é um padrão comum de debug
    ports::outb(0x80, 0xAA);
    
    // Não podemos ler de volta (write-only), mas se não crashar (GPF), o teste passou.
}
