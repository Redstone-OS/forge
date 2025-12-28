.section .text
.global syscall_entry
.code64

syscall_entry:
    swapgs

    mov gs:[0], rsp
    mov rsp, gs:[8]

    push 0x1b
    push qword ptr gs:[0]
    push r11
    push 0x23
    push rcx

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

    mov rdi, rsp
    
    call syscall_dispatcher

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
    pop rax

    pop rcx
    add rsp, 8
    pop r11
    
    pop rsp
    
    swapgs

    sysretq
