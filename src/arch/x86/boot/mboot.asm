bits 32

%define MB2_MAGIC 0e85250d6h
%define MB2_CKSUM(arch) (100000000h - (MB2_MAGIC + (arch) + header_length))

struc mb2_header
.magic  resd    1   ;; magic number
.arch   resd    1   ;; architecture (0 = i386, protected)
.length resd    1   ;; size of the header
.cksum  resd    1   ;; cksum
endstruc

struc mb2_tag
.type   resw    1   ;; type
.flags  resw    1   ;; flags
.size   resd    1   ;; size
endstruc

section .boot
header_start:
align   4, db 0
istruc  mb2_header
    at  mb2_header.magic,   dd  MB2_MAGIC
    at  mb2_header.arch,    dd  0h
    at  mb2_header.length,  dd  header_length
    at  mb2_header.cksum,   dd  MB2_CKSUM(0h)
iend

align   8, db 0
istruc  mb2_tag
    at  mb2_tag.type,       dw  0
    at  mb2_tag.flags,      dw  0
    at  mb2_tag.size,       dd  8
iend
header_end:
header_length   equ     (header_end - header_start)
