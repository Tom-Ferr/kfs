global start
extern kernel
extern enable_paging


section .text
bits 32
check_multiboot:
    ; check the bootloader wrote its magic value in eax before loading our kernel
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    ; ERR:  0, our kernel wasn't launched by a multiboot compliant bootloader (shouldn't happen with GRUB)
    mov al, "0"
    jmp .error
.error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

init_table:
    xor eax, eax
    or eax, 3
    mov ecx, 1024
    mov edi, page_table - 0xC0000000
.map_pages:
    stosd                ; Store the value in EAX at the address pointed by EDI
    add eax, 0x1000      ; Increment EAX by 4 KB (next physical page)
    loop .map_pages      ; Decrement ECX, and repeat until ECX = 0
    jmp .setup_directory
.setup_directory:
    mov eax, page_table - 0xC0000000
    or eax, 3
    mov edi, directory_table - 0xC0000000
    mov [edi], eax
    mov [edi + 0xC00], eax
    mov eax, virtual_space - 0xC0000000
    or eax, 3
    mov [edi + 0xE30], eax
    ret

start:
    call check_multiboot
    call init_table

    mov eax, directory_table - 0xC0000000
.enable_paging:    
    mov cr3, eax        ; update cr3
    mov eax, cr0        ; read current cr0
    or  eax, 0x80000001 ; set Paging and Protected Mode
    mov cr0, eax        ; update cr0

    lea ecx, [rel higher_half]
    jmp ecx


section .kernel_text
higher_half:
    xor eax, eax
    mov [directory_table], eax
    mov ecx, cr3
    mov cr3, ecx
    cli                 ; Disable interrupts
    mov esp, stack_top  ; Set stack pointer to top of stack
    and esp, 0xFFFFFFF0 ; Ensure 16-byte alignment
    mov ebp, esp        ; Initialize base pointer
    push ebx
    call kernel
    hlt


section .bss
align 4096
directory_table:
    resb 4096
page_table:
    resb 4096
virtual_space:
    resb 4096

align 16
stack_bottom:
    resb 4096 * 4
stack_top:
