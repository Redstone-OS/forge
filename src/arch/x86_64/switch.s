.global context_switch
.intel_syntax noprefix

# void context_switch(u64* old_stack_ptr, u64 new_stack_ptr);
# RDI = old_stack_ptr (ponteiro para u64 onde guardamos o RSP antigo)
# RSI = new_stack_ptr (o valor do novo RSP)

context_switch:
    # 1. Salvar registradores "callee-saved" na stack atual
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
    # RFLAGS já é salvo parcialmente pelo comportamento do sistema, 
    # mas para robustez completa, poderíamos usar pushfq.
    # Por hora, seguimos o padrão System V ABI básico.

    # 2. Se old_stack_ptr (RDI) não for 0, salvar o RSP atual lá
    test rdi, rdi
    jz .load_new
    mov [rdi], rsp

.load_new:
    # 3. Carregar nova stack
    mov rsp, rsi

    # 4. Restaurar registradores da nova stack
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx

    # 5. Retornar (o RIP de retorno já está na nova stack)
    ret