section .syscalls
global fork
global exit
global swtch
global get_pid

fork:
    mov eax, 3
    int 0x80
    ret

exit:
    mov ebx, [esp+4]
    mov eax, 4
    int 0x80
    ret

swtch:
    mov eax, 5
    int 0x80
    ret

get_pid:
    mov eax, 6
    int 0x80
    ret