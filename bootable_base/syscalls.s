section .syscalls
global fork

fork:
    mov eax, 3
    int 0x80
    ret