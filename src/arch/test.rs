#[cfg(feature = "self_test")]
pub fn run_tests() {
    test_cpu_interrupts();
    test_io_ports();
}

#[cfg(feature = "self_test")]
fn test_cpu_interrupts() {
    use crate::arch::Cpu;
    use crate::arch::traits::CpuTrait;
    
    // Desabilitar e verificar
    Cpu::disable_interrupts();
    assert!(!Cpu::interrupts_enabled());
    
    // Habilitar e verificar
    Cpu::enable_interrupts();
    assert!(Cpu::interrupts_enabled());
    
    // Deixar desabilitado no final
    Cpu::disable_interrupts();
}

#[cfg(feature = "self_test")]
fn test_io_ports() {
    use crate::arch::x86_64::ports;
    
    // Testar porta 0x80 (POST codes, sempre funciona)
    ports::outb(0x80, 0xAA);
    // Não podemos ler de volta (write-only), mas não deve crashar
}
