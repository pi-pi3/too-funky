bits 32

%define phys(x) (x - 0xe0000000)

section .text
global setup_paging:function (setup_paging.end - setup_paging)
setup_paging:
        ;; map last page table to the page dir itself
        mov     edx, phys(page_dir)
        or      edx, 11b ; r/w | present
        mov     [phys(page_dir) + 1023 * 4], edx

        ;; identity map first 4MiB
        mov     edx, phys(id_table)
        or      edx, 11b ; r/w | present
        mov     [phys(page_dir)], edx

        ;; 1024 entries, each pointing to a 4kiB page
        ;; makes up 4MiB
        mov     ecx, 0
.idloop:
        ;; edx = ecx * 4kiB
        mov     edx, ecx
        ;mul     1000h
        shl     edx, 12
        or      edx, 11b ; r/w | present
        mov     [ecx * 4 + phys(id_table)], edx
        inc     ecx
        cmp     ecx, 1024
        jne     .idloop

        ;; map first 4MiB of kernel to 0xe0000000
        mov     edx, phys(kp_table)
        or      edx, 11b ; r/w | present
        mov     [phys(page_dir) + 0x380 * 4], edx

        ;; 1024 entries, each pointing to a 4kiB page
        ;; makes up 4MiB
        mov     ecx, 0
.kploop:
        ;; edx = ecx * 4kiB
        mov     edx, ecx
        ;mul     1000h
        shl     edx, 12
        or      edx, 11b ; r/w | present
        mov     [ecx * 4 + phys(kp_table)], edx
        inc     ecx
        cmp     ecx, 1024
        jne     .kploop

        ;; physical page address of base directory
        mov     edx, phys(page_dir)
        mov     cr3, edx

        ;; enable paging && write protect
        mov     edx, cr0
        ;; pg | wp
        or      edx, (1 << 31) | (1 << 16)
        mov     cr0, edx

extern _start.higher_half
        lea     edx, [_start.higher_half]
        jmp     edx
        ;ret
.end:

global unmap_id:function (unmap_id.end - unmap_id)
unmap_id:
        ;; unmap the first 4MiB identity
        mov     dword [phys(page_dir)], 0
        invlpg  [0]
        ret
.end:

section .bss
align 4096, resb 0
page_dir    resd    1024
id_table    resd    1024
kp_table    resd    1024
