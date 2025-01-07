global enable_paging

enable_paging:
    mov eax, [esp+4]    ; read address from parameter
    mov cr3, eax        ; update cr3

    mov eax, cr4        ; read current cr4
    or  eax, 0x00000010 ; set Page Size Extension (4MB)
    mov cr4, eax        ; update cr4

    mov eax, cr0        ; read current cr0
    or  eax, 0x80000001 ; set Paging and Protected Mode
    mov cr0, eax        ; update cr0
    xor eax, eax
    ret