.intel_syntax noprefix
.section .text
.global divide_error_wrapper
.global invalid_opcode_wrapper
.global page_fault_wrapper
.global general_protection_wrapper
.global double_fault_wrapper
.global breakpoint_wrapper
.global timer_handler

.extern divide_error_handler_inner
.extern invalid_opcode_handler_inner
.extern page_fault_handler_inner
.extern general_protection_handler_inner
.extern double_fault_handler_inner
.extern breakpoint_handler_inner
.extern timer_handler_inner
.extern should_reschedule
.extern clear_need_resched
.extern schedule

# =============================================================================
# MACROS
# =============================================================================

.macro PUSH_SCRATCH_REGS
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
.endm

.macro POP_SCRATCH_REGS
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax
.endm

# =============================================================================
# EXCEPTION HANDLERS (WRAPPERS)
# =============================================================================

# -----------------------------------------------------------------------------
# DIVIDE ERROR (#DE) - NO ERROR CODE
# -----------------------------------------------------------------------------
divide_error_wrapper:
    PUSH_SCRATCH_REGS
    # Checar se veio de User Mode (CS & 3 == 3)
    # CS esta em [rsp + 72 + 8] = [rsp + 80]
    test byte ptr [rsp + 80], 3
    jz .L_div_no_swap_in
    swapgs
.L_div_no_swap_in:
    lea rdi, [rsp + 72]      # RDI = Ponteiro para stack frame
    call divide_error_handler_inner
    
    test byte ptr [rsp + 80], 3
    jz .L_div_no_swap_out
    swapgs
.L_div_no_swap_out:
    POP_SCRATCH_REGS
    iretq

# -----------------------------------------------------------------------------
# INVALID OPCODE (#UD) - NO ERROR CODE
# -----------------------------------------------------------------------------
invalid_opcode_wrapper:
    PUSH_SCRATCH_REGS
    test byte ptr [rsp + 80], 3
    jz .L_ud_no_swap_in
    swapgs
.L_ud_no_swap_in:
    lea rdi, [rsp + 72]
    call invalid_opcode_handler_inner
    test byte ptr [rsp + 80], 3
    jz .L_ud_no_swap_out
    swapgs
.L_ud_no_swap_out:
    POP_SCRATCH_REGS
    iretq

# -----------------------------------------------------------------------------
# BREAKPOINT (#BP) - NO ERROR CODE
# -----------------------------------------------------------------------------
breakpoint_wrapper:
    PUSH_SCRATCH_REGS
    test byte ptr [rsp + 80], 3
    jz .L_bp_no_swap_in
    swapgs
.L_bp_no_swap_in:
    lea rdi, [rsp + 72]
    call breakpoint_handler_inner
    test byte ptr [rsp + 80], 3
    jz .L_bp_no_swap_out
    swapgs
.L_bp_no_swap_out:
    POP_SCRATCH_REGS
    iretq

# -----------------------------------------------------------------------------
# PAGE FAULT (#PF) - WITH ERROR CODE
# Stack: [Regs(72), ERR(8), RIP(8), CS(8)]
# -----------------------------------------------------------------------------
page_fault_wrapper:
    PUSH_SCRATCH_REGS
    # CS esta em 72 + 8 + 8 = 88
    test byte ptr [rsp + 88], 3
    jz .L_pf_no_swap_in
    swapgs
.L_pf_no_swap_in:
    lea rdi, [rsp + 80]      # RDI = Stack frame (pula erro)
    mov rsi, [rsp + 72]      # RSI = Error Code
    call page_fault_handler_inner
    
    test byte ptr [rsp + 88], 3
    jz .L_pf_no_swap_out
    swapgs
.L_pf_no_swap_out:
    POP_SCRATCH_REGS
    add rsp, 8            # Pop error code
    iretq

# -----------------------------------------------------------------------------
# GENERAL PROTECTION (#GP) - WITH ERROR CODE
# -----------------------------------------------------------------------------
general_protection_wrapper:
    PUSH_SCRATCH_REGS
    test byte ptr [rsp + 88], 3
    jz .L_gp_no_swap_in
    swapgs
.L_gp_no_swap_in:
    lea rdi, [rsp + 80]
    mov rsi, [rsp + 72]
    call general_protection_handler_inner
    
    test byte ptr [rsp + 88], 3
    jz .L_gp_no_swap_out
    swapgs
.L_gp_no_swap_out:
    POP_SCRATCH_REGS
    add rsp, 8
    iretq

# -----------------------------------------------------------------------------
# DOUBLE FAULT (#DF) - WITH ERROR CODE
# -----------------------------------------------------------------------------
double_fault_wrapper:
    PUSH_SCRATCH_REGS
    test byte ptr [rsp + 88], 3
    jz .L_df_no_swap_in
    swapgs
.L_df_no_swap_in:
    lea rdi, [rsp + 80]
    mov rsi, [rsp + 72]
    call double_fault_handler_inner
    
    test byte ptr [rsp + 88], 3
    jz .L_df_no_swap_out
    swapgs
.L_df_no_swap_out:
    POP_SCRATCH_REGS
    add rsp, 8
    iretq

# =============================================================================
# TIMER HANDLER (IRQ 0)
# =============================================================================
timer_handler:
    PUSH_SCRATCH_REGS
    call timer_handler_inner
    
    # Verifica CS em [RSP + 80]
    mov rax, [rsp + 80]
    and rax, 3
    cmp rax, 3
    jne .L_skip_preemption

    call should_reschedule
    test al, al
    jz .L_skip_preemption

    call clear_need_resched
    
    # Salvar R12 (callee-saved) antes de usar para alinhar stack
    push r12
    mov r12, rsp
    and rsp, -16
    call schedule
    mov rsp, r12
    pop r12

.L_skip_preemption:
    POP_SCRATCH_REGS
    iretq
