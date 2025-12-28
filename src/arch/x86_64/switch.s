# Arquivo: switch.s
#
# Propósito: Rotina de troca de contexto (Context Switch) em nível de thread/processo.
# Responsável por salvar os registradores "callee-saved" da thread atual e restaurar
# os registradores da próxima thread.
#
# Assinatura Rust esperada:
# `pub unsafe extern "C" fn context_switch(prev: *mut Context, next: *const Context);`
#
# Detalhes de Implementação:
# - Salva RBX, RBP, R12, R13, R14, R15 (Callee-saved pela ABI System V).
# - Troca o Stack Pointer (RSP).
# - O RIP de retorno é automaticamente tratado pelo `call`/`ret`.
# - RFLAGS é preservado parcialmente pela convenção de chamada (flags de status).

.section .text
.global context_switch
.code64

# --------------------------------------------------------------------------------
# context_switch
#
# Troca o contexto de execução entre duas threads.
#
# Argumentos (ABI System V):
# - RDI: Ponteiro para estrutura onde salvar o contexto atual (`prev`)
# - RSI: Ponteiro para estrutura de onde carregar o próximo contexto (`next`)
#
# A estrutura `Context` deve ter o layout:
# struct Context {
#     r15: u64,
#     r14: u64,
#     r13: u64,
#     r12: u64,
#     rbx: u64,
#     rbp: u64,
#     rip: u64, // Empilhado pelo call, desempilhado pelo ret
# }
#
# Nota: O layout exato da struct Context não importa tanto quanto a ordem
# de push/pop bater com a struct no Rust.
# --------------------------------------------------------------------------------
context_switch:
    # 1. Salvar registradores callee-saved na stack da thread ATUAL
    pushq %rbx
    pushq %rbp
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    # 2. Salvar Stack Pointer atual na struct `prev` (primeiro argumento, RDI)
    # Assumimos que o primeiro campo de `prev` é o RSP ou que `Context` é apenas um ponteiro de stack.
    # Se `Context` for uma struct contendo registradores, o ponteiro da struct aponta para o topo salvo.
    movq %rsp, (%rdi)

    # 3. Carregar Stack Pointer da próxima thread de `next` (segundo argumento, RSI)
    movq (%rsi), %rsp

    # 4. Restaurar registradores callee-saved da stack da thread PRÓXIMA
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbp
    popq %rbx

    # 5. Retornar
    # O `ret` vai desempilhar o RIP que foi salvo na stack da PRÓXIMA thread
    # quando ela chamou `context_switch` pela última vez (ou foi forjada).
    ret