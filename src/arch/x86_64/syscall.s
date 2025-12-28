# Arquivo: syscall.s
# 
# Propósito: Trampolim em assembly para chamadas de sistema (System Calls).
# Responsável por salvar o contexto do usuário (TrapFrame), trocar para a stack do kernel,
# invocar o dispatcher Rust (`syscall_dispatcher`) e restaurar o contexto para retorno.
#
# Este arquivo deve ser incluído via `global_asm!` no módulo Rust correspondente.
#
# Detalhes de Implementação:
# - Usa a instrução `swapgs` para acessar dados per-cpu (Kernel Stack).
# - Constrói um `TrapFrame` compatível com a struct Rust `repr(C)`.
# - Garante alinhamento de stack para a ABI SysV x86_64.
# - Usa `sysretq` para retorno rápido ao ring 3.

.section .text
.global syscall_entry
.extern syscall_dispatcher

.code64

# --------------------------------------------------------------------------------
# syscall_entry
#
# Ponto de entrada para chamadas de sistema via instrução SYSCALL.
#
# Estado da CPU na Entrada:
# - RIP: RIP do usuário salvo em RCX
# - RFLAGS: RFLAGS do usuário salvo em R11
# - CS: Carregado do MSR_STAR (CS do Kernel)
# - SS: Carregado do MSR_STAR (SS do Kernel)
# - RSP: Stack Pointer do Usuário (NÃO ALTERADO pelo hardware)
# - CPL: 0 (Ring 0)
#
# Responsabilidade:
# 1. Trocar diretamente para Stack do Kernel (usando SWAPGS e dados Per-CPU)
# 2. Salvar Contexto do Usuário (TrapFrame)
# 3. Chamar handler Rust
# 4. Restaurar Contexto do Usuário
# 5. Retornar para Modo de Usuário (SYSRETQ)
# --------------------------------------------------------------------------------
syscall_entry:
    # 1. Trocar base GS para GS do Kernel (para acessar dados Per-CPU)
    swapgs

    # 2. Trocar Stacks
    # Assumimos que GS:0 contém o espaço de rascunho para RSP do Usuário
    # Assumimos que GS:8 contém o Topo da Stack do Kernel
    movq %rsp, %gs:0         # Salvar RSP do Usuário
    movq %gs:8, %rsp         # Carregar RSP do Kernel

    # 3. Construir TrapFrame na Stack do Kernel
    # Isso deve corresponder ao layout da struct `TrapFrame` no Rust (repr(C))
    
    # --- Frame de Hardware Simulado (para consistência com Interrupções) ---
    pushq $0x1b              # SS (Segmento de Dados Usuário | RPL 3)
    pushq %gs:0              # RSP (Restaurado do rascunho)
    pushq %r11               # RFLAGS (Salvo pelo SYSCALL)
    pushq $0x23              # CS (Segmento de Código Usuário | RPL 3)
    pushq %rcx               # RIP (Salvo pelo SYSCALL)

    # --- Registradores de Propósito Geral ---
    pushq %rax
    pushq %rbx
    pushq %rcx               # Salvo para completude da struct (contém RIP)
    pushq %rdx
    pushq %rbp
    pushq %rdi
    pushq %rsi
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11               # Salvo para completude da struct (contém RFLAGS)
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    # 4. Chamar Dispatcher Rust
    # fn syscall_dispatcher(frame: &mut TrapFrame)
    movq %rsp, %rdi          # Arg 1: Ponteiro para TrapFrame
    
    # Alinhar stack se necessário (RSP está alinhado em 8 bytes aqui devido a 20 pushes)
    # A convenção de chamada requer alinhamento de 16 bytes.
    # Empilhamos 20 qwords (160 bytes), então se começamos alinhados, estamos alinhados.
    call syscall_dispatcher

    # 5. Restaurar Contexto
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11                # Restaurar R11 (Temporário)
    popq %r10
    popq %r9
    popq %r8
    popq %rsi
    popq %rdi
    popq %rbp
    popq %rdx
    popq %rcx                # Restaurar RCX (Temporário)
    popq %rbx
    popq %rax                # Restaurar RAX (Valor de Retorno)

    # Stack Atual: [RIP, CS, RFLAGS, RSP, SS]
    
    # 6. Preparar para SYSRETQ
    # Requisitos do SYSRETQ:
    # - RCX deve conter o RIP de destino
    # - R11 deve conter o RFLAGS de destino
    # - RSP deve ser restaurado manualmente
    
    popq %rcx                # Carregar RIP do Usuário em RCX
    addq $8, %rsp            # Pular CS (não usado pelo SYSRETQ)
    popq %r11                # Carregar RFLAGS do Usuário em R11
    
    # Crítico: Restaurar RSP implica usar Stack do Usuário em Ring 0 por algumas instruções.
    popq %rsp                # Carregar RSP do Usuário
    
    # Pular SS (que agora é efetivamente "desempilhado" porque movemos RSP para longe dele)
    # O valor de SS permanece na stack do kernel que agora está abandonada.

    # 7. Restaurar GS do Usuário
    swapgs

    # 8. Retornar para Modo de Usuário
    sysretq
