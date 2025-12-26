.global syscall_handler
.extern syscall_dispatcher

.section .text

# Handler para int 0x80
# ABI:
# RAX = Syscall Number
# RDI = Arg1
# RSI = Arg2
# RDX = Arg3
# R10 = Arg4
# R8  = Arg5
# R9  = Arg6
syscall_handler:
    # 1. Salvar contexto (parcial, pois é Syscall, mas interrupt gate salva tudo)
    # Stack layout pelo CPU (Interrupt Gate): SS, RSP, RFLAGS, CS, RIP
    # Precisamos salvar o resto para preservar o estado do user process.
    push rbp
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push rdi
    push rsi
    push rdx
    push rcx
    push rbx
    push rax  # Push RAX original (Syscall Num)

    # 2. Preparar argumentos para o Rust
    # fn syscall_dispatcher(context: &mut ContextFrame)
    mov rdi, rsp

    # 3. Chamar Handler Rust
    call syscall_dispatcher

    # 4. Restaurar contexto
    # O Rust pode ter alterado o contexto (ex: RAX = retorno), então de pop em tudo.
    pop rax
    pop rbx
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    pop rbp

    # 5. Retornar (IRETQ)
    iretq
