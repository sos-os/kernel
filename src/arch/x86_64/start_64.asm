global start_64

section .text
bits 64
start_64:

    ; call into kernel main function
    extern kernel_main
    call kernel_main

    ; print `OKAY` to screen
    mov rax, 0x2f592f412f4b2f4f
    mov qword [0xb8000], rax
    hlt
