# Documentação do Sistema de Drivers (`src/drivers`)

> **Caminho**: `src/drivers`  
> **Responsabilidade**: Gerenciamento de dispositivos de hardware e barramentos (Bus).  
> **Modelo**: Device Tree / Driver Binding dinâmico.

---

## 🏛️ O Modelo de Drivers

O RedstoneOS adota um modelo hierárquico de dispositivos.
1.  **Device**: Uma instância física ou virtual de hardware (ex: "Placa de Rede Intel E1000").
2.  **Driver**: O código de software que sabe controlar aquele hardware.
3.  **Bus**: O canal de comunicação onde dispositivos vivem (PCI, USB, Platform).

O processo de **Matching** conecta um `Driver` a um `Device` compatível (via VendorID/DeviceID).

---

## 📂 Implementações (`src/drivers/`)

### 1. `mod.rs` (O Orquestrador)
Contém a função `init()`, que dispara a descoberta de hardware na ordem correta:
1.  Drivers Base (System Timer, Serial).
2.  Barramentos principais (PCI Scan).
3.  Drivers de Vídeo.

### 2. `pci/` (Peripheral Component Interconnect)
O barramento mais importante em x86_64.
*   Enumera dispositivos conectados.
*   Lê o Header de Configuração PCI (Vendor ID, Device ID, BARs).
*   Carrega o driver apropriado se disponível.

### 3. Categorias de Drivers

| Diretório | Tipo de Dispositivo | Exemplos |
|:----------|:--------------------|:---------|
| `serial/` | UART / COM Ports | `serial.rs` (debug log) |
| `timer/`  | Relógios de Hardware| `pit.rs` (Programmable Interval Timer), `hpet.rs`, `lapic.rs` |
| `input/`  | Dispositivos de Entrada | Teclado PS/2, Mouse, USB HID (futuro) |
| `display/`| Vídeo | VESA, GOP (UEFI), Drivers nativos (GPU) |
| `net/`    | Rede | Drivers E1000, Realtek, VirtIO-Net |
| `block/`  | Armazenamento | AHCI (SATA), NVMe, VirtIO-Blk |

---

## 🔧 Exemplo de Fluxo de Inicialização (PCI)

1.  **Scan**: O módulo `pci` percorre todos os barramentos (0-255), dispositivos (0-31) e funções (0-7).
2.  **Discovery**: Encontra um dispositivo com `Vendor=0x8086` e `Device=0x100E` (Intel E1000).
3.  **Lookup**: Consulta a tabela de drivers registrados. Encontra o driver `e1000`.
4.  **Probe**: Chama `e1000::probe(pci_device)`.
5.  **Init**: O driver configura o hardware, aloca buffers de DMA e registra uma interface de rede no kernel.
6.  **IRQ**: O driver registra um tratador de interrupção para receber pacotes.

---

## ⚠️ Abstração de Hardware

Para manter os drivers portáveis e seguros:
*   Drivers **nunca** acessam portas de I/O arbitrariamente. Usam wrappers como `Port<u8>`.
*   Acesso a memória de dispositivo (MMIO) é feito via `Volatile` reads/writes em regiões mapeadas como `Uncacheable` pelo VMM.
*   Interrupções devem ser curtas e rápidas. Processamento pesado deve ser adiado (Deferred Work).
