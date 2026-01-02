# 🔌 Subsistema de Drivers - RedstoneOS

O subsistema de drivers do RedstoneOS (Forge Kernel) é o motor que traduz as intenções do kernel e das aplicações em sinais elétricos no hardware. Ele foi projetado para ser modular, extensível e seguro, utilizando as garantias de tipagem do Rust para gerenciar acessos a I/O e memória.

---

## 🏛️ Arquitetura de Drivers

O modelo segue uma hierarquia de quatro camadas:

```text
┌─────────────────────────────────────────────────────────┐
│ 1. Subsystems (Frameworks)                              │
│    VFS (Block), Networking (Net), Input Stack (Keyboard)  │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│ 2. Device Drivers (Lógica Específica)                   │
│    ATA Driver, VirtIO-BLK, PS/2 Keyboard                │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│ 3. Bus Controllers (Descoberta & Transporte)            │
│    PCI Bus, USB Host Controller, Platform Bus           │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│ 4. Hardware Abstraction Layer (HAL)                     │
│    I/O Ports, Memory Mapped I/O (MMIO), DMA, IRQs       │
└─────────────────────────────────────────────────────────┘
```

---

## 🗺️ Mapa do Módulo (`src/drivers`)

### 📦 Armazenamento (`block/`)
Responsável por dispositivos de bloco (setores de 512 bytes ou 4KB).
- **`traits.rs`**: Define o `BlockDevice` trait, a interface universal para o kernel ler/escrever em discos.
- **`ata.rs`**: Driver ATA/IDE legacy usando modo PIO. Essencial para compatibilidade com o modo `fat:rw:` do QEMU.
- **`virtio_blk.rs`**: Driver moderno de alta performance para ambientes virtualizados.
- **`virtqueue.rs`**: Infraestrutura de filas circulares para comunicação VirtIO.

### 🚌 Barramentos (`pci/`)
O espinha dorsal da descoberta de hardware em arquiteturas modernas.
- **`pci.rs`**: Implementa o escaneamento recursivo do barramento PCI, identificando dispositivos via Vendor/Device IDs.
- **`config.rs`**: Acesso ao espaço de configuração PCI (registros de 32 bits).

### ⌨️ Entrada (`input/`)
- **`keyboard.rs`**: Driver de teclado PS/2 com suporte a Scancodes e estados de teclas.

### 📺 Gráficos (`display/`)
- **`vga.rs`**: Modo texto clássico 80x25.
- **`framebuffer/`**: (Planejado) Abstração gráfica para resoluções modernas via VESA/GOP.

### 🕒 Tempo & Interrupções (`timer/`, `irq/`)
- **`pit.rs`**: Programmable Interval Timer para ticks de sistema básicos.
- **`pic.rs`**: Programmable Interrupt Controller legacy.

---

## 💿 Foco: Dispositivos de Bloco (Block IO)

A grande inovação recente foi a unificação de dispositivos de bloco sob um único trait, permitindo ao sistema de arquivos (FAT) operar sem saber a tecnologia do disco abaixo dele.

### O Trait `BlockDevice`
```rust
pub trait BlockDevice: Send + Sync {
    fn read_block(&self, sector: u64, buf: &mut [u8]) -> Result<(), BlockError>;
    fn write_block(&self, sector: u64, buf: &[u8]) -> Result<(), BlockError>;
    fn block_size(&self) -> usize;
    fn total_blocks(&self) -> u64;
}
```

### Ordem de Inicialização (Business Logic)
O kernel segue uma heurística de prioridade para dispositivos de boot:
1. **ATA/IDE**: Verificado primeiro para suportar discos de desenvolvimento rápidos.
2. **VirtIO-BLK**: Verificado em seguida para máxima performance em produção cloud/VM.
3. **NVMe/AHCI**: (Planejado) Para máquinas reais.

---

## 🔍 Processo de Descoberta (PCI Discovery)

O RedstoneOS realiza um escaneamento dinâmico no boot:
1. **Enumeration**: Percorre todos os Slots PCI e lê o Device ID.
2. **Registration**: O kernel mantém uma lista global de dispositivos encontrados.
3. **Driver Binding**:
   - O Driver de Bloco pede ao barramento: "Me dê o primeiro dispositivo que se identifique como VirtIO Storage".
   - Se encontrado, o driver toma controle do dispositivo e o registra no VFS.

---

## 🛡️ Segurança e Boas Práticas

1. **Isolation de I/O**: Drivers nunca usam instruções `in` ou `out` brutas. Eles usam a estrutura `Port<T>` que garante operações atômicas e seguras.
2. **Volatile Memory**: Todo acesso a hardware via MMIO é feito através de ponteiros voláteis, impedindo que o compilador Rust otimize e remova lógicas de controle vitais.
3. **Arc & Mutex**: Dispositivos são protegidos por `Arc<Spinlock<T>>` para permitir acesso seguro por múltiplos núcleos de CPU durante operações assíncronas de I/O.

---

## 🔮 Roadmap de Hardware

- [ ] **DMA (Direct Memory Access)**: Migrar o driver ATA de PIO para DMA para liberar a CPU durante transferências.
- [ ] **MSI/MSI-X**: Substituir interrupções legadas por Message Signaled Interrupts para melhor escalabilidade em servidores.
- [ ] **USB Stack**: Iniciar o suporte a drivers XHCI e dispositivos HID.
- [ ] **AHCI/SATA**: Driver completo para discos modernos de máquinas reais.

---
*Atualizado em Janeiro de 2026 pelo Forge Kernel Team.*
