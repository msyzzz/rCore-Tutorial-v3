    .section .text.entry
    .globl _start
_start:
    mv tp, a0

    add t0, a0, 1
    slli t0, t0, 16
    la sp, boot_stack
    add sp, sp, t0
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 1024 * 64 * 4    # 64 K/core
    .globl boot_stack_top
boot_stack_top: