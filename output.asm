global _start
section .text
; --- Entrypoint ---
_start:
    call func_main
; --- Debug Dump ---
func_dump:
    sub rsp, 40
    lea rsi, [rsp + 31]
    mov byte [rsp + 31], 10
    mov ecx, 1
    mov r8, -3689348814741910323
.LBB0_1:
    mov rax, rdi
    mul r8
    shr rdx, 3
    lea eax, [rdx + rdx]
    lea eax, [rax + 4*rax]
    mov r9d, edi
    sub r9d, eax
    or r9b, 48
    mov byte [rsi - 1], r9b
    dec rsi
    inc rcx
    cmp rdi, 9
    mov rdi, rdx
    ja .LBB0_1
    mov edi, 1
    mov rdx, rcx
    mov rax, 1
    syscall
    add rsp, 40
    ret
; --- Exit ---
func_exit:
    mov rax, 60
    syscall
; --- MMap ---
func_mmap:
    push rbp
    mov rbp, rsp
    mov rax, 9
    mov rsi, rdi
    xor rdi, rdi
    mov rdx, 3
    mov r10, 34
    mov r8, -1
    xor r9, r9
    syscall
    mov rsp, rbp
    pop rbp
    ret
; --- MUnmap ---
func_munmap:
    mov rax, 11
    syscall
    ret
; Function Declaration
func_main:
    push rbp
    mov rbp, rsp
; Function Call
    call func_test_one
    push rax
; Function Call
    call func_test_two
    push rax
; Literal Int
    push 0
; Function Call
    pop rdi
    call func_exit
.epilogue:
    mov rsp, rbp
    pop rbp
    ret
; Function Declaration
func_test_one:
    push rbp
    mov rbp, rsp
; Var Decl
    sub rsp, 8
; Assign
; Literal Int
    push 4096
; Function Call
    pop rdi
    call func_mmap
    push rax
    pop rax
    mov [rbp  -8], rax
; Assign
; Literal Int
    push 10
    pop rax
    mov rbx, [rbp  -8]
    mov qword [rbx], rax
; Assign
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 8
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
    pop rax
    mov [rbp  -8], rax
; Assign
; Literal Int
    push 20
    pop rax
    mov rbx, [rbp  -8]
    mov qword [rbx], rax
; Assign
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 8
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
    pop rax
    mov [rbp  -8], rax
; Assign
; Literal Int
    push 30
    pop rax
    mov rbx, [rbp  -8]
    mov qword [rbx], rax
; Assign
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 8
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
    pop rax
    mov [rbp  -8], rax
; Assign
; Literal Int
    push 40
    pop rax
    mov rbx, [rbp  -8]
    mov qword [rbx], rax
; Assign
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 24
; BinOp Minus
    pop rbx
    pop rax
    sub rax, rbx
    push rax
    pop rax
    mov [rbp  -8], rax
; Var
    mov rax, [rbp  -8]
    push rax
; UnOp Dereference
    pop rax
    mov rax, [rax]
    push rax
; Function Call
    pop rdi
    call func_dump
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 8
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
; UnOp Dereference
    pop rax
    mov rax, [rax]
    push rax
; Function Call
    pop rdi
    call func_dump
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 16
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
; UnOp Dereference
    pop rax
    mov rax, [rax]
    push rax
; Function Call
    pop rdi
    call func_dump
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 24
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
; UnOp Dereference
    pop rax
    mov rax, [rax]
    push rax
; Function Call
    pop rdi
    call func_dump
; Literal Int
    push 4096
; Var
    mov rax, [rbp  -8]
    push rax
; Function Call
    pop rdi
    pop rsi
    call func_munmap
    push rax
.epilogue:
    mov rsp, rbp
    pop rbp
    ret
; Function Declaration
func_test_two:
    push rbp
    mov rbp, rsp
; Var Decl
    sub rsp, 8
; Assign
; Literal Int
    push 4096
; Function Call
    pop rdi
    call func_mmap
    push rax
    pop rax
    mov [rbp  -8], rax
; Literal Int
    push 1
; Function Call
    pop rdi
    call func_dump
; Assign
; Literal Int
    push 100
    pop rax
    mov rbx, [rbp  -8]
    mov qword [rbx], rax
; Literal Int
    push 2
; Function Call
    pop rdi
    call func_dump
; Assign
; Literal Int
    push 200
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 8
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
    pop rbx
    pop rax
    mov qword [rbx], rax
; Literal Int
    push 3
; Function Call
    pop rdi
    call func_dump
; Var
    mov rax, [rbp  -8]
    push rax
; UnOp Dereference
    pop rax
    mov rax, [rax]
    push rax
; Function Call
    pop rdi
    call func_dump
; Literal Int
    push 4
; Function Call
    pop rdi
    call func_dump
; Var
    mov rax, [rbp  -8]
    push rax
; Literal Int
    push 8
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
; UnOp Dereference
    pop rax
    mov rax, [rax]
    push rax
; Function Call
    pop rdi
    call func_dump
; Literal Int
    push 5
; Function Call
    pop rdi
    call func_dump
; Literal Int
    push 4096
; Var
    mov rax, [rbp  -8]
    push rax
; Function Call
    pop rdi
    pop rsi
    call func_munmap
    push rax
.epilogue:
    mov rsp, rbp
    pop rbp
    ret
