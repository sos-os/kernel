%define PAGE_TABLE_SIZE 512 * 8
%define PAGE_SIZE 4096

; page_map: macro to map the first entry in the first argument to the second
%macro  page_map 2

        mov     eax, %2
        or      eax, 0b11 ; present + writable
        mov     [%1], eax

%endmacro

%macro  export  1

        global %1
        %1:

%endmacro


global start

extern kernel_start

section .text
bits 32

; == start the kernel ========================================================
; this is the beginning of our boot process called by GRUB.
start:
    ; 0. Move the stack pointer to the top of the stack. ---------------------
    mov     esp, stack_top

    ; 1. Move Multiboot info pointer to edi ----------------------------------
    mov     edi, ebx

    ; 2. Make sure that the system supports SOS. -----------------------------
    call    is_multiboot ; check that multiboot is supported
    call    is_cpuid     ; check CPUID is supported (to check for long mode)
    call    is_long_mode ; check if long mode (64-bit) is available.

    ; 3. if everything is okay, create the page tables and start long mode
    call    create_page_tables
    call    set_long_mode

    ; 4. load the 64-bit GDT
    lgdt    [gdt64.ptr]

    ; 5. update selectors
    mov     ax, 16
    mov     ss, ax  ; stack selector
    mov     ds, ax  ; data selector
    mov     es, ax  ; extra selector

    ; 6. print `OK` to screen and jump to the 64-bit boot subroutine.
    mov     dword [0xb8000], 0x2f4b2f4f

    jmp     gdt64.code:kernel_start

; == Tests whether or not multiboot is enabled ==============================
is_multiboot:
    cmp     eax, 0x36d76289
    jne     .no_multiboot
    ret
.no_multiboot:
    mov     al, "0"
    jmp     err


; == Tests whether or not long mode is available ==============================
; If long mode is not available, die (we are a long mode OS).
is_long_mode:
    mov     eax, 0x80000000   ; Set the A-register to 0x80000000.
    cpuid                     ; CPU identification.
    cmp     eax, 0x80000001   ; Compare the A-register with 0x80000001.
    jb      .no_long_mode     ; It is less, there is no long mode.
    mov     eax, 0x80000001   ; Set the A-register to 0x80000001.
    cpuid                     ; Do the CPUID thing once more.
    test    edx, 1 << 29      ; Test if the LM-bit, (bit 29), is set in edx.
    jz      .no_long_mode     ; If it isn't, there is no long mode,
    ret                       ; and we are left with only the void for company.
.no_long_mode:
    mov     al, "2"
    jmp     err

; == Tests wether or not CPUID is available ==================================
; If the system does not support CPUID, we cannot boot, since we need to use
; CPUID to check if we can switch to 64-bit long mode
is_cpuid:
    pushfd                  ; Store the FLAGS-register.
    pop     eax             ; Restore the A-register.
    mov     ecx, eax        ; Set the C-register to the A-register.
    xor     eax, 1 << 21    ; Flip the ID-bit, which is bit 21.
    push    eax             ; Store the A-register.
    popfd                   ; Restore the FLAGS-register.
    pushfd                  ; Store the FLAGS-register.
    pop     eax             ; Restore the A-register.
    push    ecx             ; Store the C-register.
    popfd                   ; Restore the FLAGS-register.
    xor     eax, ecx        ; Do a XOR  A-register andC-register.
    jz      .no_cpuid       ; The zero flag is set, no CPUID.
    ret                     ; CPUID is available for use.
.no_cpuid:
    mov al, "1"
    jmp err

; == Prints a boot error code to the VGA buffer ==============================
err:
    mov     dword [0xb8000], 0x4f524f45
    mov     byte  [0xb8004], al
    hlt

; == Creates the page tables =================================================
; Map the following:
;   - the first PML4 entry -> PDP
;   - the first PDP entry -> PD
;   - each PD entry to its own 2mB page
create_page_tables:
    ; recursive map last entry in PML4 ---------------------------------------
    mov         eax, pml4_table
    or          eax, 0b11
    mov         [pml4_table + 511 * 8], eax

    page_map    pml4_table, pdp_table   ; map first PML4 entry to PDP table
    page_map    pdp_table,  pd_table    ; map first PDP entry to PD table

    ; map each PD table entry to its own 2mB page
    mov         ecx, 0

.pd_table_map: ; maps the PD table -----------------------------------------
    mov     eax, 0x200000   ; 2 mB
    mul     ecx             ; times the start address of the page
    or      eax, 0b10000011 ; check if present + writable + huge

    mov     [pd_table + ecx * 8], eax ; map nth entry from pd -> own page

    ; increment counter and check if done
    inc     ecx
    cmp     ecx, 512
    jne     .pd_table_map

    ret

; == Sets long mode and enables paging =======================================
; In order to do this, we must first create the initial page tables.
set_long_mode:

    ; load PML4 addr to cr3 register -----------------------------------------
    mov     eax, pml4_table
    mov     cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension) --------------------
    mov     eax, cr4
    or      eax, 1 << 5
    mov     cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register) --------
    mov     ecx, 0xC0000080
    rdmsr
    or      eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register -------------------------------------
    mov     eax, cr0
    or      eax, 1 << 31
    or      eax, 1 << 16
    mov     cr0, eax

    ret

section .bss
align 4096
; == page tables =============================================================
; Page-Map Level-4 Table
pml4_table:
    resb    PAGE_TABLE_SIZE
; Page Directory Pointer Table
pdp_table:
    resb    PAGE_TABLE_SIZE
; Page-Directory Table
pd_table:
    resb    PAGE_TABLE_SIZE
 ; Page Table
page_table:
    resb    PAGE_TABLE_SIZE

; == kernel stack =============================================================
stack_base:
    resb    PAGE_SIZE * 2 ; reserve 2 pages for the kernel stack
    ; for some unspeakable reason, doubling the kernel stack size
    ; magically fixes all of the memory allocator bugs? i suspect
    ; the Malloc Gods interpret the extra stack space as a
    ; sacrifice. my mind grows weary of this treatchery.
    ; 𐅃 𐅐𐆂𐅛𐅜𐅀𐅂𐅲𐅯𐅊𐅭𐅙 𐅗 𐅏 𐅽𐅆 𐅲𐆇𐅿𐅚𐆁𐅐𐅶𐅬𐅯𐅴𐅮𐅼 𐅊𐅦 𐅒𐅉 𐅻𐅷𐅘 𐅊𐅗 𐅤𐆁𐅛𐅒𐅎𐅅𐅨𐅓𐅵𐅯𐅺𐅐𐆀
    ; 𐅵𐅿 𐅘 𐅈𐅘𐅁 𐅫 𐅟𐅸 𐅥𐅣𐅑𐅼𐅷𐅻𐆁 𐆊 𐆉𐆇𐆅𐅐 𐅦𐅕 𐅢𐅷𐅗𐅤𐅧 𐅣𐅖𐅺 𐅁𐅿𐅩𐅣 𐅥 𐆄𐅱𐅕 𐅈 𐅙𐅀 𐅋
    ; 𐅩𐅿𐅋𐅫𐅌𐆆𐅊𐆇 𐅜𐅦𐅲 𐅷 𐅱𐆁𐅓𐅞
stack_top:

; == kernel heap =============================================================
heap_base:
    resb    4 * 1024 * 1024 ; reserved space for the kernel heap
heap_top:

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

; GDT offset location exported to Rust kernel
export gdt64_offset
    dq gdt64.code

export stack_base_addr
    dq stack_base

export stack_top_addr
    dq stack_top

export heap_base_addr
    dq heap_base

export heap_top_addr
    dq heap_top
