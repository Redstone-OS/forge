# Documentação do Sistema IPC (`src/ipc`)

> **Caminho**: `src/ipc`  
> **Responsabilidade**: Inter-Process Communication. Permitir troca de dados e sinais entre processos isolados.  
> **Performance**: Foco em Zero-Copy e processamento assíncrono.

---

## 🏛️ Visão Geral

O IPC é o sistema nervoso do microkernel (ou kernel híbrido moderno). No RedstoneOS, o IPC é projetado para ser **Baseado em Entidades** e **Orientado a Capabilities**.

---

## 📦 Mecanismos de Comunicação

O kernel oferece quatro primitivas principais, cada uma para um caso de uso:

| Mecanismo | Topologia | Cópia de Dados? | Bloqueante? | Caso de Uso |
|:----------|:----------|:----------------|:------------|:------------|
| **Port** | 1:N (Servidor) | Sim (Pequena) | Sim | RPC, Serviços do Sistema, Syscalls complexas. |
| **Channel**| 1:1 (Socket) | Sim | Sim | Conversa direta entre dois processos (ex: Pipe). |
| **SHM** | N:N (Memória) | **Não (Zero)** | Não | Transferência de buffers grandes (Vídeo, Áudio). |
| **Futex** | N:N (Sinal) | Não | **Sim** | Sincronização de threads e coordenação de SHM. |

---

## 📂 Estrutura de Arquivos

| Módulo | Descrição Técnica |
|:-------|:------------------|
| `port/` | Implementação de Portas de Mensagem. Filas de mensagens com prioridade. |
| `channel/` | Canais bidirecionais (semelhante a Unix Sockets). |
| `shm/` | Shared Memory Manager. Mapeia as mesmas páginas físicas em múltiplos Address Spaces. |
| `futex/` | Fast Userspace Mutex. Permite dormir no kernel e acordar via sinal de outro processo. |
| `message/` | Definição do "Envelope" de mensagem. Suporta envio de dados + handles (Handle Passing). |

---

## 🔧 Detalhamento Técnico

### 1. Ports (O Modelo Cliente-Servidor)
Uma `Port` é uma caixa postal.
*   Um **Servidor** cria a porta e mantém o direito de `RECEIVE`.
*   Múltiplos **Clientes** recebem o direito de `SEND`.
*   Quando um cliente envia, a mensagem entra numa fila. O servidor consome em ordem (FIFO).

### 2. Handle Passing (A "Mágica")
Uma mensagem IPC não carrega apenas bytes (`u8`). Ela pode carregar **Capabilities**.
Isso permite que um processo passe o acesso de um arquivo aberto ou de uma região de memória para outro, simplesmente "enviando" o handle pela porta.
*   O Kernel intercepta a mensagem.
*   Remove o handle da tabela do remetente.
*   Insere na tabela do destinatário.
*   Entrega o novo ID para o destinatário.

### 3. Shared Memory (SHM)
Para alta performance (ex: Compositor Gráfico recebendo frames de Apps), copiar dados é inviável.
*   `sys_shm_create`: Aloca páginas físicas.
*   `sys_shm_map`: Mapeia essas páginas no processo A e no processo B.
*   Ambos leem/escrevem instantaneamente. `Futex` é usado para avisar "terminei de escrever".

---

## ⚠️ Segurança

O IPC é estritamente controlado pelo módulo `security` (Capabilities).
*   Você não pode enviar para uma porta que não possui handle.
*   Você não pode mapear memória compartilhada que não lhe foi concedida.
*   Flooding: Portas têm capacidade máxima (`capacity`). Se cheia, o remetente bloqueia ou recebe erro.
