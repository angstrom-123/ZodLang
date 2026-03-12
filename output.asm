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
func_strlen:
    push rbp
    mov rbp, rsp
; Args
    push rdi
; Body
; Var Decl
    sub rsp, 8
; Assign
; Var
    mov rax, [rbp -8]
    push rax
; Literal Int
    push 0
; BinOp Subscript
    pop rbx
    pop rax
    mov rcx, 1
    imul rbx, rcx
    add rax, rbx
    mov rax, [rax]
    push rax
    pop rax
    mov [rbp -16], al
; Var Decl
    sub rsp, 8
; Assign
; Literal Int
    push 1
    pop rax
    mov [rbp -24], rax
; While
.while_4_5:
; Var
    mov al, [rbp -16]
    push rax
; Literal Char
    push 0
; BinOp NEq
    pop rbx
    pop rax
    cmp rax, rbx
    mov rax, 0
    setne al
    push rax
    pop rax
    cmp rax, 0
    je .end_4_5
; Assign
; Var
    mov rax, [rbp -8]
    push rax
; Var
    mov rax, [rbp -24]
    push rax
; BinOp Subscript
    pop rbx
    pop rax
    mov rcx, 1
    imul rbx, rcx
    add rax, rbx
    mov rax, [rax]
    push rax
    pop rax
    mov [rbp -16], al
; Assign
; Var
    mov rax, [rbp -24]
    push rax
; Literal Int
    push 1
; BinOp Plus
    pop rax
    pop rbx
    add rax, rbx
    push rax
    pop rax
    mov [rbp -24], rax
.post_4_5:
    jmp .while_4_5
.break_4_5:
.end_4_5:
; Var
    mov rax, [rbp -24]
    push rax
; Literal Int
    push 1
; BinOp Minus
    pop rbx
    pop rax
    sub rax, rbx
    push rax
; Return
    pop rax
    jmp .epilogue
.epilogue:
    mov rsp, rbp
    pop rbp
    ret
; Function Declaration
func_puts:
    push rbp
    mov rbp, rsp
; Args
    push rdi
; Body
; Var Decl
    sub rsp, 8
; Assign
; Var
    mov rax, [rbp -8]
    push rax
; Function Call
    pop rdi
    call func_strlen
    push rax
; Var
    mov rax, [rbp -8]
    push rax
; Literal Int
    push 1
; Literal Int
    push 1
; Syscall
    pop rax
    pop rdi
    pop rsi
    pop rdx
    syscall
    push rax
    pop rax
    mov [rbp -16], rax
; Var
    mov rax, [rbp -16]
    push rax
; Return
    pop rax
    jmp .epilogue
.epilogue:
    mov rsp, rbp
    pop rbp
    ret
; Function Declaration
func_main:
    push rbp
    mov rbp, rsp
; Args
; Body
; Literal String
    push str_0
; Function Call
    pop rdi
    call func_puts
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
segment .data
str_0: db 0x48,0x65,0x6C,0x6C,0x6F,0x2C,0x20,0x57,0x6F,0x72,0x6C,0x64,0x21,0x0A,0x00
