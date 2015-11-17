extern handle_exception

section .text
bits 64

; This is the ASM version of the `InterruptCtx64` structure defined in
; `interrupts.rs`.
;
; We use this for stashing an interrupt context to pass to the Rust
; interrupt handlers.
struc InterruptCtx
    ; the registers rsi, rdi, r11, r10, r9, r8, rdx, rcx, and rax
    .regs: resq 9
endstruc
