ENTRY(_start)

SECTIONS {
    /* Endereço base inicial - não zero para evitar entry point inválido */
    /* O bootloader vai realocar de qualquer forma */
    . = 0x1000;

    /* TODO: Revisar alinhamento de 1MB futuramente (sugerido para evitar colisão/NOT_FOUND em alocação UEFI) */
    .text : ALIGN(0x1000) {
        __text_start = .;
        /* Garante que _start (header) fique no começo */
        *(.text._start)
        *(.text*)
        __text_end = .;
    }

    .rodata : ALIGN(0x1000) {
        __rodata_start = .;
        *(.rodata*)
        __rodata_end = .;
    }

    .data : ALIGN(0x1000) {
        *(.data*)
    }

    .bss : ALIGN(0x1000) {
        *(.bss*)
        *(COMMON)
    }

    .alt : ALIGN(0x1000) {
        __altrelocs_start = .;
        *(.alt.reloc*)
        __altrelocs_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note*)
        *(.comment*)
    }
}
