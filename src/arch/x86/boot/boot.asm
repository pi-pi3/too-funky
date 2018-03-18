bits 32

;; lib.rs
extern _rust_start

extern kernel_end
extern kernel_start

section .text
global _start:function (_start.end - _start)
global _start.higher_half
_start:
        cli
        ;; setup a 16kiB stack
        lea     esp, [stack_end - 0xe0000000]

        mov     ebp, 0
        push    ebp

        add     ebx, 0xe0000000 ; higher half offset

        push    kernel_end
        push    kernel_start
        push    ebx ; mbi addr
        ;; skip mb2 magic
        ; push    eax ; mb2 magic

        call    _rust_start
.end:

section .bss
align 16, resb 0
stack_bottom:
            resb    16384
stack_end:
