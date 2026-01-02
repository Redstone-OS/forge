# 📂 Sistema de Arquivos (FS) - RedstoneOS

O subsistema de Filesystem do RedstoneOS (Forge Kernel) é projetado como uma arquitetura de camadas modulares que abstrai dispositivos de armazenamento físico em uma hierarquia de arquivos e diretórios unificada e lógica.

---

## 🏛️ Arquitetura de Camadas

A arquitetura do FS é dividida em quatro níveis principais de abstração:

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ 1. Camada de Aplicação & Syscalls                                          │
│    open(), read(), write(), close(), lseek(), stat(), readdir()            │
└────────────────────────────────────────────────────────────────────────────┘
                                     ↓
┌────────────────────────────────────────────────────────────────────────────┐
│ 2. Virtual File System (VFS)                                               │
│    Abstracts: Inodes, File Handles, Dentries, Mount Points                 │
│    Logic: Path Resolution, Permissions, Operation Routing                  │
└────────────────────────────────────────────────────────────────────────────┘
                                     ↓
┌────────────────────────────────────────────────────────────────────────────┐
│ 3. Filesystem Backends (Drivers de FS)                                     │
│    InitRAMFS (TAR) │  FAT (16/32)  │  RFS (Advanced COW)  │  DevFS (Virtual)  │
└────────────────────────────────────────────────────────────────────────────┘
                                     ↓
┌────────────────────────────────────────────────────────────────────────────┐
│ 4. Block Device Interface (HAL)                                            │
│    BlockDevice Trait ← ATA/IDE Driver, VirtIO-BLK, RAMDisk                 │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 🧩 Mapa do Módulo (`src/fs`)

| Submódulo | Descrição | Status | Cabeçalho Principal |
|-----------|-----------|--------|---------------------|
| `vfs` | Virtual File System - Coração da abstração | Estável (Core) | `vfs/mod.rs` |
| `initramfs`| Sistema boot-only baseado em formato TAR | Estável | `initramfs/mod.rs` |
| `fat` | Suporte a FAT16/32 (Discos físicos) | Funcional (Read-only) | `fat/mod.rs` |
| `rfs` | Redstone File System (Nativo, Avançado) | Em Design/SPA | `rfs/mod.rs` |
| `devices` | Device nodes e abstrações de HW | Planejado | `devices/mod.rs` |

---

## 🗺️ Mapa Detalhado de Arquivos

### 🚀 Virtual File System (`vfs/`)
- **`mod.rs`**: Ponto de entrada, inicialização e roteamento global de caminhos.
- **`inode.rs`**: Definição de Inodes, tipos de arquivos e o trait `InodeOps`.
- **`file.rs`**: Gerenciamento de arquivos abertos e o trait `FileOps`.
- **`path.rs`**: Parser e normalizador de caminhos (canonicalização).
- **`dentry.rs`**: Estruturas para cache de entradas de diretório (Nome -> Inode).
- **`mount.rs`**: Gerenciamento de pontos de montagem e sistemas de arquivos registrados.

### 💾 Filesystem Backends
#### FAT (`fat/`)
- **`mod.rs`**: Lógica de montagem, detecção de MBR e leitura de clusters.
- **`bpb.rs`**: Parser do BIOS Parameter Block (Boot Sector).
- **`dir.rs`**: Navegação em diretórios FAT e suporte a nomes curtos/longos.
- **`file.rs`**: Implementação de leitura sequencial e aleatória de arquivos FAT.

#### InitRAMFS (`initramfs/`)
- **`mod.rs`**: Driver para formato TAR, extração de arquivos estáticos da memória de boot.

#### RFS (`rfs/`)
- **`spa.rs`**: Storage Pool Allocator (Gerenciamento de discos e pools).
- **`dmu.rs`**: Data Management Unit (Transações e Objetos).
- **`zpl.rs`**: Redstone Posix Layer (Interface VFS).
- **`arc.rs`**: Adaptive Replacement Cache (Cache de dados em memória).

---

## 🚀 Virtual File System (VFS)

O VFS unifica múltiplos dispositivos e formatos de arquivo em uma árvore única começando em `/`.

### Estruturas Core:
1. **`Inode`**: Representa um objeto no disco (arquivo ou diretório). Contém metadados (UID, GID, tamanho, tipo).
2. **`File Handle`**: Representa um arquivo aberto por um processo. Mantém o cursor (`offset`) e flags de acesso.
3. **`Dentry`**: Representa uma entrada de diretório (Nome -> Inode). Usado para cache de caminhos.

### Path Resolution e Roteamento:
O VFS roteia requisições baseado no caminho e na tabela de montagem:
- Arquivos em `/system/core/` são prioritariamente buscados no **InitRAMFS**.
- Demais caminhos como `/system/services/` ou `/apps/` são roteados para o **FAT** (disco principal).

---

## 💾 Filesystem Backends

