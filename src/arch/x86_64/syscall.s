
.section .text
.global syscall_entry
.extern syscall_dispatcher

syscall_entry:
    // TODO: Salvar estado (swapgs)
    // TODO: Chamar syscall_dispatcher
    // TODO: Restaurar estado (sysretq)
    sysretq
