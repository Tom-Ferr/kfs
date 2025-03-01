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

    kernel_text = .;

    .kernel_text ALIGN (0x1000) : AT(ADDR(.kernel_text)-0xC0000000)
    {
        *(.kernel_text)
        *(.text.*)
        *(.gdt)
        *(.idt)
        *(.panic)
        *(.user_mode)
        *(.syscalls)
    }

    kernel_rodata = .;

    .kernel_rodata ALIGN (0x1000) : AT(ADDR(.kernel_rodata)-0xC0000000)
    {
        *(.rodata.*)
    }

    kernel_data = .;

    .kernel_data ALIGN (0x1000) : AT(ADDR(.kernel_data)-0xC0000000)
    {
        *(.data)
        *(.data.*)
    }

    kernel_bss = .;

    .bss ALIGN (0x1000) : AT(ADDR(.bss)-0xC0000000)
    {
        *(.bss)
        *(.bss.*)
    }

    kernel_end = .;

}
