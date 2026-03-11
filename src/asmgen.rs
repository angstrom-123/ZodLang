pub mod nasm_x86_64 {
    use std::fs;
    use std::io::Write;

    use crate::representation::{IR, InstructionType};

    pub fn generate(out_path: &String, ir: &IR) -> std::io::Result<()> {
        let mut f = fs::File::create(out_path)?;
        // Header
        writeln!(f, "global _start")?;
        writeln!(f, "section .text")?;
        writeln!(f, "; --- Entrypoint ---")?;
        writeln!(f, "_start:")?;
        writeln!(f, "    call func_main")?;

        for instr in &ir.instrs {
            match instr.kind {
                InstructionType::DefineIntrinsicDump => {
                    writeln!(f, "; --- Debug Dump ---")?;
                    writeln!(f, "func_dump:")?;
                    writeln!(f, "    sub rsp, 40")?;
                    writeln!(f, "    lea rsi, [rsp + 31]")?;
                    writeln!(f, "    mov byte [rsp + 31], 10")?;
                    writeln!(f, "    mov ecx, 1")?;
                    writeln!(f, "    mov r8, -3689348814741910323")?;
                    writeln!(f, ".LBB0_1:")?;
                    writeln!(f, "    mov rax, rdi")?;
                    writeln!(f, "    mul r8")?;
                    writeln!(f, "    shr rdx, 3")?;
                    writeln!(f, "    lea eax, [rdx + rdx]")?;
                    writeln!(f, "    lea eax, [rax + 4*rax]")?;
                    writeln!(f, "    mov r9d, edi")?;
                    writeln!(f, "    sub r9d, eax")?;
                    writeln!(f, "    or r9b, 48")?;
                    writeln!(f, "    mov byte [rsi - 1], r9b")?;
                    writeln!(f, "    dec rsi")?;
                    writeln!(f, "    inc rcx")?;
                    writeln!(f, "    cmp rdi, 9")?;
                    writeln!(f, "    mov rdi, rdx")?;
                    writeln!(f, "    ja .LBB0_1")?;
                    writeln!(f, "    mov edi, 1")?;
                    writeln!(f, "    mov rdx, rcx")?;
                    writeln!(f, "    mov rax, 1")?;
                    writeln!(f, "    syscall")?;
                    writeln!(f, "    add rsp, 40")?;
                    writeln!(f, "    ret")?;
                },
                InstructionType::DefineIntrinsicExit => {
                    writeln!(f, "; --- Exit ---")?;
                    writeln!(f, "func_exit:")?;
                    writeln!(f, "    mov rax, 60")?;
                    writeln!(f, "    syscall")?;
                },
                InstructionType::DefineIntrinsicMMap => {
                    writeln!(f, "; --- MMap ---")?;
                    writeln!(f, "func_mmap:")?;
                    writeln!(f, "    push rbp")?;
                    writeln!(f, "    mov rbp, rsp")?;
                    writeln!(f, "    mov rax, 9")?; 
                    writeln!(f, "    mov rsi, rdi")?; 
                    writeln!(f, "    xor rdi, rdi")?; 
                    writeln!(f, "    mov rdx, 3")?; 
                    writeln!(f, "    mov r10, 34")?; 
                    writeln!(f, "    mov r8, -1")?; 
                    writeln!(f, "    xor r9, r9")?;
                    writeln!(f, "    syscall")?; 
                    writeln!(f, "    mov rsp, rbp")?;
                    writeln!(f, "    pop rbp")?;
                    writeln!(f, "    ret")?;
                },
                InstructionType::DefineIntrinsicMUnmap => {
                    writeln!(f, "; --- MUnmap ---")?;
                    writeln!(f, "func_munmap:")?;
                    writeln!(f, "    mov rax, 11")?;
                    writeln!(f, "    syscall")?;
                    writeln!(f, "    ret")?;
                },
                InstructionType::StartDataSegment => {
                    writeln!(f, "segment .data")?;
                },
                InstructionType::DeclareString => {
                    writeln!(f, "str_{}: db {}", instr.opera, instr.operb)?;
                },
                InstructionType::PushStackLiteralString => {
                    writeln!(f, "    push str_{}", instr.opera)?;
                },
                InstructionType::CopyVarValToRegister => {
                    writeln!(f, "    mov {}, [rbp {}]", instr.opera, instr.operb)?;
                },
                InstructionType::PushStackRegister => {
                    writeln!(f, "    push {}", instr.opera)?;
                },
                InstructionType::PushStackLiteralInt => {
                    writeln!(f, "    push {}", instr.opera)?;
                },
                InstructionType::PopStack => {
                    writeln!(f, "    pop {}", instr.opera)?;
                },
                InstructionType::AddRegisterBToA => {
                    writeln!(f, "    add {}, {}", instr.opera, instr.operb)?;
                },
                InstructionType::MulRegisterAByB => {
                    writeln!(f, "    imul {}, {}", instr.opera, instr.operb)?;
                },
                InstructionType::DivAByBManglingD => {
                    writeln!(f, "    xor rdx, rdx")?; 
                    writeln!(f, "    mov rax, {}", instr.opera)?;
                    writeln!(f, "    idiv {}", instr.operb)?;
                },
                InstructionType::SubRegisterBFromA => {
                    writeln!(f, "    sub {}, {}", instr.opera, instr.operb)?;
                },
                InstructionType::CopyRegisterBToA => {
                    writeln!(f, "    mov {}, {}", instr.opera, instr.operb)?;
                },
                InstructionType::CopyRegisterToVar => {
                    writeln!(f, "    mov [rbp {}], {}", instr.opera, instr.operb)?;
                },
                InstructionType::MakeLabel => {
                    writeln!(f, "{}:", instr.opera)?;
                },
                InstructionType::ReturnToCaller => {
                    writeln!(f, "    ret")?;
                },
                InstructionType::JumpIfZero => {
                    writeln!(f, "    cmp {}, 0", instr.opera)?;
                    writeln!(f, "    je {}", instr.operb)?;
                },
                InstructionType::RegisterBLessA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setl {}", instr.opera.byte())?;
                },
                InstructionType::RegisterBLessEqA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setle {}", instr.opera.byte())?;
                },
                InstructionType::RegisterBGreaterA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setg {}", instr.opera.byte())?;
                },
                InstructionType::RegisterBGreaterEqA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setge {}", instr.opera.byte())?;
                },
                InstructionType::SubLiteralIntFromRegister => {
                    writeln!(f, "    sub {}, {}", instr.operb, instr.opera)?;
                },
                InstructionType::RegisterBEqA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    sete {}", instr.opera.byte())?;
                },
                InstructionType::RegisterBNEqA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setne {}", instr.opera.byte())?;
                },
                InstructionType::RegisterBNEqLiteralIntA => {
                    writeln!(f, "    cmp {}, {}", instr.operb, instr.opera)?;
                    writeln!(f, "    mov {}, 0", instr.operb)?;
                    writeln!(f, "    setne {}", instr.operb.byte())?;
                },
                InstructionType::DeallocateStackBytes => {
                    writeln!(f, "    add rsp, {}", instr.opera)?;
                },
                InstructionType::JumpToLabel => {
                    writeln!(f, "    jmp {}", instr.opera)?;
                },
                InstructionType::Syscall => {
                    writeln!(f, "    syscall")?;
                },
                InstructionType::CallFunction => {
                    writeln!(f, "    call {}", instr.opera)?;
                },
                InstructionType::ZeroRegister => {
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                },
                InstructionType::CopyLiteralIntToRegister => {
                    writeln!(f, "    mov {}, {}", instr.opera, instr.operb)?;
                },
                InstructionType::NegateRegister => {
                    writeln!(f, "    neg {}", instr.opera)?;
                },
                InstructionType::DereferenceRegister => {
                    writeln!(f, "    mov {}, [{}]", instr.opera, instr.opera)?;
                },
                InstructionType::CopyRegisterAToAdrAtRegisterB => {
                    writeln!(f, "    mov [{}], {}", instr.operb, instr.opera)?;
                },
                InstructionType::Comment => {
                    writeln!(f, "; {}", instr.opera)?;
                }
            }
        }

        Ok(())
    }
}
