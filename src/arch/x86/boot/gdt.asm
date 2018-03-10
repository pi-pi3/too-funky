bits 32

%define GDT_ENTRY(base, limit, flags, access) \
        (((limit) & 00ffffh) | \
        (((limit) & 0f0000h) << 32) | \
        (((base) & 000fffffh) << 16) | \
        (((base) & 0ff00000h) << 56) | \
        (((flags) & 0fh) << 52) | \
        (((access) & 0ffh) << 40))

section .text
global setup_gdt:function (setup_gdt.end - setup_gdt)
setup_gdt:
        ;; null entry
        mov     dword [gdt.null], 0
        mov     dword [gdt.null + 4], 0
        ;; kernel segments
        mov     edx, GDT_ENTRY(0h, 0fffffh, 1100b, 10011010b) >> 32 ; page granularity | 32bit, present | descr | executable | r/w
        mov     eax, GDT_ENTRY(0h, 0fffffh, 1100b, 10011010b) & 0xffffffff ; page granularity | 32bit, present | descr | executable | r/w
        mov     [gdt.kcode], eax
        mov     [gdt.kcode + 4], edx

        mov     edx, GDT_ENTRY(0h, 0fffffh, 1100b, 10010010b) >> 32 ; page granularity | 32bit, present | descr | r/w
        mov     eax, GDT_ENTRY(0h, 0fffffh, 1100b, 10010010b) & 0xffffffff ; page granularity | 32bit, present | descr | r/w
        mov     [gdt.kdata], eax
        mov     [gdt.kdata + 4], edx

        ;; userspace segments
        mov     edx, GDT_ENTRY(0h, 0dffffh, 1100b, 11111010b) >> 32 ; page granularity | 32bit, present | ring = 3 | descr | executable | r/w
        mov     eax, GDT_ENTRY(0h, 0dffffh, 1100b, 11111010b) & 0xffffffff ; page granularity | 32bit, present | ring = 3 | descr | executable | r/w
        mov     [gdt.ucode], eax
        mov     [gdt.ucode + 4], edx

        mov     edx, GDT_ENTRY(0h, 0dffffh, 1100b, 11110010b) >> 32 ; page granularity | 32bit, present | ring = 3 | descr | r/w
        mov     eax, GDT_ENTRY(0h, 0dffffh, 1100b, 11110010b) & 0xffffffff ; page granularity | 32bit, present | ring = 3 | descr | r/w
        mov     [gdt.udata], eax
        mov     [gdt.udata + 4], edx
        ;; task state segment
        ;; higher half of tss entry
        mov     edx, GDT_ENTRY(0, 0, 0100b, 10011001b) >> 32 ; 32bit, present | executable | accessed

        ;; base 16:23
        mov     eax, tss
        shr     eax, 16
        and     eax, 0xff
        ;; and store in higher byte at 0:7 (32:39)
        or      edx, eax

        ;; base 24:31
        mov     eax, tss
        and     eax, 0xff000000
        ;; and store in higher byte at 24:31 (56:63)
        or      edx, eax

        ;; limit 16:19
        mov     eax, tss.end
        and     eax, 0xf00
        ;; and store in higher byte at 16:19 (48:51)
        or      edx, eax

        ;; base 0:15
        mov     ebx, tss
        shl     ebx, 16
        ;; limit 0:15
        mov     eax, tss.end
        and     eax, 0xff
        or      eax, ebx
        ;; little endian: store low half first
        mov     [gdt.tss], eax
        mov     [gdt.tss + 4], edx

        mov     word [gdt_pointer.size], gdt.size - 1
        mov     dword [gdt_pointer.offset], gdt

        lgdt    [gdt_pointer]

        jmp     8h:.reload_segments

.reload_segments:
        mov     ax, 10h
        mov     ds, ax
        mov     es, ax
        mov     fs, ax
        mov     gs, ax
        mov     ss, ax
        ret
.end:

section .bss
align 16, resb 0
gdt_pointer:
.size       resw    1
.offset     resd    1

align 16, resb 0
gdt:
;; inaccessible null segment
.null       resq    1
;; kernel segments
.kcode      resq    1
.kdata      resq    1
;; userspace segments
.ucode      resq    1
.udata      resq    1
;; task state segment
.tss        resq    1
.size       equ     ($ - gdt)

;; task state segment
tss:
            resw    1 ; reserved
.link:      resw    1 ; LINK
.esp0:      resd    1 ; ESP0
            resw    1 ; reserved
.ss0:       resd    1 ; SS0
.esp1:      resd    1 ; ESP1
            resw    1 ; reserved
.ss1:       resd    1 ; SS1
.esp2:      resd    1 ; ESP2
            resw    1 ; reserved
.ss2:       resd    1 ; SS2
.cr3:       resd    1 ; CR3
.eip:       resd    1 ; EIP
.ef:        resd    1 ; EFLAGS
.eax:       resd    1 ; EAX
.ecx:       resd    1 ; ECX
.edx:       resd    1 ; EDX
.ebx:       resd    1 ; EBX
.esp:       resd    1 ; ESP
.ebp:       resd    1 ; EBP
.esi:       resd    1 ; ESI
.edi:       resd    1 ; EDI
            resw    1 ; reserved
.es:        resd    1 ; ES
            resw    1 ; reserved
.cs:        resd    1 ; CS
            resw    1 ; reserved
.ss:        resd    1 ; SS
            resw    1 ; reserved
.ds:        resd    1 ; DS
            resw    1 ; reserved
.fs:        resd    1 ; FS
            resw    1 ; reserved
.gs:        resd    1 ; GS
            resw    1 ; reserved
.ldtr:      resd    1 ; LDTR
.iopb:      resw    1 ; IOPB offset
            resd    1 ; reserved 
.end:
