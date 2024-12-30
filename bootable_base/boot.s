global start
extern kernel
section .text
bits 32
start:
    cli                 ; Disable interrupts
    mov esp, stack_top  ; Set stack pointer to top of stack
    and esp, 0xFFFFFFF0 ; Ensure 16-byte alignment
    mov ebp, esp        ; Initialize base pointer
    call check_multiboot
    call kernel
    hlt

check_multiboot:
    ; check the bootloader wrote its magic value in eax before loading our kernel
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret

.no_multiboot:
    ; ERR:  0, our kernel wasn't launched by a multiboot compliant bootloader (shouldn't happen with GRUB)
    mov al, "0"
    jmp error

error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

section .bss
align 16
stack_bottom:
    resb 4096 * 4
stack_top: