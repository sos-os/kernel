global start_64

section .text
bits 64
start_64:

    ; set SSE mode on
    call    set_SSE

    ; call into kernel main function
    extern  kernel_main
    call    kernel_main

.os_returned:
    ; rust main returned, print `OS returned!`
    mov     rax, 0x4f724f204f534f4f
    mov     [0xb8000], rax
    mov     rax, 0x4f724f754f744f65
    mov     [0xb8008], rax
    mov     rax, 0x4f214f644f654f6e
    mov     [0xb8010], rax
    hlt

; Tests if SSE is enabled
is_SSE:
    mov     rax, 0x1
    cpuid
    test    edx, 1<<25
    jz      .no_SSE

    ret
.no_SSE:
    mov     al, "a"
    jmp     error

; Sets SSE mode
;
; This may be a bad (slow) thing, look into setting no-SSE compiler flags
; instead.
set_SSE:
    call    is_SSE
    mov     rax, cr0
    and     ax, 0xFFFB  ; clear coprocessor emulation CR0.EM
    or      ax, 0x2     ; set coprocessor monitoring  CR0.MP
    mov     cr0, rax
    mov     rax, cr4
    or      ax, 3 << 9  ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
    mov     cr4, rax

    ret

; Prints `ERROR: ` and the given error code to screen and hangs.
; parameter: error code (in ascii) in al
error:
    mov     rbx, 0x4f4f4f524f524f45
    mov     [0xb8000], rbx
    mov     rbx, 0x4f204f204f3a4f52
    mov     [0xb8008], rbx
    mov     byte [0xb800e], al
    hlt
    jmp     error
