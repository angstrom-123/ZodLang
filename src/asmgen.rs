pub mod nasm_x86_64 {
    use std::fs;
    use std::io::Write;

    use crate::representation::{IR, InstructionType, Operand};

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
                InstructionType::CopyVarValToRegister => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let Operand::StackOffset { value } = &instr.operb else { unreachable!(); };

                    let op = if *value < 0 { "" } else { "+" };

                    writeln!(f, "    mov {}, [rbp {} {}]", name, op, value)?;
                },
                InstructionType::PushStackRegister => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };

                    writeln!(f, "    push {}", name)?;
                },
                InstructionType::PushStackLiteralInt => {
                    let Operand::LiteralInt { value } = &instr.opera else { unreachable!(); };

                    writeln!(f, "    push {}", value)?;
                },
                InstructionType::PopStack => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };

                    writeln!(f, "    pop {}", name)?;
                },
                InstructionType::AddRegisterBToA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    add {}, {}", namea, nameb)?;
                },
                InstructionType::MulRegisterAByB => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    imul {}, {}", namea, nameb)?;
                },
                InstructionType::DivRCXByRAXManglingRDX => {
                    writeln!(f, "    xor rdx, rdx")?; 
                    writeln!(f, "    idiv rcx")?;
                },
                InstructionType::SubRegisterBFromA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    sub {}, {}", namea, nameb)?;
                },
                InstructionType::CopyRegisterBToA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    mov {}, {}", namea, nameb)?;
                },
                InstructionType::CopyRegisterToVar => {
                    let Operand::StackOffset { value } = &instr.opera else { unreachable!(); };
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };

                    // Negative values already have the `-`
                    let op = if *value < 0 { "" } else { "+" };

                    writeln!(f, "    mov [rbp {} {}], {}", op, value, name)?;
                },
                InstructionType::MakeLabel => {
                    let Operand::Name { name } = &instr.opera else { unreachable!(); };

                    writeln!(f, "{}:", name)?;
                },
                InstructionType::ReturnToCaller => {
                    writeln!(f, "    ret")?;
                },
                InstructionType::JumpIfZero => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Name { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    cmp {}, 0", namea)?;
                    writeln!(f, "    je {}", nameb)?;
                },
                InstructionType::RegisterBLessA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    cmp {}, {}", namea, nameb)?;
                    writeln!(f, "    mov {}, 0", namea)?;
                    writeln!(f, "    setl {}", namea.as_byte())?;
                },
                InstructionType::RegisterBLessEqA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    cmp {}, {}", namea, nameb)?;
                    writeln!(f, "    mov {}, 0", namea)?;
                    writeln!(f, "    setle {}", namea.as_byte())?;
                },
                InstructionType::RegisterBGreaterA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    cmp {}, {}", namea, nameb)?;
                    writeln!(f, "    mov {}, 0", namea)?;
                    writeln!(f, "    setg {}", namea.as_byte())?;
                },
                InstructionType::RegisterBGreaterEqA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    cmp {}, {}", namea, nameb)?;
                    writeln!(f, "    mov {}, 0", namea)?;
                    writeln!(f, "    setge {}", namea.as_byte())?;
                },
                InstructionType::SubLiteralIntFromRegister => {
                    let Operand::LiteralInt { value } = &instr.opera else { unreachable!(); };
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };

                    writeln!(f, "    sub {}, {}", name, value)?;
                },
                InstructionType::RegisterBEqA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    cmp {}, {}", namea, nameb)?;
                    writeln!(f, "    mov {}, 0", namea)?;
                    writeln!(f, "    sete {}", namea.as_byte())?;
                },
                InstructionType::RegisterBNEqA => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    cmp {}, {}", namea, nameb)?;
                    writeln!(f, "    mov {}, 0", namea)?;
                    writeln!(f, "    setne {}", namea.as_byte())?;
                },
                InstructionType::RegisterBNEqLiteralIntA => {
                    let Operand::LiteralInt { value } = &instr.opera else { unreachable!(); };
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };

                    writeln!(f, "    cmp {}, {}", name, value)?;
                    writeln!(f, "    mov {}, 0", name)?;
                    writeln!(f, "    setne {}", name.as_byte())?;
                },
                InstructionType::DeallocateStackBytes => {
                    let Operand::LiteralInt { value } = &instr.opera else { unreachable!(); };

                    writeln!(f, "    add rsp, {}", value)?;
                },
                InstructionType::JumpToLabel => {
                    let Operand::Name { name } = &instr.opera else { unreachable!(); };

                    writeln!(f, "    jmp {}", name)?;
                },
                InstructionType::CallFunction => {
                    let Operand::Name { name } = &instr.opera else { unreachable!(); };

                    writeln!(f, "    call {}", name)?;
                },
                InstructionType::ZeroRegister => {
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };

                    writeln!(f, "    mov {}, 0", name)?;
                },
                InstructionType::CopyLiteralIntToRegister => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let Operand::LiteralInt { value } = &instr.operb else { unreachable!(); };

                    writeln!(f, "    mov {}, {}", name, value)?;
                },
                InstructionType::NegateRegister => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    writeln!(f, "    neg {}", name)?;
                },
                InstructionType::DereferenceRegister => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    writeln!(f, "    mov {}, [{}]", name, name)?;
                },
                InstructionType::CopyRegisterAToAdrAtRegisterB => {
                    let Operand::Register { name } = &instr.opera else { unreachable!(); };
                    let namea = name;
                    let Operand::Register { name } = &instr.operb else { unreachable!(); };
                    let nameb = name;

                    writeln!(f, "    mov qword [{}], {}", nameb, namea)?;
                },
                InstructionType::Comment => {
                    let Operand::Comment { comment } = &instr.opera else { unreachable!(); };

                    writeln!(f, "; {}", comment)?;
                }
            }
        }

        Ok(())
    }
}
