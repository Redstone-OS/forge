/* Redstone OS Kernel Linker Script */

ENTRY(_start)

SECTIONS {
/* Carregar em 1MiB (Convencional para Kernels).
Evita a BIOS Data Area (0x400-0x500) e o Bootloader (geralmente < 1MB).
*/
. = 0x100000;

.text : ALIGN(4K) {
    __text_start = .;
    /* Garante que o Header Multiboot/Entry Point fique no início */
    KEEP(*(.text._start))
    *(.text .text.*)
    __text_end = .;
}

.rodata : ALIGN(4K) {
    __rodata_start = .;
    *(.rodata .rodata.*)
    __rodata_end = .;
}

.data : ALIGN(4K) {
    __data_start = .;
    *(.data .data.*)
    __data_end = .;
}

.bss : ALIGN(4K) {
    __bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    __bss_end = .;
}

/* Stack Guard ou seções descartáveis */
/DISCARD/ : {
    *(.eh_frame)
    *(.note .note.*)
}


}