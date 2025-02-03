ENTRY(start)

SECTIONS {
    . = 1M;
    
    kernel_start = .;

    .text ALIGN (0x1000) :
    {
        *(.multiboot_header)
        *(.text)
    }

    . += 0xC0000000;

    .kernel_text ALIGN (0x1000) : AT(ADDR(.kernel_text)-0xC0000000)
    {
        *(.kernel_text)
        *(.text.*)
        *(.gdt)
        *(.idt)
        *(.panic)
    }
    
    .bss ALIGN (0x1000) : AT(ADDR(.bss)-0xC0000000)
    {
        *(.bss)
    }

    .kernel_rodata ALIGN (0x1000) : AT(ADDR(.kernel_rodata)-0xC0000000)
    {
        *(.rodata.*)
    }

    .kernel_data ALIGN (0x1000) : AT(ADDR(.kernel_data)-0xC0000000)
    {
        *(.data)
        *(.data.*)
    }

    .kernel_bss ALIGN (0x1000) : AT(ADDR(.kernel_bss)-0xC0000000)
    {
        *(.bss.*)
    }

    kernel_end = .;

}
