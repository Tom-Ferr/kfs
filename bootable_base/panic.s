section .panic
global panic_halt

panic_halt:
    cli
    xor eax, eax
    xor ecx, ecx
    xor edx, edx
    xor ebx, ebx
    ;xor esp, esp
    ;xor ebp, ebp
    xor esi, esi
    xor edi, edi
.halt:
    hlt
    jmp .halt