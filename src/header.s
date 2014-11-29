.comm memory, 1024
.comm ptr 8, 8

.text

read:
    movq    $0, %rax
    movq    $0, %rdi
    movq    (ptr), %rsi
    movq    $1, %rdx
    syscall
    ret

write:
    movq    $1, %rax
    movq    $1, %rdi
    movq    (ptr), %rsi
    movq    $1, %rdx
    syscall
    ret

.global _start

_start:
    movq    $memory, (ptr)
