ENTRY(_start)

SECTIONS {
    . = 0xe0100000;

    kernel_start = .;

    .text ALIGN(0x1000) : AT(ADDR(.text) - 0xe0000000) {
        *(.text.boot.*)
        *(.text.*)
    }

    .rodata ALIGN(0x1000) : AT(ADDR(.rodata) - 0xe0000000) {
        *(.rodata.*)
    }

    .data ALIGN(0x1000) : AT(ADDR(.data) - 0xe0000000) {
        *(.data.*)
    }

    .bss ALIGN(0x1000) : AT(ADDR(.bss) - 0xe0000000) {
        *(COMMON)
        *(.bss.*)
    }

    kernel_end = .;
}
