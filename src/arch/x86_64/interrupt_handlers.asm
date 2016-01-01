global  interrupt_handlers

extern  handle_interrupt

section .text
bits    64

;;; Push all the caller-saved registers onto the stack.
;;;
;;; The caller-saved registers need to be pushed in the opposite order as the
;;; fields in the `Registers` structure defined in `context.rs`; otherwise,
;;; we won't be able to access them correctly from Rust
%macro  push_ctx 0
        push rax
        push rcx
        push rdx
        push r8
        push r9
        push r10
        push r11
        push rdi
        push rsi
%endmacro

;;; Pop all the caller-saved registers off the stack
%macro  pop_ctx 0
        pop rsi
        pop rdi
        pop r11
        pop r10
        pop r9
        pop r8
        pop rdx
        pop rcx
        pop rax
%endmacro

%macro  int_err 1
interrupt_%1:
        ; the error code has already been pushed, so we just push the ID
        push    qword %1        ; push interrupt ID
        jmp     call_handler    ; call into Rust handler
%endmacro

%macro  interrupt 1
interrupt_%1:
        push    qword 0         ; push fake error code of 0
        push    qword %1        ; push interrupt ID
        jmp     call_handler    ; call into Rust handler
%endmacro

interrupt   0   ; divide-by-zero exception
interrupt   1   ; debug exception
interrupt   2   ; non-maskable intterupt
interrupt   3   ; breakpoint exception
interrupt   4   ; overflow exception
interrupt   5   ; bound-range exception
interrupt   6   ; invalid opcode exception
interrupt   7   ; device not available exception
int_err     8   ; double fault exception
; int_err   9   ; coprocessor segment overrun exception (reserved in x86_64)
int_err     10  ; invalid TSS exception
int_err     11  ; segment not present exception
int_err     12  ; stack exception
int_err     13  ; general protection fault exception
int_err     14  ; page fault exception
; interrupt 15  ; reserved
interrupt   16  ; x87 floating-point exception
int_err     17  ; alignment check exception
interrupt   18  ; machine check exception
interrupt   19  ; SIMD floating-point exception

;;; Fill in handlers 32 through 255.
interrupt 32
%assign i 33
%rep    224
interrupt i
%assign i i+1
%endrep

;;; Call into the Rust interrupt handler function
call_handler:
    push_ctx                    ; push the context registers to the stack
    mov     rdi, rsp            ; push pointer to interrupt data

    call    handle_interrupt    ; call the Rust interrupt handler

    pop_ctx                     ; pop context off of the stack
    add     rsp, 16             ; skip past the interrupt id & err code
    iretq


section .rodata
interrupt_handlers:
    dq interrupt_0
    dq interrupt_1
    dq interrupt_2
    dq interrupt_3
    dq interrupt_4
    dq interrupt_5
    dq interrupt_6
    dq interrupt_7
    dq interrupt_8
    dq 0
    dq interrupt_10
    dq interrupt_11
    dq interrupt_12
    dq interrupt_13
    dq interrupt_14
    dq 0
    dq interrupt_16
    dq interrupt_17
    dq interrupt_18
    dq interrupt_19
    dq 0
    dq 0
    dq 0
    dq 0
    dq 0
    dq 0
    dq 0
    dq 0
    dq 0
    dq 0
    dq 0                    ; interrupt_30
    dq 0
%assign i 32
%rep    224
        dq interrupt_%+i
%assign i i+1
%endrep
