# Guia de Contribuição

Obrigado pelo interesse em contribuir com o **Forge Kernel**! Este documento guia você pelo processo de desenvolvimento e padrões de código.

## 📋 Índice

- [Padrões de Código](#padrões-de-código)
- [Fluxo de Git](#fluxo-de-git)
- [Dicas de Debug](#dicas-de-debug)

---

## Padrões de Código

Seguimos as convenções oficiais do Rust (Rustfmt e Clippy), com algumas regras adicionais para Kernel Space:

### 1. `unsafe`
-   Todo bloco `unsafe` DEVE ter um comentário `// SAFETY:` explicando por que é seguro.
-   Minimize o escopo de blocos `unsafe`.

```rust
// SAFETY: Garantimos que o ponteiro é válido e alinhado na inicialização.
unsafe {
    *ptr = 0xDEADBEEF;
}
```

### 2. Alocação
-   Evite alocações no caminho crítico de interrupções.
-   Prefira estruturas na stack quando possível.
-   Use `Option` e `Result` extensivamente; nunca `panic!` em produção (exceto falhas catastróficas durante o boot).

### 3. Documentação
-   Documente todas as funções públicas com `///`.
-   Para módulos complexos, inclua um módulo-level doc `//!`.

---

## Fluxo de Git

1.  **Fork** o repositório.
2.  Crie uma **Branch** para sua feature (`feat/scheduler-rr` ou `fix/heap-corruption`).
3.  **Commit** com mensagens claras (Conventional Commits):
    -   `feat: add round robin scheduler`
    -   `fix: resolve page fault in vmm`
    -   `docs: update build instructions`
4.  Abra um **Pull Request**.

---

## Dicas de Debug

### Serial Output
O método mais confiável é usar logs na porta serial.
```rust
println!("DEBUG: Valor de cr3 = {:#x}", cr3);
```
Certifique-se de iniciar o QEMU com `-serial stdio` para ver a saída no terminal.

### QEMU Monitor
Pressione `Ctrl + Alt + 2` (ou use o socket) para acessar o monitor do QEMU.
-   `info registers`: Ver estado da CPU.
-   `info mem`: Ver mapeamentos de memória.
-   `x /10i $rip`: Disassemble da instrução atual.

### GDB
Rode o QEMU com `-s -S` para esperar conexão do GDB na porta 1234.
```bash
rust-gdb target/x86_64-redstone/release/forge
(gdb) target remote :1234
```
