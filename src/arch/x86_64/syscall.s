.global syscall_entry_asm
.extern syscall_rust_entry

.section .text

syscall_entry_asm:
    push r15
    push r14
    push r13
    push r12
    push rbp
    push rbx
    push rcx
    push r11

    push r9
    mov r9, r8
    mov r8, r10
    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax

    call syscall_rust_entry
    add rsp, 8

    pop r11
    pop rcx
    pop rbx
    pop rbp
    pop r12
    pop r13
    pop r14
    pop r15

    sysretq
