# Guia de Migração: Sistema de Logs Zero-Overhead

## Contexto

O sistema de logging do Redstone OS Kernel foi completamente refatorado para eliminar problemas de `#UD` (Invalid Opcode) causados por instruções SSE/AVX geradas pelo `core::fmt`.

**Data**: 2025-12-26  
**Versão**: forge 0.0.3

## Arquivos Refatorados

| Arquivo | Status | Descrição |
|---------|--------|-----------|
| `Cargo.toml` | ✅ FEITO | Novas features de log |
| `src/drivers/serial.rs` | ✅ FEITO | Escrita direta via assembly |
| `src/core/logging.rs` | ✅ FEITO | Macros zero-overhead |

## O que Mudou

### 1. Features do Cargo.toml

```toml
# NOVA configuração
default = ["log_trace"]   # Modo padrão

# Features disponíveis:
no_logs = []     # Release: remove 100% dos logs
log_info = []    # Dev: ERROR, WARN, INFO, DEBUG
log_trace = []   # Debug: todos os níveis
```

### 2. Nova Sintaxe dos Macros

**ATENÇÃO**: A nova sintaxe **NÃO suporta format_args!** ou placeholders `{}`.

#### Sintaxe Antiga (REMOVER)
```rust
kinfo!("Valor: {:#x}", some_value);
kdebug!("(PMM) Frame {} alocado", idx);
ktrace!("(VMM) Mapeando virt={:#x} -> phys={:#x}", virt, phys);
```

#### Sintaxe Nova (USAR)
```rust
// Apenas string literal
kinfo!("(PMM) Inicializando...");

// String + valor hex
kinfo!("(PMM) Addr=", some_value);
kdebug!("(PMM) Frame alocado em ", addr);

// Múltiplos valores (use serial:: diretamente)
serial::emit_str("[DEBG] Start=");
serial::emit_hex(start);
serial::emit_str(" End=");
serial::emit_hex(end);
serial::emit_nl();

// OU use klog! para construção manual
klog!("Start=", start, " End=", end);
knl!();
```

### 3. Macros Disponíveis

| Macro | Nível | Quando Ativo |
|-------|-------|--------------|
| `kerror!` | ERROR | Sempre (exceto no_logs) |
| `kwarn!` | WARN | Sempre (exceto no_logs) |
| `kinfo!` | INFO | Sempre (exceto no_logs) |
| `kdebug!` | DEBUG | log_info ou log_trace |
| `ktrace!` | TRACE | Apenas log_trace |
| `klog!` | - | Sempre (exceto no_logs) |
| `knl!` | - | Sempre (exceto no_logs) |
| `kok!` | OK | Sempre (exceto no_logs) |
| `kfail!` | FAIL | Sempre (exceto no_logs) |
| `kprint!` | - | Sempre (exceto no_logs) |
| `kprintln!` | - | Sempre (exceto no_logs) |

### 4. Console de Vídeo

Os logs **NÃO são mais enviados para o console de vídeo**.
Apenas a serial recebe os logs agora.

A função `console_print_fmt()` ainda existe mas não é mais chamada pelo sistema de logs.

## Tarefa de Migração

### Arquivos que Precisam Migração

Todos os arquivos abaixo usam macros de log com sintaxe antiga:

**Módulos Críticos (Prioridade Alta)**
- [ ] `src/mm/pmm/bitmap.rs`
- [ ] `src/mm/pmm/pt_scanner.rs`
- [ ] `src/mm/pmm/zones.rs`
- [ ] `src/mm/vmm/vmm.rs`
- [ ] `src/mm/heap/mod.rs`
- [ ] `src/mm/mod.rs`
- [ ] `src/core/entry.rs`
- [ ] `src/arch/x86_64/gdt.rs`
- [ ] `src/arch/x86_64/idt.rs`

**Drivers**
- [ ] `src/drivers/serial.rs` (logs de init)
- [ ] `src/drivers/timer.rs`
- [ ] `src/drivers/pic.rs`
- [ ] `src/drivers/console.rs`

