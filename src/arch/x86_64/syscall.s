

.section .text
.global syscall_entry
.code64

# Syscall entry point
# Convenção: RCX = RIP de retorno, R11 = RFLAGS
# SYSCALL clobbers RCX e R11

syscall_entry:
    swapgs

    # Salvar RSP do user e carregar kernel RSP
    mov gs:0, rsp
    mov rsp, gs:8

    # Construir frame para IRETQ (5 * 8 = 40 bytes)
    push 0x1b               # SS = User Data
    push QWORD PTR gs:0     # RSP do user
    push r11                # RFLAGS (salvo por SYSCALL)
    push 0x23               # CS = User Code  
    push rcx                # RIP do user (salvo por SYSCALL)

    # Salvar registradores de propósito geral (15 * 8 = 120 bytes)
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rdi
    push rsi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    # Chamar dispatcher Rust
    # RDI = ponteiro para o TrapFrame (primeiro argumento)
    mov rdi, rsp
    call syscall_dispatcher

    # Restaurar registradores
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax     # RAX foi modificado pelo dispatcher com o resultado

    # Trocar GS de volta para userspace
    swapgs
    
    # Retornar ao userspace via IRETQ
    # Stack tem: RIP, CS, RFLAGS, RSP, SS
    iretq
