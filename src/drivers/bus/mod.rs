//! # Infraestrutura de Barramentos (Bus Infrastructure)
//!
//! A camada de barramento do RedstoneOS é responsável pela ponte entre o
//! hardware físico e o Redstone Driver Model (RDM).
//!
//! ## 🔬 Responsabilidades:
//! 1. **Enumeração**: Percorrer slots, portas ou registros para encontrar hardware.
//! 2. **Abstração de Endereço**: Gerenciar BDF (PCI), Portas (USB) ou MMIO (Platform).
//! 3. **Isolamento**: Fornecer meios para resetar dispositivos sem afetar o resto do sistema.
//! 4. **Encapsulamento**: Armazenar dados técnicos de baixo nível (PciDeviceInfo, UsbSpeed)
//!    para que os drivers funcionais não precisem lidar com a "sujeira" do barramento.
//!
//! ## 🛠️ Tipos de Barramentos Implementados:
//! - **ACPI**: Barramento lógico de configuração e energia (CPUs, IOAPICs).
//! - **PCI/PCIe**: O barramento principal de alta velocidade (Placas de Rede, Discos).
//! - **USB**: Barramento serial universal (Teclado, Mouse, Pen-drives).
//! - **ISA/LPC**: Barramento legado para periféricos de sistema (COM1, RTC).
//! - **Platform**: Dispositivos fixos na placa-mãe ou em memória MMIO.
//! - **VirtIO**: Barramento virtual para comunicação eficiente com o Hypervisor (QEMU).
//!
//! ## ⚙️ Fluxo de Inicialização:
//! A inicialização segue uma ordem de dependência rigorosa:
//! 1. Barramentos estáticos/lógicos (ACPI, ISA, Platform).
//! 2. Barramentos de descobrimento físico (PCI).
//! 3. Barramentos complexos que dependem de outros (USB depende de PCI/MMIO).
//! 4. Barramentos de virtualização (VirtIO depende de PCI/MMIO).

pub mod acpi;
pub mod isa;
pub mod pci;
pub mod platform;
pub mod usb;
pub mod virtio;

/// Centraliza a ativação de todos os barramentos do kernel.
/// Deve ser chamado pelo `drivers::init()` geral.
pub fn init() {
    crate::kinfo!("(Bus) Inicializando infraestrutura de barramentos...");

    // 1. ACPI (Configuração básica do sistema e topologia)
    // Nota histórica: Em arquiteturas PC, o ACPI deve ser lido antes do PCI
    // para saber sobre o roteamento de interrupções.
    // acpi::init(rsdp);

    // 2. ISA/LPC (Legacy Support)
    // Expõe dispositivos como COM1 e Teclado PS/2.
    isa::init();

    // 3. Platform (Maquinário fixo)
    // Registra hardware MMIO mapeado estaticamente.
    platform::init();

    // 4. PCI/PCIe (Discovery de Alta Performance)
    // O motor principal de descoberta de hardware moderno.
    pci::init();

    // 5. USB (Universal Serial Bus)
    // Inicializa o Core e registra os Host Controllers (xhci/ehci).
    usb::init();

    // 6. VirtIO (Transporte Virtualizado)
    // Inicializa o barramento de mensagens para ambientes cloud/QEMU.
    virtio::init();

    crate::kdebug!("(Bus) Todos os barramentos operacionais.");
}
