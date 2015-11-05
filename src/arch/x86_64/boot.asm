%define PAGE_TABLE_SIZE 512 * 8

; page_map: macro to map the first entry in the first argument to the second
%macro  page_map 2

        mov eax, %2
        or  eax, 0b11 ; present + writable
        mov [%1], eax

%endmacro


global start

section .text
bits 32

; Tests whether or not multiboot is enabled
is_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp err


; Tests whether or not long mode is available.
;
; If long mode is not available, die (we are a long mode OS).
is_long_mode:
    mov eax, 0x80000000    ; Set the A-register to 0x80000000.
    cpuid                  ; CPU identification.
    cmp eax, 0x80000001    ; Compare the A-register with 0x80000001.
    jb .no_long_mode       ; It is less, there is no long mode.
    mov eax, 0x80000001    ; Set the A-register to 0x80000001.
    cpuid                  ; CPU identification.
    test edx, 1 << 29      ; Test if the LM-bit, which is bit 29, is set in the D-register.
    jz .no_long_mode       ; They aren't, there is no long mode.
    ret
.no_long_mode:
    mov al, "2"
    jmp err

is_cpuid:
    pushfd               ; Store the FLAGS-register.
    pop eax              ; Restore the A-register.
    mov ecx, eax         ; Set the C-register to the A-register.
    xor eax, 1 << 21     ; Flip the ID-bit, which is bit 21.
    push eax             ; Store the A-register.
    popfd                ; Restore the FLAGS-register.
    pushfd               ; Store the FLAGS-register.
    pop eax              ; Restore the A-register.
    push ecx             ; Store the C-register.
    popfd                ; Restore the FLAGS-register.
    xor eax, ecx         ; Do a XOR-operation on the A-register and the C-register.
    jz .no_cpuid         ; The zero flag is set, no CPUID.
    ret                  ; CPUID is available for use.
.no_cpuid:
    mov al, "1"
    jmp error

; Prints a boot error code to the VGA buffer
err:
    mov dword [0xb8000], 0x4f524f45
    mov byte  [0xb8004], al
    hlt

; Creates the page tables by mapping:
;   - the first PML4 entry -> PDP
;   - the first PDP entry -> PD
;   - each PD entry to its own 2mB page
create_page_tables:
    page_map    pml4_table, pdp_table   ; map first PML4 entry to PDP table
    page_map    pdp_table,  pd_table    ; map first PDP entry to PD table

    ; map each PD table entry to its own 2mB page
    mov         ecx, 0

.pd_table_map:
    mov         eax, 0x200000   ; 2 mB
    mul         ecx             ; times the start address of the page
    or          eax, 0b10000011 ; check if present + writable + huge

    mov         [pd_table + ecx * 8], eax ; map nth entry from pd -> own page

    ; increment counter and check if done
    inc         ecx
    cmp         ecx, 512
    jne         .pd_table_map

    ret

; Sets long mode and enables paging
set_long_mode:

    ; load PML4 addr to cr3 register
    mov eax, pml4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    sor eax, 1 << 31
    or eax, 1 << 16
    mov cr0, eax

    ret

start:
    mov esp, stack_top

    call is_multiboot
    call is_cpuid
    call is_long_mode

    ; if everything is okay, create the page tables and start long mode
    call create_page_tables
    call set_long_mode

    ; load the 64-bit GDT
    lgdt [gdt64.ptr]

    ; update selectors
    mov ax, 16
    mov ss, ax  ; stack selector
    mov ds, ax  ; data selector
    mov es, ax  ; extra selector

    ; print `OK` to screen
    mov dword [0xb8000], 0x2f4b2f4f
    hlt


section .bss
align 4096
pml4_table:                 ; Page-Map Level-4 Table
    resb PAGE_TABLE_SIZE
pdp_table:                  ; Page Directory Pointer Table
    resb PAGE_TABLE_SIZE
pd_table:                   ; Page-Directory Table
    resb PAGE_TABLE_SIZE
page_table:                 ; Page Table
    resb PAGE_TABLE_SIZE
stack_end:
    resb 64
stack_top:

section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64 ; new
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data: equ $ - gdt64 ; new
    dq (1<<44) | (1<<47) | (1<<41) ; data segment
.ptr:
    dw $ - gdt64 - 1
    dq gdt64
