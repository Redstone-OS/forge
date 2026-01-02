# Documentação do Sistema de Arquivos (`src/fs`)

> **Caminho**: `src/fs`  
> **Responsabilidade**: Abstração de armazenamento persistente e interface unificada de E/S.  
> **Arquitetura**: Virtual File System (VFS).

---

## 🏛️ Arquitetura VFS

O RedstoneOS implementa um **Virtual File System (VFS)** clássico, inspirado no Unix. O Kernel não interage diretamente com discos ou partições, mas sim com objetos abstratos (`Inode`, `File`, `Dentry`).

```mermaid
graph TD
    UserApp[Aplicação] -->|open/read| Syscall
    Syscall -->|fd -> File| VFS
    VFS -->|Resolve Path| Dentry[Dentry Cache]
    Dentry -->|Lookup| Inode[Inode (Metadados)]
    Inode -->|Read ops| Backend
    
    Backend -->|Driver| Initramfs
    Backend -->|Driver| Ext2/FAT
    Backend -->|Driver| DevFS
    Backend -->|Driver| ProcFS
```

---

## 🧩 Componentes Principais (`vfs/`)

### `Inode` (Index Node)
Representa um objeto único no sistema de arquivos (arquivo ou diretório). Contém metadados:
*   Tamanho
*   Permissões (0777)
*   Timestamps (Access, Modify, Create)
*   Ponteiros de dados (ex: blocos no disco).

### `Dentry` (Directory Entry)
Representa o nome de um arquivo em um diretório e faz a ponte "Nome -> Inode".
*   O VFS mantém um cache (`dcache`) para agilizar lookups de paths frequentes.

### `File`
Representa um arquivo **aberto** por um processo.
*   Contém a posição atual do cursor (`offset`).
*   Pode haver múltiplos objetos `File` apontando para o mesmo `Inode` (se dois processos abrirem o mesmo arquivo).

---

## 📂 Sistemas de Arquivos Implementados

O `src/fs` contém implementações de FS específicos:

### 1. `initramfs`
Sistema de arquivos somente-leitura carregado na memória durante o boot.
*   Contém executáveis essenciais (`init`, `shell`) e drivers críticos.
*   Estrutura simples (CPIO ou similar).

### 2. `devfs` (`/dev`)
Sistema de arquivos sintético que expõe dispositivos como arquivos.
*   `/dev/null`: Buraco negro.
*   `/dev/serial`: Porta serial.
*   `/dev/fb0`: Framebuffer de vídeo.

### 3. `tmpfs`
Sistema de arquivos volátil que reside na RAM (Heap/Páginas).
*   Rápido.
*   Dados perdidos no reboot.
*   Usado para `/tmp` e arquivos temporários de IPC.

### 4. `procfs` (`/proc`)
Interface de texto para estruturas internas do kernel.
*   `/proc/1/status`: Informações do processo PID 1.
*   `/proc/meminfo`: Uso de memória global.
*   Não armazena dados, gera o conteúdo dinamicamente na leitura (`read`).

---

## 🛠️ Interface VFS (`FileOps` Trait)

Qualquer novo sistema de arquivos deve implementar os traits:

```rust
pub trait FileOps {
    fn read(&mut self, buf: &mut [u8]) -> usize;
    fn write(&mut self, buf: &[u8]) -> usize;
    fn seek(&mut self, offset: i64, whence: SeekWhence) -> u64;
    fn close(&mut self);
}

pub trait InodeOps {
    fn lookup(&self, name: &str) -> Option<Arc<Inode>>;
    fn create(&self, name: &str, type: FileType) -> Result<Arc<Inode>>;
    // ...
}
```

Isso permite polimorfismo: o kernel pode chamar `.read()` sem saber se está lendo de um SSD NVMe ou de um arquivo gerado na RAM.
