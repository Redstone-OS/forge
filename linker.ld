/* Redstone OS Kernel Linker Script */

ENTRY(_start)

SECTIONS {
    /* Virtual Memory Address (High Half) */
    . = 0xFFFFFFFF80000000;
    
    /* Offset entre Virtual e Físico */
    __kernel_offset = 0xFFFFFFFF80000000 - 0x100000;

    .text : AT(ADDR(.text) - __kernel_offset) {
        __text_start = .;
        /* Garante que o Entry Point fique no início */
        KEEP(*(.text._start))
        *(.text .text.*)
        __text_end = .;
    }

    .rodata : AT(ADDR(.rodata) - __kernel_offset) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data : AT(ADDR(.data) - __kernel_offset) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss : AT(ADDR(.bss) - __kernel_offset) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /* Debug sections explicitly kept for Debug builds */
    .debug_abbrev : { *(.debug_abbrev) }
    .debug_info : { *(.debug_info) }
    .debug_aranges : { *(.debug_aranges) }
    .debug_ranges : { *(.debug_ranges) }
    .debug_str : { *(.debug_str) }
    .debug_pubnames : { *(.debug_pubnames) }
    .debug_pubtypes : { *(.debug_pubtypes) }
    .debug_frame : { *(.debug_frame) }
    
    /* Stack Guard ou seções descartáveis */
    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}