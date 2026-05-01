ENTRY(_start)

SECTIONS
{
    . = 0x41000000;

    .text :
    {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(4096);
    . += 0x100000;
    __stack_top = .;
}