**Scheduler**
- [ ] `src/sched/scheduler.rs`
- [ ] `src/sched/task.rs`

**Filesystem**
- [ ] `src/fs/mod.rs`
- [ ] `src/fs/initramfs.rs`

**IPC**
- [ ] `src/ipc/mod.rs`
- [ ] `src/ipc/port.rs`

**Syscall**
- [ ] `src/syscall/dispatch.rs`
- [ ] `src/syscall/*.rs` (vários)

**Core**
- [ ] `src/core/panic.rs`
- [ ] `src/core/elf.rs`
- [ ] `src/core/handle.rs`

**Arch**
- [ ] `src/arch/x86_64/cpu.rs`
- [ ] `src/arch/x86_64/interrupts.rs`

**Testes**
- [ ] `src/*/test.rs` (todos os módulos de teste)

## Regras de Migração

### Regra 1: Strings Simples
```rust
// ANTES
kinfo!("(GDT) Inicializado");

// DEPOIS (sem mudança!)
kinfo!("(GDT) Inicializado");
```

### Regra 2: String + Um Valor Hex
```rust
// ANTES
kinfo!("(PMM) Addr: {:#x}", addr);
kdebug!("(VMM) CR3={:#x}", cr3);

// DEPOIS
kinfo!("(PMM) Addr=", addr);
kdebug!("(VMM) CR3=", cr3);
```

### Regra 3: Múltiplos Valores
```rust
// ANTES
ktrace!("(VMM) Mapping virt={:#x} -> phys={:#x}", virt, phys);

// DEPOIS (opção 1 - serial direto)
serial::emit_str("[TRAC] (VMM) Mapping virt=");
serial::emit_hex(virt);
serial::emit_str(" -> phys=");
serial::emit_hex(phys);
serial::emit_nl();

// DEPOIS (opção 2 - klog!)
klog!("[TRAC] (VMM) Mapping virt=", virt, " -> phys=");
serial::emit_hex(phys);
knl!();
```

### Regra 4: Valores Decimais
```rust
// ANTES
kinfo!("(PMM) Total frames: {}", count);

// DEPOIS
serial::emit_str("[INFO] (PMM) Total frames: ");
serial::emit_dec(count);
serial::emit_nl();
```

### Regra 5: Remover format_args! Completamente
```rust
// ANTES (código que NÃO compila mais)
KernelLogger::log(LogLevel::Info, format_args!("Hello {}", x));

// DEPOIS
kinfo!("Hello");  // Sem o valor, ou:
serial::emit_str("[INFO] Hello ");
serial::emit_hex(x as u64);
serial::emit_nl();
```

## Verificação Pós-Migração

1. Compilar com `cargo build` (deve passar sem erros)
2. Compilar com `cargo build --features no_logs --no-default-features`
3. Rodar no QEMU e verificar logs na serial
4. Verificar que não há `#UD` durante o boot

## Funções Serial Disponíveis

```rust
use crate::drivers::serial;

serial::emit(b'X');           // Byte único
serial::emit_str("Hello");    // String
serial::emit_hex(0x1234);     // u64 em hex (0x0000000000001234)
serial::emit_hex32(0x1234);   // u32 em hex (0x00001234)
serial::emit_dec(42);         // usize em decimal
serial::emit_nl();            // Newline (\r\n)
```

## Código Morto a Remover

Após migração completa, este código pode ser removido:

1. `KernelLogger` struct e métodos (em logging.rs - já removido)
2. `SERIAL1: Mutex<SerialPort>` (em serial.rs - já removido)
3. Imports de `core::fmt` não utilizados
4. Chamadas a `console_print_fmt` em logging

## Formato de Output Esperado

```
[INFO] (GDT) Inicializado
[DEBG] (IDT) init: Tabela configurada
[TRAC] (PMM) Frame alocado addr=0x000000001E050000
[WARN] (VMM) Huge page collision em PD[234]
[ERRO] (Heap) OOM: Sem frames disponíveis
```
