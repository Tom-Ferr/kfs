section .syscalls
global fork

fork:
    mov eax, 3
    int 0x80
    cmp eax, 2
    jne .loop
    ret
.loop:
    jmp .loop