### 1. InitRAMFS (Boot FS)
Carregado pelo Bootloader como um módulo na RAM. 
- **Formato**: TAR (Tape Archive).
- **Propósito**: Contém o `supervisor` e serviços críticos necessários antes do carregamento dos drivers de disco.
- **Vantagem**: Simplicidade extrema e zero dependência de HW de disco.

### 2. FAT Driver (Disk Migration)
Permite ao RedstoneOS carregar arquivos de diretórios do Host (via QEMU `fat:rw:`) ou discos físicos formatados.
- **Suporte**: FAT16 e FAT32.
- **Destaque**: Parser de MBR integrado para localizar partições ativas.
- **Modo**: Atualmente Read-Only para segurança do kernel.

### 3. Redstone File System (RFS) - *Projeto Futuro*
O RFS é o sistema de arquivos nativo planejado para ser o "state-of-the-art" do SO, trazendo características de nível enterprise para o desktop.

#### Camadas do RFS:
1.  **SPA (Storage Pool Allocator)**:
    - Gerencia `vdevs` (Virtual Devices).
    - Abstrai múltiplos discos físicos em um pool lógico de armazenamento.
    - Implementa RAID-Z e espelhamento (planejado).
2.  **DMU (Data Management Unit)**:
    - Gerencia objetos e transações.
    - Garante que o sistema nunca esteja em estado inconsistente via **Copy-on-Write (COW)**.
    - Permite a criação de snapshots instantâneos e clones.
3.  **ZPL (Redstone Posix Layer)**:
    - Traduz os objetos da DMU em primitivas POSIX (arquivos, diretórios, links simbólicos).
    - É a camada que se comunica diretamente com o VFS.

#### Princípios de Design:
- **Zero-Downtime**: Atualizações do kernel via snapshots (`/system`).
- **Data Integrity**: Cada bloco de metadados e dados terá um checksum SHA-256 (ou similar).
- **Elasticity**: Adição de discos ao pool sem necessidade de reformatação.

---

## 🏗️ Fluxo de E/S (Exemplo: `read()`)

Quando uma aplicação chama `read()`, o dado percorre o seguinte caminho:

1.  **Syscall**: O contexto muda de User para Kernel.
2.  **VFS (`vfs/file.rs`)**: O Kernel localiza o `File Handle` do processo.
3.  **Inode Table (`vfs/inode.rs`)**: O VFS verifica as permissões e chama o método `read` do Inode associado.
4.  **Backend (`fat/mod.rs` ou `initramfs/mod.rs`)**:
    - Se for FAT: Calcula o cluster → Calcula o LBA no disco → Chama o Driver ATA.
    - Se for InitRAMFS: Localiza o offset no buffer TAR na memória.
5.  **Block Layer (`drivers/block/mod.rs`)**: O driver de hardware executa a transferência física.
6.  **Retorno**: O dado é copiado para o buffer do usuário e a syscall retorna.

---

## 📂 Hierarquia de Diretórios Planejada

O RedstoneOS segue uma hierarquia rigorosa para garantir separação de preocupações e segurança:

| Path | Descrição | Regra de Negócio |
|------|-----------|------------------|
| `/system` | Firmware e OS Core | Read-Only. Atualizado apenas via snapshots. |
| `/apps` | Software do Usuário | Partição FAT ou RFS persistente. |
| `/users` | `home` dos usuários | Isolamento de dados e configurações. |
| `/devices` | Abstração de Hardware | Arquivos virtuais (DevFS). |
| `/volumes` | Pontos de montagem | Onde partições secundárias são expostas. |
| `/runtime` | Dados voláteis | `tmpfs`. Limpo a cada reboot. |
| `/state`   | Estado persistente | Configurações globais pequenas. |
| `/snapshots`| Histórico do SO | Links para estados anteriores do `/system`. |
| `/boot`    | Bootloader & Kernel | Arquivos necessários para o próximo boot. |

---

## 🛠️ Regras de Negócio e Segurança

1. **Imutabilidade do Core**: O diretório `/system` deve ser considerado imutável pela runtime do kernel. Qualquer alteração deve ser transacional.
2. **Persistence-Later**: O sistema prioriza subir rapidamente com InitRAMFS e atrasar a montagem de volumes complexos até que os drivers PCI/ATA estejam estáveis.
3. **Abstração de Bloco**: Nenhum driver de FS comunica-se diretamente com portas I/O. Eles usam o Trait `BlockDevice`, permitindo que o SO mude de ATA para VirtIO ou NVMe sem alterar o driver FAT.

---

## 🔮 Roadmap para o Módulo FS

- [ ] **Block Cache**: Implementar cache de 4KB para setores de disco no kernel.
- [ ] **Writable FAT**: Adicionar operações de `write()` e `create()` no driver FAT.
- [ ] **DevFS**: Implementar `/devices/fb0` e `/devices/ttyS0` via sistema de arquivos virtual.
- [ ] **RFS Alpha**: Finalizar o SPA (Storage Pool Allocator) para gerenciamento básico de blocos COW.
- [ ] **Mount Points**: Implementar a função `mount()` real para permitir múltiplas partições.

---
*Documentação gerada pelo Forge Kernel Architecture Team.*
