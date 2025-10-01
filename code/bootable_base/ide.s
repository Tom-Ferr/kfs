global pio_read
global pio_write

pio_read:
    mov ax, [esp+4]
    mov dx, [esp+8]
    mov cx, [esp+12]
    mov edi, [esp+16]

    push es
    mov es, ax
    rep insw
    pop es

pio_write:
    mov ax, [esp+4]
    mov dx, [esp+8]
    mov cx, [esp+12]
    mov esi, [esp+16]

    push ds
    mov ds, ax
    rep outsw
    pop ds
