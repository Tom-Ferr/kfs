section .panic
global panic_halt

panic_halt:
    cli
.save_stack:
    mov [stack_pointer], esp
    mov ecx, 4096
    lea esi, [esp]
    lea edi, [stack_backup]
    rep movsb
.clean_registers:
    xor eax, eax
    xor ecx, ecx
    xor edx, edx
    xor ebx, ebx
    xor esp, esp
    xor ebp, ebp
    xor esi, esi
    xor edi, edi
.halt:
    hlt
    jmp .halt

section .data
stack_pointer dd 0

section .bss
stack_backup:
    resb 4096