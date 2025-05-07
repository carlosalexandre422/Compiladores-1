section .bss
section .text
global _start

fib:
push rbp
mov rbp, rsp
sub rsp, 8
mov rax, 0
mov [rbp-8], rax
mov rax, 2
push rax
mov rax, [rbp+16]
pop rbx
xor rcx, rcx
cmp rax, rbx
setl cl
mov rax, rcx
cmp rax, 0
je Lfalso0
mov rax, 1
mov [rbp-8], rax
jmp Lfim1
Lfalso0:
mov rax, 2
push rax
mov rax, [rbp+16]
pop rbx
sub rax, rbx
push rax
call fib
add rsp, 8
push rax
mov rax, 1
push rax
mov rax, [rbp+16]
pop rbx
sub rax, rbx
push rax
call fib
add rsp, 8
pop rbx
add rax, rbx
mov [rbp-8], rax
Lfim1:
mov rax, [rbp-8]
add rsp, 8
pop rbp
ret

fatorial:
push rbp
mov rbp, rsp
sub rsp, 8
mov rax, 1
mov [rbp-8], rax
Linicio2:
mov rax, 1
push rax
mov rax, [rbp+16]
pop rbx
xor rcx, rcx
cmp rax, rbx
setg cl
mov rax, rcx
cmp rax, 0
je Lfim3
mov rax, [rbp+16]
push rax
mov rax, [rbp-8]
pop rbx
imul rax, rbx
mov [rbp-8], rax
mov rax, 1
push rax
mov rax, [rbp+16]
pop rbx
sub rax, rbx
mov [rbp+16], rax
jmp Linicio2
Lfim3:
mov rax, [rbp-8]
add rsp, 8
pop rbp
ret

_start:
mov rax, 4
push rax
call fatorial
add rsp, 8
push rax
mov rax, 5
push rax
call fib
add rsp, 8
pop rbx
add rax, rbx
mov rdi, rax
mov rax, 60
syscall
