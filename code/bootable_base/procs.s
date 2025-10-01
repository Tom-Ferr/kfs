section .procs

global switch_to_user_mode
global run_proc

switch_to_user_mode:
    cli
    mov ax, 0x2b	    ; user mode data selector is 0x28 (GDT entry 4). Also sets RPL to 3 (0x28 | 0x3)
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    xor eax, eax
    mov edi, [esp+4]
    mov esi, [esp+8]
    push 0x33	        ; SS user mode stack selector is 0x30. With RPL 3 this is 0x33
    push edi	        ; ESP
    pushfd			    ; EFLAGS
    pop eax             ; READ EFLAGS
    or eax, 0x200       ; ENABLE INTERRUPT
    push eax            ; PUSH MODIFED EFLAGS
    push 0x23		    ; CS, user mode code selector is 0x20. With RPL 3 this is 0x23
    push esi            ; EIP
    iret

run_proc:
    mov eax, [esp+4]
    mov ebx, [eax+4]
    mov ds, bx
    mov es, bx
    mov fs, bx
    mov gs, bx

    mov edi, [eax+8]
    mov esi, [eax+12]
    mov ebp, [eax+16]
    mov ebx, [eax+24]
    mov edx, [eax+28]
    mov ecx, [eax+32]

    push DWORD[eax+64] ;SS
    push DWORD[eax+60] ;ESP
    push DWORD[eax+56] ;FLAGS
    push DWORD[eax+52] ;CS
    push DWORD[eax+48] ;EIP

    mov eax, [eax+36]
    sti
    iret