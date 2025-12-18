# Testes do Filesystem - Guia de Uso

## 📋 Estrutura dos Testes

```
forge/src/fs/tests/
├── mod.rs           # Módulo principal com helpers
├── devfs.rs         # Testes do DevFS
├── procfs.rs        # Testes do ProcFS
├── sysfs.rs         # Testes do SysFS
├── tmpfs.rs         # Testes do TmpFS
├── fat32.rs         # Testes do FAT32
├── vfs.rs           # Testes do VFS
└── integration.rs   # Testes de integração
```

## 🚀 Como Executar

### Todos os Testes de Filesystem

```bash
cargo test --package forge --lib fs::tests
```

### Testes de um Módulo Específico

```bash
# DevFS
cargo test --package forge --lib fs::tests::devfs

# ProcFS
cargo test --package forge --lib fs::tests::procfs

# TmpFS
cargo test --package forge --lib fs::tests::tmpfs

# FAT32
cargo test --package forge --lib fs::tests::fat32

# Integração
cargo test --package forge --lib fs::tests::integration
```

### Teste Específico

```bash
# Executar apenas um teste
cargo test --package forge --lib fs::tests::devfs::test_device_number

# Com output detalhado
cargo test --package forge --lib fs::tests::devfs::test_device_number -- --nocapture
```

### Testes com Verbose

```bash
# Mostrar todos os testes executados
cargo test --package forge --lib fs::tests -- --nocapture

# Mostrar apenas testes que falharam
cargo test --package forge --lib fs::tests
```

## 📝 Convenções de Nomenclatura

- **`test_*`** - Testes unitários básicos
- **`integration_*`** - Testes de integração entre módulos
- **`bench_*`** - Benchmarks (quando disponível)

## ✅ Status Atual dos Testes

| Módulo | Testes Básicos | Testes Completos | Status |
|--------|----------------|------------------|--------|
| DevFS | ✅ 7 testes | ⏳ Pendente | Compilando |
| ProcFS | ✅ 4 testes | ⏳ Pendente | Compilando |
| SysFS | ✅ 3 testes | ⏳ Pendente | Compilando |
| TmpFS | ✅ 5 testes | ⏳ Pendente | Compilando |
| FAT32 | ✅ 5 testes | ⏳ Pendente | Compilando |
| VFS | ⏳ Placeholder | ⏳ Pendente | Aguardando impl |
| Integration | ✅ 1 teste | ⏳ Pendente | Compilando |

**Total:** 25 testes básicos implementados

## 🔧 Helpers Disponíveis

O módulo `tests/mod.rs` fornece helpers para criar instâncias de teste:

```rust
use crate::fs::tests::*;

// Criar filesystems para testes
let devfs = create_test_devfs();
let procfs = create_test_procfs();
let sysfs = create_test_sysfs();
let tmpfs = create_test_tmpfs();  // 1MB
let fat32 = create_test_fat32();
```

## 📚 Exemplos de Uso

### Exemplo 1: Testar DeviceNumber

```rust
#[test]
fn test_my_device() {
    use crate::fs::devfs::DeviceNumber;
    
    let dev = DeviceNumber::new(1, 3);
    assert_eq!(dev.major, 1);
    assert_eq!(dev.minor, 3);
}
```

### Exemplo 2: Testar TmpFS

```rust
#[test]
fn test_tmpfs_space() {
    use crate::fs::tests::create_test_tmpfs;
    
    let tmpfs = create_test_tmpfs();
    assert_eq!(tmpfs.available_space(), 1024 * 1024);
}
```

### Exemplo 3: Teste de Integração

```rust
#[test]
fn test_multiple_fs() {
    use crate::fs::tests::*;
    
    let devfs = create_test_devfs();
    let tmpfs = create_test_tmpfs();
    
    // Ambos devem coexistir
    assert!(true);
}
```

## 🎯 Próximos Passos

### Testes a Adicionar (quando implementar funcionalidades):

**DevFS:**
- [ ] `test_register_device` - Registrar dispositivo
- [ ] `test_unregister_device` - Remover dispositivo
- [ ] `test_lookup_device` - Buscar dispositivo por nome
- [ ] `test_read_from_null` - Ler de /dev/null
- [ ] `test_write_to_null` - Escrever em /dev/null
- [ ] `test_read_from_zero` - Ler de /dev/zero

**ProcFS:**
- [ ] `test_read_cpuinfo` - Ler /proc/cpuinfo
- [ ] `test_read_meminfo` - Ler /proc/meminfo
- [ ] `test_read_process_status` - Ler /proc/[pid]/status
- [ ] `test_list_processes` - Listar processos

**TmpFS:**
- [ ] `test_create_file` - Criar arquivo
- [ ] `test_write_file` - Escrever em arquivo
- [ ] `test_read_file` - Ler arquivo
- [ ] `test_out_of_space` - Testar limite de espaço

**FAT32:**
- [ ] `test_mount_volume` - Montar volume FAT32
- [ ] `test_read_directory` - Ler diretório
- [ ] `test_read_file` - Ler arquivo
- [ ] `test_parse_long_filename` - Parse de nomes longos

**VFS:**
- [ ] `test_mount_filesystem` - Montar filesystem
- [ ] `test_path_lookup` - Resolver caminho
- [ ] `test_file_operations` - Operações de arquivo

## 🐛 Debugging

### Executar com Backtrace

```bash
RUST_BACKTRACE=1 cargo test --package forge --lib fs::tests
```

### Executar Teste Específico com Output

```bash
cargo test --package forge --lib fs::tests::devfs::test_device_number -- --nocapture --test-threads=1
```

### Ignorar Testes Lentos

```bash
# Marcar teste como ignorado
#[test]
#[ignore]
fn slow_test() { }

# Executar apenas testes não-ignorados
cargo test --package forge --lib fs::tests

# Executar APENAS testes ignorados
cargo test --package forge --lib fs::tests -- --ignored
```

## 📊 Cobertura de Código

```bash
# Instalar tarpaulin (apenas uma vez)
cargo install cargo-tarpaulin

# Gerar relatório de cobertura
cargo tarpaulin --package forge --lib --out Html
```

---

**Última atualização:** 2025-12-16
