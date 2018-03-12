bits 32

;; paging.asm
extern setup_paging
extern unmap_id
;; gdt.asm
extern setup_gdt

;; lib.rs
extern kmain

extern kernel_end
extern kernel_start

section .text
global _start:function (_start.end - _start)
global _start.higher_half
_start:
        cli
        ;; this maps the first virtual 4MiB to physical first 4MiB
        ;; and maps the last pde to the pde itself
        jmp     setup_paging

.higher_half:
        ;; setup a 16kiB stack
        mov     esp, stack_end

        mov     ebp, 0
        push    ebp

        push    ebx ; mbi addr
        push    eax ; mb2 magic

        call    unmap_id

        ;; sets up gdt
        call    setup_gdt

        sti

        pop     eax
        pop     ebx
        add     ebx, 0xe0000000 ; higher half offset

        push    kernel_end
        push    kernel_start
        push    ebx ; mbi addr
        push    eax ; mb2 magic

        call    kmain
        add     esp, 16

        cli
.hang:  hlt
        jmp     .hang

        pop     ebp
.end:

section .bss
align 16, resb 0
stack_bottom:
            resb    16384
stack_end:
