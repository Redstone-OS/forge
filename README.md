# Kernel Forge

**Versão**: 0.0.1  
**Linguagem**: Rust 100%  
**Arquitetura**: x86_64 (aarch64 e riscv64 planejados)  
**Modelo**: Microkernel Híbrido  
**Status**: Arrumando inicialização

---

## 📋 Visão Geral

O Forge é o kernel do Redstone OS, completamente reorganizado seguindo padrões profissionais da indústria (estilo Linux). Esta reorganização torna o código mais limpo, escalável e fácil de manter.

## 📁 Estrutura

```
src/
├── core/          # Núcleo (scheduler CFS, processos, threads, init)
├── mm/            # Memória (VMM, PMM Buddy, SLUB, page cache)
├── fs/            # Filesystem (VFS completo + DevFS/ProcFS/SysFS/TmpFS/FAT32)
├── drivers/       # Drivers por barramento (PCI, USB, legacy)
├── net/           # Rede (TCP/IP stack - TODO v2.0)
├── ipc/           # IPC (pipes, shm, futex, unix sockets)
├── security/      # Segurança (DAC + Capabilities + Audit)
├── hal/           # HAL (x86_64 funcional, aarch64/riscv64 TODO)
├── syscall/       # Syscalls por subsistema (0-99, 100-199, etc)
└── lib/           # Bibliotecas (sync, collections, util)
```

## 🚀 Funcionalidades

### ✅ Implementado (v0.0.1)
- Gerenciamento de memória (RMM)
- Paginação de 4 níveis (x86_64)
- Sistema de syscalls
- Gerenciamento de processos
- Sistema de schemes
- Suporte ACPI
- Multitarefa preemptiva

### 🔄 Reorganizado (v0.0.1)
- **Core**: Scheduler CFS (140 níveis), processos pesados + threads leves
- **MM**: VMM/PMM separados, Buddy + SLUB Allocators
- **FS**: VFS completo estilo Linux
- **HAL**: Abstração de hardware
- **IPC**: Pipes, shared memory, futex, unix sockets
- **Security**: DAC + Capabilities + Audit
- **Syscalls**: Organizados por subsistema com numeração

### 📋 Planejado (v1.0+)
- Implementar CFS scheduler completo
- Implementar Buddy Allocator
- Implementar SLUB Allocator
- Implementar Copy-on-Write
- Completar VFS
- Implementar drivers essenciais
- Stack TCP/IP (v2.0)

## 📝 Documentação

Toda a documentação está em **Português (PT-BR)** com:
- Rustdoc completo em cada função
- TODOs estruturados: `TODO(prioridade=alta, versão=v1.0): Descrição`
- Comentários explicativos
- Exemplos de uso

### TODOs Estruturados

```rust
// TODO(prioridade=alta, versão=v1.0): Implementar Buddy Allocator
// TODO(prioridade=média, versão=v2.0): Adicionar huge pages
// TODO(prioridade=baixa, versão=v3.0): Otimizar para NUMA
```

## 🔧 Compilação

```bash
# Compilar kernel
cargo build --target x86_64-unknown-none --release

# Output: target/x86_64-unknown-none/release/forge
```

**Nota**: A estrutura reorganizada está em `src_new/`. Para usar, renomeie:
```bash
mv src src_old
mv src_new src
```

## 🎓 Padrões Seguidos

### Organização
- **Estilo Linux**: Hierarquia clara por subsistema
- **Modularidade**: Cada módulo tem responsabilidade única
- **Escalabilidade**: Fácil adicionar novos subsistemas

### Código
- **snake_case**: Arquivos e funções
- **PascalCase**: Structs e traits
- **SCREAMING_SNAKE_CASE**: Constantes
- **Máximo 1000 linhas** por arquivo

### Inicialização
- **10 fases nomeadas**: CPU → Memory → Scheduler → Process → IPC → VFS → Drivers → Security → Network → Userspace

## 📊 Estatísticas

- **Arquivos criados**: 40+
- **Linhas de documentação**: ~800
- **TODOs adicionados**: 80+
- **Módulos principais**: 10
- **Submódulos**: 30+

## 🗺️ Roadmap

### v1.0 (6 meses)
- ✅ Reorganização completa
- ⏳ Implementar CFS scheduler
- ⏳ Implementar Buddy + SLUB
- ⏳ Implementar VFS completo
- ⏳ Implementar drivers essenciais
- ⏳ Documentação 100%

### v2.0 (12 meses)
- Stack TCP/IP completo
- Drivers userspace
- Copy-on-Write
- Huge pages
- Namespaces/Containers

### v3.0 (18 meses)
- SELinux-like MAC
- Criptografia de disco
- Real-time scheduling
- NUMA support

## 🤝 Contribuindo

1. Leia a documentação em cada módulo
2. Siga os padrões de código
3. Adicione TODOs estruturados
4. Documente em PT-BR
5. Teste suas mudanças

## 📄 Licença

MIT License - Veja `LICENSE` para detalhes

---

**Última atualização**: 16 de dezembro de 2025  
**Status**: v0.3.0 - Reorganização completa ✅  
**Próxima versão**: v1.0 - Implementação das funcionalidades
