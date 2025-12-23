# System Calls (Chamadas de Sistema)

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Interface (ABI)](#interface-abi)
- [Lista de Syscalls](#lista-de-syscalls)

---

## Visão Geral

As **System Calls** são a interface fundamental entre aplicações de usuário (Userspace) e o Kernel. Elas permitem que programas solicitem serviços como alocação de memória, I/O de arquivos e criação de processos.

O Forge utiliza a instrução `syscall` (x86_64) para transições rápidas entre Ring 3 e Ring 0.

### Estrutura do Módulo (`src/syscall/`)
-   **`dispatcher.rs`**: Ponto central de despacho. Recebe o ID da syscall e chama a função correspondente.
-   **`numbers.rs`**: Define os IDs numéricos de cada syscall.
-   **`process.rs`**: Syscalls de processo (exit, yield, sleep).
-   **`fs.rs`**: Syscalls de arquivo (read, write, open).
-   **`memory.rs`**: Syscalls de memória (mmap, munmap).

---

## Interface (ABI)

O Forge segue a convenção de chamadas **System V AMD64 ABI** para syscalls, similar ao Linux.

| Registrador | Uso |
|-------------|-----|
| `RAX` | Número da Syscall (Entrada) / Valor de Retorno (Saída) |
| `RDI` | 1º Argumento |
| `RSI` | 2º Argumento |
| `RDX` | 3º Argumento |
| `R10` | 4º Argumento (RCX é usado pela instrução syscall) |
| `R8` | 5º Argumento |
| `R9` | 6º Argumento |

### Exemplo em Assembly (NASM)
```nasm
mov rax, 1      ; Syscall Write
mov rdi, 1      ; File Descriptor (Stdout)
mov rsi, msg    ; Buffer
mov rdx, 12     ; Tamanho
syscall
```

---

## Lista de Syscalls

> **Nota**: A lista abaixo reflete as syscalls definidas em `src/syscall/numbers.rs`.

### Processos
-   `SYS_EXIT` (0): Termina o processo atual.
-   `SYS_YIELD` (1): Cede o restante do tempo de CPU voluntariamente.
-   `SYS_SLEEP` (2): Dorme por N milissegundos.
-   `SYS_GETPID` (3): Retorna o ID do processo.

### Arquivos (File I/O)
-   `SYS_READ` (4): Lê de um descritor de arquivo.
-   `SYS_WRITE` (5): Escreve em um descritor de arquivo.
-   `SYS_OPEN` (6): Abre um arquivo.
-   `SYS_CLOSE` (7): Fecha um arquivo.

### Memória
-   `SYS_MMAP` (8): Mapeia memória ou arquivos.
-   `SYS_MUNMAP` (9): Desmapeia memória.
