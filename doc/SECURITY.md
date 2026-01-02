# Documentação do Módulo Security (`src/security`)

> **Caminho**: `src/security`  
> **Responsabilidade**: Garantir isolamento, controle de acesso e auditoria no kernel.  
> **Modelo**: Object-Capability (OCAP).  
> **Status**: Funcional (`Capability`), Placeholder (`Audit`, `Credentials`, `Sandbox`).

---

## 🏛️ Filosofia de Segurança (OCAP)

O RedstoneOS abandona o modelo tradicional de segurança baseado em identidade e ACLs (como usuarios `root` x `user` no Unix) em favor de um modelo baseado em **Capabilities**.

### Princípios Fundamentais
1.  **Sem Superusuário**: Não existe um "root" que pode tudo. O poder vem da posse de tokens.
2.  **Posse é Poder (Token)**: Se você tem um Handle para um objeto, você tem acesso a ele.
3.  **Delegação Explícita**: Um processo só pode passar acesso a outro se tiver direitos de `TRANSFER`.
4.  **Granularidade Fina**: Handles carregam direitos específicos (`READ`, `WRITE`, `EXEC`).

---

## 📂 Estrutura de Arquivos

| Diretório | Descrição Técnica | Status |
|:----------|:------------------|:-------|
| `capability/` | Coração do sistema OCAP. Define `CSpace`, `Rights` e `CapHandle`. | ✅ Ativo |
| `sandbox/` | Mecanismos de isolamento estilo Namespaces. | 🚧 WIP |
| `credentials/` | Identidade de processo (compatibilidade legada/auditoria). | 🚧 WIP |
| `audit/` | Logging de eventos de segurança. | 🚧 WIP |

---

## 🔑 Sistema de Capabilities (`capability/`)

### 1. `CSpace` (Capability Space)
Cada processo possui seu próprio "Espaço de Capabilities", que é uma tabela indexada (como uma File Descriptor Table superpoderosa).
*   **Isolamento**: O Processo A não consegue ver ou tocar nos handles do Processo B.
*   **Lookup**: A syscall recebe um `CapHandle(u32)`, e o kernel traduz isso para `&Capability` usando o CSpace do processo atual.

### 2. `Capability` (O Token)
O token real armazenado no kernel. Estrutura opaca para o usuário.
```rust
pub struct Capability {
    pub cap_type: CapType,    // Ex: Port, VMO, Thread
    pub rights: CapRights,    // Ex: READ | WRITE
    pub object_ref: u64,      // Ponteiro interno para o objeto real
    pub badge: u64,           // Marca d'água para IPC (identifica quem chama)
}
```

### 3. `CapRights` (Máscara de Direitos)
Bitmask definindo o que pode ser feito com o handle.
*   `READ` / `WRITE`: Acesso a dados.
*   `GRANT`: Permite criar um capability *filho* com menos poderes (derivado).
*   `TRANSFER`: Permite enviar este handle para outro processo via IPC.

---

## 🛡️ Sandbox e Namespaces (`sandbox/`)

Planejado para funcionar como os Namespaces do Linux ou Jails do FreeBSD.
*   **Meta**: Permitir que um processo rode achando que é o único no sistema (PID 1), com seu próprio FS root e interfaces de rede.
*   Atualmente contém apenas esqueletos (`Sandbox`, `Namespace`).

---

## 🏗️ Guia de Uso (Kernel Dev)

### Validando Acesso
Ao implementar uma syscall que opera sobre um objeto (ex: `sys_send_msg`), o procedimento padrão é:

1.  Receber o `handle_id` do usuário.
2.  Obter o `CSpace` do processo atual.
3.  Chamar `cspace.lookup(handle_id)`.
4.  Verificar se o handle existe E se o tipo é correto (`CapType::Port`).
5.  Verificar se os direitos são suficienes (`cap.rights.has(CapRights::WRITE)`).

```rust
// Exemplo Conceitual
fn sys_send(handle: u32) -> Result {
    let proc = current_process();
    let cap = proc.cspace.lookup(handle)?; // Retorna Erro se handle inválido

    if cap.type != CapType::Port { return Err(TypeMismatch); }
    if !cap.rights.has(CapRights::WRITE) { return Err(PermissionDenied); }

    // Acesso permitido
    kernel_send(cap.object_ref);
}
```

---

## 🔮 Futuro

1.  **Revocation**: Implementar sistema para revogar handles "filhos" criados a partir de um handle "pai". Implica em rastrear a árvore de genealogia das caps.
2.  **Audit Hooks**: Inserir pontos de log em todas as falhas de verificação de permissão.
