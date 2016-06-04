global isrs

extern handle_interrupt
extern handle_cpu_exception

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

%macro  ex_err 1
isr_%1:
        ; the error code has already been pushed, so we just push the ID
        ; TODO: interrupt IDs can probably just be a byte?
        push    qword %1         ; push interrupt ID
        jmp     call_ex_handler ; call into Rust handler
%endmacro

%macro  exception 1
isr_%1:
        push    qword 0         ; push fake error code of 0
        push    qword %1         ; push interrupt ID
        jmp     call_ex_handler ; call into Rust handler
%endmacro

%macro  interrupt 1
isr_%1:
        push    qword %1         ; push interrupt ID
        jmp     call_handler    ; call into Rust handler
%endmacro

exception 0   ; divide-by-zero exception
exception 1   ; debug exception
exception 2   ; non-maskable intterupt
exception 3   ; breakpoint exception
exception 4   ; overflow exception
exception 5   ; bound-range exception
exception 6   ; invalid opcode exception
exception 7   ; device not available exception
ex_err    8   ; double fault exception
; exception 9 ; coprocessor segment overrun exception (reserved in x86_64)
ex_err    10  ; invalid TSS exception
ex_err    11  ; segment not present exception
ex_err    12  ; stack exception
ex_err    13  ; general protection fault exception
ex_err    14  ; page fault exception
; exception 15 ; reserved
exception 16  ; x87 floating-point exception
ex_err    17  ; alignment check exception
exception 18  ; machine check exception
exception 19  ; SIMD floating-point exception

;;; Fill in handlers 32 through 255.
%assign i 32
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
    add     rsp, 8              ; skip past the interrupt id
    iretq

;;; Call into the Rust interrupt handler function
call_ex_handler:
    push_ctx                     ; push the context registers to the stack
    mov     rdi, rsp             ; push pointer to interrupt data

    call    handle_cpu_exception ; call the Rust interrupt handler

    pop_ctx                      ; pop context off of the stack
    add     rsp, 16              ; skip past the interrupt id & err code
    iretq


section .rodata
isrs:
    dq isr_0
    dq isr_1
    dq isr_2
    dq isr_3
    dq isr_4
    dq isr_5
    dq isr_6
    dq isr_7
    dq isr_8
    dq 0
    dq isr_10
    dq isr_11
    dq isr_12
    dq isr_13
    dq isr_14
    dq 0
    dq isr_16
    dq isr_17
    dq isr_18
    dq isr_19
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
        dq isr_%+i
%assign i i+1
%endrep
