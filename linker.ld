ENTRY(_start)

SECTIONS {
    . = 0xFFFFFFFF80000000;
    __kernel_vaddr = .;
    __kernel_paddr = 0x00100000;

    .text : AT(__kernel_paddr) {
        __text_start = .;
        KEEP(*(.text._start))
        *(.text .text.*)
        __text_end = .;
    }

    .rodata : AT(ADDR(.rodata) - __kernel_vaddr + __kernel_paddr) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data : AT(ADDR(.data) - __kernel_vaddr + __kernel_paddr) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss : AT(ADDR(.bss) - __kernel_vaddr + __kernel_paddr) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /* debug mantido */
    .debug_info : { *(.debug_info) }
    .debug_frame : { *(.debug_frame) }
    .debug_abbrev : { *(.debug_abbrev) }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
