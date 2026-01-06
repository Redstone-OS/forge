# SMP Trampoline - Assembly Code
#
# Este código é executado pelos APs após receberem o SIPI.
# Transiciona de Real Mode para Long Mode.
#
# Layout em 0x8000:
#   0x8000 - 0x802F: Real Mode (16-bit)
#   0x8030 - 0x805F: GDT (48 bytes)
#   0x8060 - 0x80AF: Protected Mode (32-bit)
#   0x80B0 - 0x80FF: Long Mode (64-bit)
#   0x8100+:         TrampolineData

.section .rodata
.global ap_trampoline_start
.global ap_trampoline_end

ap_trampoline_start:

# ===========================================================================
# REAL MODE 16-bit - Offset 0x00
# CS:IP = 0x0800:0x0000 após SIPI
# ===========================================================================
.code16

    cli

    # Debug: output 'R'
    mov $0x52, %al
    out %al, $0xE9

    # Clear segment registers
    xor %ax, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss

    # Enable A20 line
    in $0x92, %al
    or $0x02, %al
    out %al, $0x92

    # Load temporary GDT
    lgdt (0x8030)

    # Enable Protected Mode
    mov %cr0, %eax
    or $0x01, %al
    mov %eax, %cr0

    # Far jump to Protected Mode code
    ljmp $0x08, $0x8060

# Padding until offset 0x30
.align 16
.org ap_trampoline_start + 0x30

# ===========================================================================
# GDT - Offset 0x30
# ===========================================================================

gdt_ptr:
    .word gdt_end - gdt_start - 1
    .long 0x8038
    .word 0

gdt_start:
    # Entry 0: Null
    .quad 0

    # Entry 1 (0x08): Code 32-bit
    .word 0xFFFF
    .word 0x0000
    .byte 0x00
    .byte 0x9A
    .byte 0xCF
    .byte 0x00

    # Entry 2 (0x10): Data 32/64-bit
    .word 0xFFFF
    .word 0x0000
    .byte 0x00
    .byte 0x92
    .byte 0xCF
    .byte 0x00

    # Entry 3 (0x18): Code 64-bit (L=1, D=0)
    .word 0xFFFF
    .word 0x0000
    .byte 0x00
    .byte 0x9A
    .byte 0xAF
    .byte 0x00

    # Entry 4 (0x20): Data 64-bit
    .word 0xFFFF
    .word 0x0000
    .byte 0x00
    .byte 0x92
    .byte 0xAF
    .byte 0x00

gdt_end:

.org ap_trampoline_start + 0x60

# ===========================================================================
# PROTECTED MODE 32-bit - Offset 0x60
# ===========================================================================
.code32

pm_entry:
    # Debug: output 'P'
    mov $0x50, %al
    out %al, $0xE9

    # Load data segments
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss

    # Load CR3 from TrampolineData
    mov $0x8100, %edi
    mov (%edi), %eax
    mov %eax, %cr3

    # Enable PAE
    mov %cr4, %eax
    or $0x20, %eax
    mov %eax, %cr4

    # Enable Long Mode via EFER
    mov $0xC0000080, %ecx
    rdmsr
    or $0x100, %eax
    wrmsr

    # Enable Paging
    mov %cr0, %eax
    or $0x80000000, %eax
    mov %eax, %cr0

    # Far jump to Long Mode
    push $0x18
    push $0x80B0
    retf

.org ap_trampoline_start + 0xB0

# ===========================================================================
# LONG MODE 64-bit - Offset 0xB0
# ===========================================================================
.code64

lm_entry:
    # Debug: output 'L'
    mov $0x4C, %al
    out %al, $0xE9

    # Load data segments
    mov $0x20, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    xor %ax, %ax
    mov %ax, %fs
    mov %ax, %gs

    # Load TrampolineData address
    mov $0x8100, %edi

    # Load stack pointer
    mov 0x18(%rdi), %rsp

    # Load ap_id
    mov 0x20(%rdi), %esi

    # Load rust_entry
    mov 0x28(%rdi), %rax

    # Set first argument = ap_id
    mov %esi, %edi

    # Debug: output 'E'
    mov $0x45, %al
    out %al, $0xE9

    # Call Rust entry point
    call *%rax

    # Halt loop
halt_loop:
    cli
    hlt
    jmp halt_loop

ap_trampoline_end:
