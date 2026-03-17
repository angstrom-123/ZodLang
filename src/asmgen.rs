pub mod nasm_x86_64 {
    use std::fs;
    use std::io::Write;

    use crate::represent::{IR, InstrKind, Operand, Register};

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
                InstrKind::Culled => {},
                InstrKind::DataSegment => {
                    writeln!(f, "segment .data")?;
                },
                InstrKind::DeclStr => {
                    writeln!(f, "str_{}: db {}", instr.opera, instr.operb)?;
                },
                InstrKind::CopyVarToReg => {
                    writeln!(f, "    mov {}, [rbp {}]", instr.opera, instr.operb)?;
                },
                InstrKind::Push => {
                    if let Operand::LiteralInt { value: _ } = instr.opera {
                        writeln!(f, "    mov rax, {}", instr.opera)?;
                        writeln!(f, "    push rax")?;
                    } else {
                        writeln!(f, "    push {}", instr.opera)?;
                    }
                },
                InstrKind::Pop => {
                    writeln!(f, "    pop {}", instr.opera)?;
                },
                InstrKind::Shl => {
                    if !operand_is_n(&instr.operb, 0) {
                        writeln!(f, "    mov cl, {}", instr.operb.byte())?;
                        writeln!(f, "    shl {}, cl", instr.opera)?;
                    }
                },
                InstrKind::Shr => {
                    if !operand_is_n(&instr.operb, 0) {
                        writeln!(f, "    mov cl, {}", instr.operb.byte())?;
                        writeln!(f, "    shr {}, cl", instr.opera)?;
                    }
                },
                InstrKind::BOr => {
                    writeln!(f, "    or {}, {}", instr.opera, instr.operb)?;
                },
                InstrKind::BAnd => {
                    writeln!(f, "    and {}, {}", instr.opera, instr.operb)?;
                },
                InstrKind::Add => {
                    if !operand_is_n(&instr.operb, 0) {
                        writeln!(f, "    add {}, {}", instr.opera, instr.operb)?;
                    }
                },
                InstrKind::Mul => {
                    if !operand_is_n(&instr.operb, 1) {
                        writeln!(f, "    imul {}, {}", instr.opera, instr.operb)?;
                    }
                },
                InstrKind::DivAByBManglingD => {
                    writeln!(f, "    xor rdx, rdx")?; 
                    if let Operand::Register { name, size: _ } = instr.opera && name != Register::RAX {
                        writeln!(f, "    mov rax, {}", instr.opera)?;
                    }
                    writeln!(f, "    idiv {}", instr.operb)?;
                },
                InstrKind::Sub => {
                    if !operand_is_n(&instr.operb, 0) {
                        writeln!(f, "    sub {}, {}", instr.opera, instr.operb)?;
                    }
                },
                InstrKind::CopyToRegA => {
                    writeln!(f, "    mov {}, {}", instr.opera, instr.operb)?;
                },
                InstrKind::CopyRegToVar => {
                    writeln!(f, "    mov [rbp {}], {}", instr.opera, instr.operb)?;
                },
                InstrKind::Label => {
                    writeln!(f, "{}:", instr.opera)?;
                },
                InstrKind::Return => {
                    writeln!(f, "    ret")?;
                },
                InstrKind::JZero => {
                    writeln!(f, "    cmp {}, 0", instr.opera)?;
                    writeln!(f, "    je {}", instr.operb)?;
                },
                InstrKind::RegBLtA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setl {}", instr.opera.byte())?;
                },
                InstrKind::RegBLeA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setle {}", instr.opera.byte())?;
                },
                InstrKind::RegBGtA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setg {}", instr.opera.byte())?;
                },
                InstrKind::RegBGeA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setge {}", instr.opera.byte())?;
                },
                InstrKind::SubFromReg => {
                    writeln!(f, "    sub {}, {}", instr.operb, instr.opera)?;
                },
                InstrKind::RegBEqA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    sete {}", instr.opera.byte())?;
                },
                InstrKind::RegBNeA => {
                    writeln!(f, "    cmp {}, {}", instr.opera, instr.operb)?;
                    writeln!(f, "    mov {}, 0", instr.opera)?;
                    writeln!(f, "    setne {}", instr.opera.byte())?;
                },
                InstrKind::RegBNeIntLitA => {
                    writeln!(f, "    cmp {}, {}", instr.operb, instr.opera)?;
                    writeln!(f, "    mov {}, 0", instr.operb)?;
                    writeln!(f, "    setne {}", instr.operb.byte())?;
                },
                InstrKind::DeallocateStack => {
                    writeln!(f, "    add rsp, {}", instr.opera)?;
                },
                InstrKind::JLabel => {
                    writeln!(f, "    jmp {}", instr.opera)?;
                },
                InstrKind::Syscall => {
                    writeln!(f, "    syscall")?;
                },
                InstrKind::Call => {
                    writeln!(f, "    call {}", instr.opera)?;
                },
                InstrKind::NegReg => {
                    writeln!(f, "    neg {}", instr.opera)?;
                },
                InstrKind::DerefReg => {
                    writeln!(f, "    mov {}, [{}]", instr.opera, instr.opera)?;
                },
                InstrKind::CopyRegAToAdrAtRegB => {
                    writeln!(f, "    mov [{}], {}", instr.operb, instr.opera)?;
                },
                InstrKind::Comment => {
                    writeln!(f, "; {}", instr.opera)?;
                }
            }
        }

        Ok(())
    }

    fn operand_is_n(o: &Operand, n: i64) -> bool {
        if let Operand::LiteralInt { value } = o && *value == n {
            return true;
        }
        false
    }
}
