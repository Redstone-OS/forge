# Guia completo de refatoração do Redstone OS (Forge)

## 1. Veredito Executivo: A Fundação é Sólida

A reestruturação do `forge` atingiu o objetivo de romper com o passado. A estrutura apresentada **não é** apenas uma reorganização cosmética; ela reflete uma mudança fundamental de filosofia, alinhando-se quase perfeitamente com os princípios de "Micro-Modularidade", "Segurança" e "Desconfiança" definidos na constituição do projeto.

O layout atual **permite** a implementação da visão de "Guest with a Badge" (Módulos supervisionados), embora a implementação real dessa lógica (o código "carne") ainda esteja em estágio embrionário em muitos arquivos.

---

## 2. Aderência aos Princípios (Análise Deep-Dive)

### ✅ Modularidade e Isolamento (Nota: A)
A separação entre `core`, `arch` e `drivers` está cristalina.
*   **`arch/`**: Contém apenas o que é "sujo" e específico da CPU. O resto do kernel desconhece assembly.
*   **`core/`**: Atua puramente como orquestrador lógico. A subdivisão em `object` (handles), `work` (tasks) e `power` demonstra uma arquitetura orientada a serviços, não apenas um "monolito espaguete".
*   **[module/](file:///D:/Github/RedstoneOS/forge/src/module/mod.rs#102-106)**: Este é o ponto alto. A existência de `verifier.rs`, `sandbox.rs` e `capability.rs` prova que o sistema de carregamento de drivers foi desenhado para ser "Zero Trust" desde o dia zero.

### ✅ Segurança e Type Safety (Nota: A-)
*   **Encapsulamento**: O uso de `Result` ao invés de pânicos é visível nas assinaturas (ex: `module::load` retorna `Result`).
*   **Abstração de Objetos**: A pasta `core/object` (com `handle.rs`, `dispatcher.rs`, `rights.rs`) sugere fortemente uma segurança baseada em **Capabilities** (semelhante ao Windows NT ou Zircon), onde você não tem acesso a memória, mas sim a um "Handle" com "Direitos". Isso é infinitamente mais seguro que o modelo UNIX tradicional (tudo é arquivo/permissão global).

### ⚠️ Assincronismo e Modernidade (Nota: B em Design, C em Implementação)
*   **Intenção**: O usuário pediu um kernel "Assíncrono". A estrutura tem `core/work/deferred.rs` e `workqueue.rs`, o que é um bom começo (estilo Linux Softirq/Tasklets).
*   **Falta o Executor**: Ao inspecionar [sched/mod.rs](file:///D:/Github/RedstoneOS/forge/src/sched/mod.rs) e [drivers/base/driver.rs](file:///D:/Github/RedstoneOS/forge/src/drivers/base/driver.rs), **não encontrei** menção a `Future`, `Waker` ou `Executor`.
    *   *Crítica*: Para ser um kernel *realmente* moderno e assíncrono (Rust-native), os drivers deveriam expor `async fn read()`. Atualmente, a estrutura sugere um modelo mais tradicional de interrupção/callback. É necessário decidir se vai adotar `async/await` no kernel.

### 🧪 Maturidade do Código (Nota: Esqueleto)
Muitos arquivos vitais são apenas "esqueletos" ou contêm TODOs críticos.
*   [drivers/base/driver.rs](file:///D:/Github/RedstoneOS/forge/src/drivers/base/driver.rs) contém apenas `// TODO: Driver trait`. Isso significa que o "contrato" entre Kernel e Driver ainda não existe em código.
*   **Contexto FPU**: [sched/mod.rs](file:///D:/Github/RedstoneOS/forge/src/sched/mod.rs) admite honestamente que o **Context Switch de FPU/SSE** está ausente (TODO Crítico), o que corromperia processos de usuário modernos.

---

## 3. Análise Pasta por Pasta

### 📂 `src/module` (A Joia da Coroa)
Esta pasta valida a arquitetura. Ao invés de o kernel confiar cegamente em drivers (`.sys`/`.ko`), ele tem um subsistema de **Supervisão**.
*   **Forte**: `sandbox.rs` e `verifier.rs` indicam que drivers serão tratados como código de terceiros, mesmo rodando em Ring 0.
*   **Faltando**: A conexão com o alocador de memória. Um módulo precisa de um heap isolado ou limitado para evitar que um driver consuma toda a RAM do sistema.

### 📂 `src/ipc` (O Barramento)
A presença de `ipc/channel`, `ipc/port` e `ipc/message` como cidadãos de primeira classe no nível raiz (`src/ipc` e não escondido em `src/core`) é excelente. Isso alinha-se com a ideia de micro-modularidade: se os serviços estão separados, a comunicação (IPC) é a artéria vital.

### 📂 `src/sched` (O Motor)
Bem organizado em `context`, `task`, `scheduler`.
*   **Alerta**: O código admite usar um "Global Lock" (`SCHEDULER` Mutex). Em um design moderno, deveríamos ter **Per-CPU Runqueues** (`smp/percpu.rs` existe, mas precisa ser integrado aqui).

### 📂 `src/drivers` (A Zona de Transição)
Atualmente está misturada no código fonte (`src/drivers/net`, `src/drivers/pci`).
*   **Observação**: Para um kernel modular, o ideal seria que `src/drivers` contivesse apenas as **Interfaces (Traits)** e a infraestrutura de barramento (`pci`, `usb`). Os drivers de dispositivo específicos (como `e1000`, `nvme`) deveriam, idealmente, ser "crates" separados na pasta raiz do repositório (fora de `forge/src`) que compilam para os módulos binários que o `src/module` carrega.

---

## 4. Recomendações e Próximos Passos

Baseado na regra "Compatibilidade só se custo 0" e "Fazer melhor":

1.  **Definir o `Driver Trait` (Prioridade Máxima)**
    *   Preencha `src/drivers/base/driver.rs`. Defina o que *é* um driver. Ele tem `init()`, `probe()`, `remove()`?
    *   *Sugestão Moderna*: Adicione `async fn handle_irq()` se for seguir o caminho async.

2.  **Mover Drivers Específicos para Fora**
    *   Para provar a modularidade, mova `virtio_net.rs` ou `ahci.rs` para uma pasta de exemplos. Eles não devem ter acesso privilegiado a `core/` via `pub use`, mas sim apenas via `sys/abi`.

3.  **Endurecer o `syscall`**
    *   Syscalls são a única porta de entrada. Garanta que `src/syscall/dispatch` seja gerado automaticamente ou extremamente rígido.

4.  **Resolver a questão FPU/SSE**
    *   Você proibiu float no kernel (Correto), mas o SO *precisa* salvar o estado float dos apps. O `sched/context` precisa de campos para `XRSTOR`/`XSAVE` area.

## 5. Conclusão Final

A arquitetura está **Aprovada**. Ela é ambiciosa, limpa e evita as armadilhas de "fazer como o UNIX fazia". O esqueleto suporta o peso de um sistema operacional moderno e seguro. O trabalho agora é preencher as lacunas (interfaces de driver e contexto de CPU completo) sem comprometer essa organização.

# Mapa Completo do Projeto Redstone OS (Forge)

Este documento detalha a estrutura de diretórios e arquivos do kernel `forge`, sem "encheção de linguiça". Aqui está o que cada peça faz na máquina.

---

## 🏗️ Visão Geral da Estrutura
O jogo é: **Dependências fluem de Cima para Baixo**.
1. `core` e [mm](file:///D:/Github/RedstoneOS/forge/src/module/mod.rs#73-84) são a base.
2. `drivers` e `fs` dependem da base.
3. `syscall` expõe tudo isso para o usuário.

---

## 📂 `src/arch` (Hardware Abstraction Layer - HAL)
**Propósito:** Isolar todo o código específico de CPU. O resto do kernel não deve saber que está rodando em x86_64.
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Ponto de entrada e re-exports da arquitetura atual.

### `arch/traits` (O Contrato)
Define *o que* o hardware pode fazer, sem dizer *como*.
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Módulo raiz dos traits.
*   `cpu.rs`: Define métodos abstratos como `halt()`, `disable_interrupts()`, `current_core_id()`.

### `arch/x86_64` (A Implementação)
Código "sujo" com Assembly e registradores específicos.
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Inicialização da CPU (Checagem de CPUID, features).
*   `cpu.rs`: Implementação dos traits para Intel/AMD. Leitura de MSRs, CR0, CR3.
*   `gdt.rs`: **Global Descriptor Table**. Configura segmentos de memória (Kernel Code/Data, User Code/Data, TSS).
*   `idt.rs`: **Interrupt Descriptor Table**. Tabela que aponta para os handlers de exceção (#PF, #GP) e IRQs.
*   `interrupts.rs`: Handlers Rust para as interrupções definidas na IDT.
*   `memory.rs`: Funções de manipulação física bruta (setup inicial de paginação).
*   `ports.rs`: Abstração para instruções `inb`/`outb` (IO Ports legadas).
*   `switch.s` (Assembly): Código crítico para troca de contexto (salva/restaura registradores RBP, RSP, R12-R15).
*   `syscall.rs`: Configura os MSRs (`LSTAR`, `STAR`) para habilitar a instrução `SYSCALL`.
*   `syscall.s` (Assembly): O "trampolim" de entrada/saída da syscall (troca de stack user->kernel).

#### `arch/x86_64/acpi` (Configuração de Energia/Hardware)
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Parser base das tabelas ACPI.
*   `dsdt.rs`: *Differentiated System Description Table*. Descreve periféricos integrados.
*   `fadt.rs`: *Fixed ACPI Description Table*. Ponteiros para controle de energia.
*   `madt.rs`: *Multiple APIC Description Table*. Essencial para **SMP** (descobre quantos cores existem).

#### `arch/x86_64/apic` (Controlador de Interrupções Avançado)
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Inicialização do subsistema APIC.
*   `ioapic.rs`: **I/O APIC**. Roteia interrupções externas (Teclado, Rede) para CPUs específicas.
*   `lapic.rs`: **Local APIC**. Timer local por core e envio de IPIs (Inter-Processor Interrupts).

#### `arch/x86_64/iommu` (Isolamento de Hardware)
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Detecção de IOMMU.
*   `intel_vtd.rs`: Intel VT-d. Protege a RAM contra escritas DMA maliciosas de drivers.

---

## 📂 `src/core` (Orquestração Lógica)
**Propósito:** O "cérebro" do sistema. Gerencia o fluxo de vida do kernel, sem se preocupar com bits de hardware.

### `core/boot` (Inicialização)
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Sequência de boot lógica.
*   `cmdline.rs`: Parser dos argumentos de boot (ex: `debug=on root=/dev/nvme0`).
*   `entry.rs`: O ponto de entrada Rust (`_start`). Chama a inicialização de subsistemas na ordem correta.
*   `handoff.rs`: Define a estrutura `BootInfo` recebida do `ignite` (mapa de memória, framebuffer).
*   `initcall.rs`: Sistema para registrar funções que rodam no boot automaticamente (similar ao Linux `module_init`).
*   `panic.rs`: Handler de pânico (`#[panic_handler]`). Para o sistema e exibe erro.

### `core/debug` (Diagnóstico)
*   `kdebug.rs`: Ferramentas para *breakpoint* de software e inspeção.
*   `klog.rs`: O sistema de logs estruturados (`kinfo!`, `kerror!`). Deve usar serial output.
*   `oops.rs`: Trata erros recuperáveis (diferente de panic). Ex: matar uma thread que falhou, mas manter o OS.
*   `stats.rs`: Contadores globais (uptime, syscalls/sec).
*   `trace.rs`: Infraestrutura para tracing de performance (estilo ftrace).

### `core/object` (Gerenciamento de Recursos - Capability Based)
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Definições básicas.
*   `dispatcher.rs`: Encontra o objeto real dado um `Handle`.
*   `handle.rs`: O "ponteiro seguro" que o userspace segura (um `u32` opaco).
*   `kobject.rs`: Trait base para tudo que o kernel gerencia (Processo, Thread, VMO, Canal).
*   `refcount.rs`: Contagem de referência atômica para gerenciamento de vida dos objetos.
*   `rights.rs`: Define o que pode ser feito com um handle (ex: `READ`, `WRITE`, `EXECUTE`, `TRANSFER`).

### `core/power` (Gestão de Energia)
*   `cpufreq.rs`: Escalonamento de frequência da CPU (Performance vs Bateria).
*   `cpuidle.rs`: Coloca a CPU em estados de baixo consumo (C-States) quando ociosa.
*   `state.rs`: Máquina de estados de energia global (Running, Suspending).
*   `suspend.rs`: Lógica para suspender para RAM (S3) ou Disco (S4).

### `core/smp` (Multiprocessamento)
*   `bringup.rs`: Lógica para acordar os cores secundários (APs).
*   `ipi.rs`: Envia mensagens entre CPUs (ex: "Pare para panic", "Flush TLB").
*   `percpu.rs`: Define variáveis locais por CPU (ex: ponteiro para a Thread atual).
*   `topology.rs`: Entende a topologia do processador (Cores, Sockets, Threads/Hyperthreading).

### `core/time` (Relógio do Sistema)
*   `clock.rs`: Mantém a hora do dia (Wall Clock).
*   `hrtimer.rs`: Timers de alta resolução para agendamento preciso.
*   `jiffies.rs`: Contador monótono simples (ticks desde o boot).
*   `timer.rs`: interface de timer genérico.

### `core/work` (Trabalho Diferido)
*   `deferred.rs`: Executa funções "mais tarde" (fora do contexto de interrupção crítica).
*   `tasklet.rs`: Pequenas tarefas de alta prioridade.
*   `workqueue.rs`: Filas de trabalho processadas por threads de kernel (pode dormir/bloquear).

---

## 📂 `src/drivers` (Drivers e Barramentos)
**Propósito:** Conectar o hardware aos subsistemas do kernel. *Nota: No futuro, implementações complexas sairão daqui para módulos.*

### `drivers/base` (O Modelo de Driver)
*   `bus.rs`: Abstração de barramento (PCI, USB). Itera sobre dispositivos.
*   `class.rs`: Classificação de dispositivos (ex: "é uma Placa de Rede", "é um Disco").
*   `device.rs`: Representa uma instância de hardware físico.
*   [driver.rs](file:///D:/Github/RedstoneOS/forge/src/drivers/base/driver.rs): A Trait que todo driver deve implementar (`probe`, `remove`, `suspend`).
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Registro global de drivers.

### Subpastas Específicas
*   `block/`: Drivers de armazenamento.
    *   `ahci.rs`: Controladora SATA.
    *   `nvme.rs`: SSDs modernos rápidos.
    *   `ramdisk.rs`: Disco na memória (usado para Initramfs).
*   `input/`: Teclado/Mouse (PS/2 ou USB Legacy).
*   `irq/`: Controladores de interrupção (glue code).
*   `net/`: Placas de rede (ex: `virtio_net.rs` para VMs).
*   `pci/`: O Barramento PCI Express.
    *   `config.rs`: Leitura/Escrita no espaço de configuração PCI.
    *   `pci.rs`: Enumeração de dispositivos ("Quem está plugado?").
*   `serial/`: Porta serial (UART) para logs de debug.
*   `timer/`: Fontes de tempo de hardware (`hpet` high precision, `pit` legacy, `tsc` cpu cycle counter).
*   `video/`: Saída gráfica.
    *   `framebuffer.rs`: Gerencia o buffer de pixels linear (GOP/UEFI).
    *   `font.rs`: Renderização de texto simples para o terminal do kernel.

---

## 📂 `src/fs` (Sistema de Arquivos)
**Propósito:** Abstração unificada para acesso a dados.

*   `devfs.rs`: Cria arquivos virtuais para dispositivos (`/dev/null`, `/dev/sda`).
*   `initramfs.rs`: O sistema de arquivos temporário carregado na memória durante o boot.
*   `vfs.rs`: A lógica central. Resolve caminhos (`/usr/bin`) para inodes.

### `fs/vfs` (Virtual File System)
*   `dentry.rs`: Cache de diretórios (Mapeia "nome" -> Inode). Acelera lookups.
*   `file.rs`: Representa um arquivo *aberto* (cursor de leitura, modo de acesso).
*   `inode.rs`: Metadados do arquivo (tamanho, permissões, onde estão os blocos).
*   `mount.rs`: Gerencia pontos de montagem.
*   `path.rs`: Utilitários para parsing de strings de caminho.

### Outros FS
*   `procfs/`, `sysfs/`: Sistemas de arquivos virtuais para expor estado do kernel para o usuário.
*   `tmpfs/`: Filesystem volátil na RAM (storage temporário).

---

## 📂 `src/ipc` (Inter-Process Communication)
**Propósito:** "O Sistema Nervoso". Como processos conversam em um microkernel/modular.

*   `message.rs`: Define o envelope da mensagem (cabeçalho + payload + handles).
*   `port.rs`: Endpoint de comunicação. Quem tem a porta, recebe a mensagem.
*   `channel/`: Comunicação 1:1 bidirecional.
*   `futex/`: *Fast Userspace Mutex*. Primitiva para threads dormirem/acordarem (usado para implementar Mutex em userspace).
*   `pipe/`: Fluxo de bytes unidirecional (estilo UNIX `|`).
*   `shm/`: **Shared Memory**. Compartilha páginas físicas entre dois processos (zero-copy).

---

## 📂 `src/klib` (Biblioteca do Kernel)
**Propósito:** Estruturas de dados e utilitários seguros. *Substitui a `std`.*

*   `align.rs`: Funções para alinhamento de memória.
*   `bitmap.rs`: Gerenciamento eficiente de bits (usado no PMM).
*   `mem_funcs.rs`: `memcpy`, `memset` otimizados e seguros.
*   `hash/`: Tabela Hash (para Dentry cache e Object map).
*   [list/](file:///D:/Github/RedstoneOS/forge/src/module/mod.rs#102-106): Lista duplamente ligada intrusiva (padrão de kernel).
*   `tree/`: Red-Black Tree (para o Scheduler e VMA do VMM).
*   `string/`: Manipulação de strings segura (sem alocação excessiva).

---

## 📂 `src/mm` (Memory Management)
**Propósito:** Gerenciar a RAM física e Virtual.

*   `oom.rs`: *Out of Memory Killer*. Decide quem morre quando acaba a RAM.

### `mm/alloc` (Alocadores de Heap)
*   `buddy.rs`: Alocador de páginas físicas (divide a RAM em potências de 2).
*   `slab.rs`: Alocador de objetos pequenos (cache de estruturas fixas).
*   `bump.rs`: Alocador simples e rápido (apenas avança ponteiro) para boot inicial.

### `mm/pmm` (Physical Memory Manager)
*   `frame.rs`: Abstração de um frame físico (4KB).
*   [mod.rs](file:///D:/Github/RedstoneOS/forge/src/core/mod.rs): Interface pública para pedir RAM física.
*   `zones.rs`: Divide RAM em zonas (DMA < 16MB, Normal, HighMem).

### `mm/vmm` (Virtual Memory Manager)
*   `mapper.rs`: Manipula as Page Tables da CPU (mapeia Virtual -> Físico).
*   `tlb.rs`: Gerencia o *Translation Lookaside Buffer* (flush quando muda mapa).
*   `vmm.rs`: Gerencia o espaço de endereçamento de um processo (VMAs).
*   `ops/`: Abstração para operações de memória (para evitar `unsafe` direto).

---

## 📂 `src/module` (Sistema de Módulos)
**Propósito:** Carregar código dinâmico (Drivers) de forma segura.

*   `loader.rs`: Parser de ELF relocável (`.ko`).
*   `verifier.rs`: Verifica assinatura criptográfica do módulo.
*   `sandbox.rs`: Configura restrições (o que o módulo pode acessar).
*   `supervisor.rs`: Monitora o módulo rodando.
*   `watchdog.rs`: Detecta módulos travados.
*   `abi.rs`: A interface binária estável que os módulos usam para falar com o kernel.

---

## 📂 `src/sched` (Scheduler - O Motor)
**Propósito:** Decidir qual tarefa roda na CPU.

*   `context/`: Salva/Restaura registradores GP e FPU/SSE.
*   `exec/`: Carregadores de executáveis.
    *   `elf/`: Carrega binários ELF estáticos/dinâmicos.
    *   `spawn/`: Criação de novo processo.
*   `scheduler/`: Algoritmo de decisão (Round Robin / Priority).
    *   `runqueue.rs`: Fila de tarefas prontas para rodar.
*   `task/`: Definição de Processo e Thread.
    *   `state.rs`: Estados (Ready, Running, Blocked, Zombie).
*   `wait/`: Wait Queues. Threads dormem aqui esperando eventos.

---

## 📂 `src/security` (Segurança)
**Propósito:** Auditoria e Controle de Acesso.

*   `capability/`: Implementação do sistema de capabilities.
*   `audit/`: Log de segurança (quem fez o quê, quando).
*   `credentials/`: Quem é este processo? (UIDs, SIDs, Tokens).
*   `sandbox/`: Namespaces e isolamento extra.

---

## 📂 `src/sync` (Sincronização)
**Propósito:** Primitivas para evitar *Data Races* em SMP.

*   `atomic/`: Wrappers para operações atômicas da CPU.
*   `mutex.rs`: Bloqueio com suspensão de thread (pode dormir).
*   `spinlock.rs`: Bloqueio com loop ativo (NÃO pode dormir, apenas para seções críticas curtas).
*   `rwlock.rs`: Read-Write Lock (muitos leitores, um escritor).
*   `rcu/`: *Read-Copy-Update*. Avançado. Permite leitura sem lock, ideal para listas muito lidas.
*   `semaphore.rs`: Controle de contagem de recursos.

---

## 📂 `src/sys` & `src/syscall` (Interface User/Kernel)
**Propósito:** Definir e implementar a fronteira com o mundo exterior.

*   `sys/`: Definições compartilhadas (códigos de erro, structs C-compatible).
*   `syscall/`: A implementação.
    *   `dispatch/`: Tabela de despacho (Número Syscall -> Função Rust).
    *   `numbers.rs`: Lista de números das syscalls (ex: `SYS_READ = 0`).
    *   `abi/`: Validação de argumentos vindos do userspace (segurança crítica).
    *   `fs/`, `ipc/`, `memory/`: Wrappers que chamam o subsistema real após validar.
