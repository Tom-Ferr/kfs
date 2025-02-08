section .user_mode

global switch_to_user_mode

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
