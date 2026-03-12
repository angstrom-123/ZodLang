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
