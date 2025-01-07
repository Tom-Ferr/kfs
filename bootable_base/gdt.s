global gdt_flush

gdt_flush:
    mov eax, [esp+4]
    lgdt [eax]
    mov ax, 0x10      ; 0x10 is the offset in the GDT to our data segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ax, 0x18     ; 0x18 is the offset in the GDT to our stack segment
    mov ss, ax
    jmp 0x08:.flush   ; 0x08 is the offset to our code segment: Far jump!
.flush:
    ret