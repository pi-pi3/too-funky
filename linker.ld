ENTRY(_start)

SECTIONS {
    . = 0xe0100000;

    kernel_start = .;

    .boot : AT(ADDR(.boot) - 0xe0000000) {
        *(.multiboot2.*)
        KEEP(*(.multiboot2.*))
    }

    .text ALIGN(0x1000) : AT(ADDR(.text) - 0xe0000000) {
        *(.text.*)
        KEEP(*(.text.*))
    }

    .rodata ALIGN(0x1000) : AT(ADDR(.rodata) - 0xe0000000) {
        *(.rodata.*)
        KEEP(*(.rodata.*))
    }

    .data ALIGN(0x1000) : AT(ADDR(.data) - 0xe0000000) {
        *(.data.*)
        KEEP(*(.data.*))
    }

    .bss ALIGN(0x1000) : AT(ADDR(.bss) - 0xe0000000) {
        *(COMMON)
        *(.bss.*)
        KEEP(*(.bss.*))
    }

    kernel_end = .;
}
