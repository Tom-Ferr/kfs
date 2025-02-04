section .user_mode

global switch_to_user_mode

switch_to_user_mode:
    cli
    mov ax, 0x2b	; user mode data selector is 0x28 (GDT entry 4). Also sets RPL to 3 (0x28 | 0x3)
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    xor eax, eax
    push 0x3b	    ; SS user mode stack selector is 0x38. With RPL 3 this is 0x3b
    push [esp+4]	; ESP
    pushfd			; EFLAGS
    pop, eax        ; READ EFLAGS
    or eax, 0x200   ; ENABLE INTERRUPT
    push eax        ; PUSH MODIFED EFLAGS
    push 0x33		; CS, user mode code selector is 0x30. With RPL 3 this is 0x33
    push [esp+8]    ; EIP
    iret
