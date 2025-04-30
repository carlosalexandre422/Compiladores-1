section .bss
a: resq 1
section .text
global _start

_start:
mov $4, %rax
mov [a], %rax
mov $10, %rax
push %rax
mov [a], %rax
pop %rbx
xor %rcx, %rcx
cmp %rax, %rbx
setl %cl
mov %rcx, %rax
cmp $0, %rax
jz Lfalso0
mov $1, %rax
mov [a], %rax
jmp Lfim1
Lfalso0:
mov $0, %rax
mov [a], %rax
Lfim1:
mov [a], %rax
mov %rdi, %rax
mov $60, %rax
syscall
