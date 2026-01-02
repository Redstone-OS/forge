# Documentação do Kernel Core (`src/core`)

> **Caminho**: `src/core`  
> **Responsabilidade**: Infraestrutura essencial, startup do sistema e gerenciamento de CPUs. Código agnóstico de hardware.

---

## 🏛️ A "Cola" do Kernel

O módulo `core` atua como o integrador central. Enquanto `mm` cuida da memória e `sched` do tempo, o `core` cuida do **Kernel em si** como uma aplicação.

---

## 📂 Subsistemas Principais

### 1. `boot/` (A Gênese)
O ponto de entrada do kernel (`kernel_main`) reside aqui.
*   **Handoff**: Recebe a estrutura `BootInfo` do bootloader (Mapa de memória, Framebuffer, ACPI tables).
*   **Orquestração**: Chama `mm::init`, `arch::init`, `sched::init`, `drivers::init` na ordem correta.
*   **Panic**: Contém o `panic_handler`, a última função q roda quando tudo dá errado (Tela Vermelha/BSOD).

### 2. `smp/` (Symmetric Multi-Processing)
Gerencia múltiplos núcleos de CPU.
*   Detecta CPUs secundárias (APs) via ACPI/MADT.
*   Envia sinais de **IPI** (Inter-Processor Interrupts) para acordar outros núcleos ou forçar TLB Flush.
*   Mantém estruturas "Per-CPU" (variáveis locais de cada núcleo).

### 3. `time/`
Gerencia a noção de tempo do kernel.
*   `Jiffies`: Contador monótono de ticks desde o boot.
*   `WallTime`: Tempo real (Data/Hora) sincronizado com RTC ou NTP.

### 4. `power/`
Gerenciamento de energia (ACPI).
*   Reboot e Shutdown seguros.
*   Estados de suspensão (Sleep - S3/S4) [WIP].

### 5. `debug/`
Ferramentas para desenvolvedores do kernel.
*   `klogger`: Sistema de logs (`kinfo!`, `kerror!`) que escreve na Serial e na Tela.
*   `symbolizer`: Converte endereços de instrução em nomes de função (Stack Trace legível) durante um panic.

---

## 🚀 Fluxo de Boot (`kernel_main`)

1.  **Early Init**: Configura Serial Logger para termos output.
2.  **Arch Init**: GDT, IDT, Interrupções básicas.
3.  **MM Init**: Inicializa PMM, HHDM e Heap. (Agora temos alocação dinâmica!).
4.  **ACPI/SMP Init**: Descobre hardware e acorda outras CPUs.
5.  **Sched Init**: Prepara a primeira Task (Init) e configura o Timer.
6.  **Drivers Init**: PCI scan, Vídeo, Disco.
7.  **Mount FS**: Monta partição root.
8.  **Spawn Init**: Carrega `/bin/init` do userland.
9.  **Idle Loop**: O core de boot vira a Idle Task 0.

---

## 🛠️ Work Queues (`work/`)

Muitas vezes, uma interrupção precisa executar uma tarefa demorada (ex: processar um pacote de rede TCP), mas não podemos travar a CPU na interrupção.
*   **Solução**: A interrupção apenas enfileira um item na `WorkQueue`.
*   O Kernel possui threads worker em background que processam esses itens fora do contexto de interrupção.
