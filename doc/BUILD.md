# Compilação e Execução

## 📋 Índice

- [Pré-requisitos](#pré-requisitos)
- [Compilando o Kernel](#compilando-o-kernel)
- [Rodando em QEMU](#rodando-em-qemu)
- [Estrutura do Build](#estrutura-do-build)

---

## Pré-requisitos

Para compilar o Forge, você precisa das seguintes ferramentas:

1.  **Rust Nightly**: O kernel usa features instáveis.
    ```bash
    rustup override set nightly
    rustup component add rust-src llvm-tools-preview
    ```
2.  **QEMU**: Emulador para testes (`qemu-system-x86_64`).
3.  **LLVM**: Ferramentas como `llvm-objdump` e `llvm-readobj` (opcional, para debug).

---

## Compilando o Kernel

O projeto utiliza `cargo` com configurações específicas em `.cargo/config.toml` para cross-compilation.

### Comando Básico
```bash
cd forge
cargo build --release
```

Isso gerará o binário ELF em `target/x86_64-redstone/release/forge`.

### Target Customizado
O arquivo `x86_64-redstone.json` define o target spec:
-   Arch: `x86_64`
-   OS: `none` (Bare Metal)
-   Features: `-mmx,-sse,+soft-float` (Kernel mode não usa FPU por padrão, exceto com cuidado)

---

## Rodando em QEMU

Para rodar o kernel, você precisa de um bootloader compatível (Ignite). Recomendamos usar o sistema de build do Redstone OS (Anvil/Xtask) na raiz do repositório, que automatiza a criação da imagem de disco.

### Via Anvil (Recomendado)
Na raiz do repositório `Redstone OS`:
```bash
cargo run --package xtask -- run
```

### Manualmente
1.  Compile o `ignite` (bootloader).
2.  Compile o `forge` (kernel).
3.  Crie uma estrutura de diretórios UEFI (ESP).
    ```
    efi/boot/bootx64.efi  -> ignite.efi
    efi/redstone/forge    -> forge (elf)
    ignite.conf           -> Configuração
    ```
4.  Rode o QEMU com a pasta como drive virtual.

---

## Estrutura do Build

O build segue o padrão Rust, mas com ajustes para "no_std":

-   **`build.rs`**: Scripts de build (se necessário).
-   **`linker.ld`**: Script do linker que define o layout de memória (VMA/LMA).
    -   `text`: Código executável.
    -   `rodata`: Dados somente leitura.
    -   `data/bss`: Variáveis globais.

> **Nota**: O Entry Point é definido como `_start` no linker script.
