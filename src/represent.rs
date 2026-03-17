use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::lex::{Lexer, Pos, Tok, TokKind};
use crate::parse::{NodeKind, Node, AST};
use crate::types::Datatype;

const KERNEL_REG_ORDER: [Register; 7] = [
    Register::RAX,
    Register::RDI,
    Register::RSI,
    Register::RDX,
    Register::R10,
    Register::R9,
    Register::R8,
];

const USER_REG_ORDER: [Register; 6] = [
    Register::RDI,
    Register::RSI,
    Register::RDX,
    Register::RCX,
    Register::R8,
    Register::R9,
];

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum InstrKind {
    Culled,

    Comment,
    DataSegment,

    DeclStr,             // Declare a new string (intended to be in the data segment)
    Syscall,             // Make a syscall
    Push,                // Push a register or int literal or str onto the stack
    Pop,                 // Pop the stack into a register
    Shl,                 // Shifts registers A left by register B bits
    Shr,                 // Shifts register A right by regiter B bits
    BOr,                 // Bitwise or of A and B, result in A
    BAnd,                // Bitwise and of A and B, result in A
    Add,                 // Adds registers A and B, result in A
    Mul,                 // Multiplies registers A by B, result in A
    DivAByBManglingD,    // Divides A by B, mangles D, result in RAX
    Sub,                 // Subtracts registers B from A, result in A
    CopyToRegA,          // Copy value from register B to register A 
    CopyRegToVar,        // Copy value from a register to variable 
    CopyVarToReg,        // Copies the value of a variable to a register
    CopyRegAToAdrAtRegB, // Copies a variable to the adress in the register
    Label,               // Make a new label
    Return,              // Return to caller of function
    JZero,               // Jumps to a label if the operand is 0
    RegBLtA,             // Stores 1 in register A if register B < A, else 0
    RegBLeA,             // Stores 1 in register A if register B <= A, else 0
    RegBGtA,             // Stores 1 in register A if register B > A, else 0
    RegBGeA,             // Stores 1 in register A if register B >= A, else 0
    RegBEqA,             // Stores 1 in register A if register B == A, else 0
    RegBNeA,             // Stores 1 in register A if register B != A, else 0
    RegBNeIntLitA,       // Stores 1 in register A if literal B != A, else 0
    DeallocateStack,     // Deallocates some bytes from the stack
    JLabel,              // Jumps unconditionally to a label
    Call,                // Calls a function by its label
    NegReg,              // Negates the value in a register
    DerefReg,            // Dereferences the registers adress into itself
    SubFromReg,          // Subtracts an immediate value from a register
}
#[derive(Clone, Copy, PartialEq)]
pub enum RegisterSize {
    Byte,
    Word,
    DWord,
    QWord
}
impl fmt::Display for RegisterSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegisterSize::QWord => write!(f, "qword"),
            RegisterSize::DWord => write!(f, "dword"),
            RegisterSize::Word => write!(f, "word"),
            RegisterSize::Byte => write!(f, "byte"),
        }
    }
}
impl RegisterSize {
    pub fn from_i64(size: i64) -> Self {
        match size {
            1 => RegisterSize::Byte,
            2 => RegisterSize::Word,
            4 => RegisterSize::DWord,
            8 => RegisterSize::QWord,
            _ => panic!("Invalid register size {}", size),
        }
    }
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum Register {
    RAX = 0,
    RCX = 1,
    RDX = 2,
    RBX = 3,
    RSP = 4,
    RBP = 5,
    RSI = 6,
    RDI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15
}
impl Register {
    fn size(&self, size: i64) -> &'static str {
        let size: RegisterSize = RegisterSize::from_i64(size);
        match size {
            RegisterSize::Byte => match self {
                Register::RAX => "al",
                Register::RCX => "cl",
                Register::RDX => "dl",
                Register::RBX => "bl",
                Register::RSP => "spl",
                Register::RBP => "bpl",
                Register::RSI => "sil",
                Register::RDI => "dil",
                Register::R8 => "r8b",
                Register::R9 => "r9b",
                Register::R10 => "r10b",
                Register::R11 => "r11b",
                Register::R12 => "r12b",
                Register::R13 => "r13b",
                Register::R14 => "r14b",
                Register::R15 => "r15b",
            },
            RegisterSize::Word => match self {
                Register::RAX => "ax",
                Register::RCX => "cx",
                Register::RDX => "dx",
                Register::RBX => "bx",
                Register::RSP => "sp",
                Register::RBP => "bp",
                Register::RSI => "si",
                Register::RDI => "di",
                Register::R8 => "r8w",
                Register::R9 => "r9w",
                Register::R10 => "r10w",
                Register::R11 => "r11w",
                Register::R12 => "r12w",
                Register::R13 => "r13w",
                Register::R14 => "r14w",
                Register::R15 => "r15w",
            },
            RegisterSize::DWord => match self {
                Register::RAX => "eax",
                Register::RCX => "ecx",
                Register::RDX => "edx",
                Register::RBX => "ebx",
                Register::RSP => "esp",
                Register::RBP => "ebp",
                Register::RSI => "esi",
                Register::RDI => "edi",
                Register::R8 => "r8d",
                Register::R9 => "r9d",
                Register::R10 => "r10d",
                Register::R11 => "r11d",
                Register::R12 => "r12d",
                Register::R13 => "r13d",
                Register::R14 => "r14d",
                Register::R15 => "r15d",
            },
            RegisterSize::QWord => match self {
                Register::RAX => "rax",
                Register::RCX => "rcx",
                Register::RDX => "rdx",
                Register::RBX => "rbx",
                Register::RSP => "rsp",
                Register::RBP => "rbp",
                Register::RSI => "rsi",
                Register::RDI => "rdi",
                Register::R8 => "r8",
                Register::R9 => "r9",
                Register::R10 => "r10",
                Register::R11 => "r11",
                Register::R12 => "r12",
                Register::R13 => "r13",
                Register::R14 => "r14",
                Register::R15 => "r15",
            },
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Operand {
    None,
    Register {
        name: Register,
        size: i64
    },
    StackOffset {
        value: i64,
        size: i64
    },
    LiteralInt {
        value: i64,
    },
    LiteralStr {
        value: i64,
    },
    Name {
        name: Label,
    },
    Bytes {
        bytes: Vec<u8>,
    },
    Comment {
        comment: String,
    },
}
impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::None => write!(f, "None"),
            Operand::Register { name, size } => write!(f, "{}", name.size(*size)),
            Operand::StackOffset { value, size: _ } => write!(f, "{}{}", if *value < 0 { "" } else { "+" }, value),
            Operand::LiteralInt { value } => write!(f, "{}", value),
            Operand::LiteralStr { value } => write!(f, "str_{}", value),
            Operand::Comment { comment } => write!(f, "{}", comment),
            Operand::Name { name } => write!(f, "{}", name),
            Operand::Bytes { bytes } => write!(f, "{}", bytes.iter().map(|b| format!("0x{:02X}", b).to_string()).collect::<Vec<String>>().join(","))
        }
    }
}
impl fmt::Debug for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::None => write!(f, "None"),
            Operand::Register { name: _, size: _ } => write!(f, "Register"),
            Operand::StackOffset { value: _, size: _ } => write!(f, "Stack Offset"),
            Operand::LiteralInt { value: _ } => write!(f, "Literal Int"),
            Operand::LiteralStr { value: _ } => write!(f, "Literal Str"),
            Operand::Bytes { bytes: _ } => write!(f, "Bytes"),
            Operand::Comment { comment: _ } => write!(f, "Comment"),
            Operand::Name { name: _ } => write!(f, "Name"),
        }
    }
}

impl Operand {
    pub fn byte(&self) -> String {
        if let Operand::Register { name, size: _ } = self {
            format!("{}", Operand::Register { name: *name, size: 1 })
        } else { panic!("Cannot convert non-register operands to byte") }
    }

    pub fn size(&self) -> String {
        match self {
            Operand::Register { name: _, size } => format!("{}", RegisterSize::from_i64(*size)),
            Operand::StackOffset { value: _, size } => format!("{}", RegisterSize::from_i64(*size)),
            _ => panic!("Cannot get size of operand `{:?}`", self),
        }
    }
}

#[derive(Clone)]
pub struct Instr {
    pub kind: InstrKind,
    pub opera: Operand,
    pub operb: Operand,
}
impl Instr {
    pub fn dump(&self) {
        eprintln!("{:?}\n    a: {}\n    b: {}", self.kind, self.opera, self.operb);
    }

    fn comment(comment: &'static str) -> Self {
        Instr {
            kind: InstrKind::Comment,
            opera: Operand::Comment { comment: comment.into() },
            operb: Operand::None,
        }
    }

    fn decl_str(index: i64, bytes: Vec<u8>) -> Self {
        Instr {
            kind: InstrKind::DeclStr,
            opera: Operand::LiteralInt { value: index },
            operb: Operand::Bytes { bytes },
        }
    }

    fn push_str_lit(index: i64) -> Self {
        Instr {
            kind: InstrKind::Push,
            opera: Operand::LiteralStr { value: index },
            operb: Operand::None,
        }
    }

    fn data_segment() -> Self {
        Instr {
            kind: InstrKind::DataSegment,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn sub_from_reg(value: i64, name: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::SubFromReg,
            opera: Operand::LiteralInt { value },
            operb: Operand::Register { name, size },
        }
    }


    fn push_reg(name: Register) -> Self {
        Instr {
            kind: InstrKind::Push,
            opera: Operand::Register { name, size: 8 },
            operb: Operand::None,
        }
    }

    fn copy_var_to_reg(name: Register, var: Var, size: i64) -> Self {
        Instr {
            kind: InstrKind::CopyVarToReg,
            opera: Operand::Register { name, size },
            operb: Operand::StackOffset { value: var.offset, size },
        }
    }

    fn neg_reg(name: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::NegReg,
            opera: Operand::Register { name, size },
            operb: Operand::None,
        }
    }

    fn shl(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::Shl,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size }
        }
    }

    fn shr(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::Shr,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size }
        }
    }

    fn bor(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::BOr,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size }
        }
    }

    fn band(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::BAnd,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size }
        }
    }

    fn pop(name: Register) -> Self {
        Instr {
            kind: InstrKind::Pop,
            opera: Operand::Register { name, size: 8 },
            operb: Operand::None,
        }
    }

    fn deref_reg(name: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::DerefReg,
            opera: Operand::Register { name, size },
            operb: Operand::None,
        }
    }

    fn copy_reg_to_var(var: Var, name: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::CopyRegToVar,
            opera: Operand::StackOffset { value: var.offset, size },
            operb: Operand::Register { name, size },
        }
    }

    fn sub_regs(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::Sub,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn add_regs(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::Add,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn mul_regs(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::Mul,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn div_regs_mangle_d(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::DivAByBManglingD,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn copy_int_lit_b_to_a(a: Register, b: i64, size: i64) -> Self {
        Instr {
            kind: InstrKind::CopyToRegA,
            opera: Operand::Register { name: a, size },
            operb: Operand::LiteralInt { value: b },
        }
    }

    fn copy_reg_b_to_a(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::CopyToRegA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn label(l: &Label) -> Self {
        Instr {
            kind: InstrKind::Label,
            opera: Operand::Name { name: l.clone() },
            operb: Operand::None,
        }
    }

    fn returnn() -> Self {
        Instr {
            kind: InstrKind::Return,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn jzero(name: Register, label: &Label, size: i64) -> Self {
        Instr {
            kind: InstrKind::JZero,
            opera: Operand::Register { name, size },
            operb: Operand::Name { name: label.clone() },
        }
    }

    fn jlabel(label: &Label) -> Self {
        Instr {
            kind: InstrKind::JLabel,
            opera: Operand::Name { name: label.clone() },
            operb: Operand::None,
        }
    }

    fn dealloc_stack(bytes: i64) -> Self {
        Instr {
            kind: InstrKind::DeallocateStack,
            opera: Operand::LiteralInt { value: bytes },
            operb: Operand::None,
        }
    }

    fn call(label: &Label) -> Self {
        Instr {
            kind: InstrKind::Call,
            opera: Operand::Name { name: label.clone() },
            operb: Operand::None,
        }
    }

    fn syscall() -> Self {
        Instr {
            kind: InstrKind::Syscall,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn push_int_lit(val: i64) -> Self {
        Instr { 
            kind: InstrKind::Push,
            opera: Operand::LiteralInt { value: val },
            operb: Operand::None,
        }
    }

    fn reg_b_lt_a(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBLtA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn reg_b_le_a(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBLeA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn reg_b_gt_a(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBGtA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn reg_b_ge_a(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBGeA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn reg_b_eq_a(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBEqA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn reg_b_eq_int_lit_a(a: Register, b: i64, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBEqA,
            opera: Operand::Register { name: a, size },
            operb: Operand::LiteralInt { value: b },
        }
    }

    fn reg_b_ne_a(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBNeA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn int_lit_a_ne_reg_b(a: i64, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::RegBNeIntLitA,
            opera: Operand::LiteralInt { value: a },
            operb: Operand::Register { name: b, size },
        }
    }

    fn copy_reg_to_adr_at_reg_b(a: Register, b: Register, size: i64) -> Self {
        Instr {
            kind: InstrKind::CopyRegAToAdrAtRegB,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size: 8 },
        }
    }
}

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
pub struct Var {
    typ: Datatype,
    offset: i64,
}

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
pub struct Label {
    name: Vec<u8>,
    index: usize
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name_str())
    }
}

impl Label {
    pub fn new(prefix: &'static str) -> Self {
        let label: Vec<u8> = Vec::from(prefix.as_bytes());
        Label { name: label, index: usize::MAX }
    }

    pub fn new_named(prefix: &'static str, name: &Vec<u8>) -> Self {
        let mut label: Vec<u8> = Vec::from(prefix.as_bytes());
        label.extend(name);
        Label { name: label, index: usize::MAX }
    }

    pub fn new_at(prefix: &'static str, pos: &Pos) -> Self {
        let mut label: Vec<u8> = Vec::from(prefix.as_bytes());
        label.extend(pos.as_vec());
        Label { name: label, index: usize::MAX }
    }

    pub fn name_str(&self) -> String {
        String::from_utf8(self.name.clone()).expect("Error: Failed to convert label name to string")
    }
}

pub struct Context {
    ident: Tok,
    locals: HashMap<Vec<u8>, Var>,
    stack_ix: i64,
}
impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
impl Context {
    pub fn new() -> Self {
        Context {
            ident: Tok::null(),
            locals: HashMap::new(),
            stack_ix: 0,
        }
    }

    pub fn from(ctx: &Context) -> Self {
        let mut res: Context = Context {
            ident: ctx.ident.clone(),
            locals: HashMap::new(),
            stack_ix: ctx.stack_ix,
        };
        res.locals.extend(ctx.locals.clone());
        res
    }

    pub fn under(ident: &Tok, ctx: &Context) -> Self {
        let mut res: Context = Context {
            ident: ident.clone(),
            locals: HashMap::new(),
            stack_ix: ctx.stack_ix,
        };
        res.locals.extend(ctx.locals.clone());
        res
    }
}

pub struct IR<'a> {
    pub instrs: Vec<Instr>,
    pub cur: usize,
    labels: HashSet<Label>,
    strs: Vec<Vec<u8>>,
    ast: &'a AST,
}
impl<'a> IR<'a> {
    pub fn new(ast: &'a AST) -> Self {
        IR {
            instrs: Vec::new(),
            labels: HashSet::new(),
            strs: Vec::new(),
            cur: 0,
            ast
        }
    }

    pub fn dump(&self) {
        for instr in &self.instrs {
            instr.dump();
        }
    }

    pub fn consume_instr(&mut self) -> Instr  {
        let res: &Instr = self.instrs.get(self.cur).expect("Error: IR failed to consume instruction");
        self.cur += 1;
        res.clone()
    }

    pub fn peek_next_instr(&mut self) -> Instr  {
        self.instrs.get(self.cur + 1).expect("Error: IR failed to peek next token").clone()
    }

    pub fn peek_instr(&mut self) -> Instr  {
        self.instrs.get(self.cur).expect("Error: IR failed to peek token").clone()
    }

    pub fn has_instr(&self) -> bool {
        self.cur < self.instrs.len()
    }

    pub fn generate(&mut self) {
        for stmt in self.ast.root.children.iter() {
            match stmt.kind {
                NodeKind::FuncDecl => {
                    self.instrs.push(Instr::comment("Function Declaration"));

                    let mut top_label: Label = Label::new_named("func_", &stmt.tok.val);
                    let mut bottom_label: Label = Label::new(".epilogue");

                    // Save function label
                    self.record_label(&mut top_label);
                    self.instrs.push(Instr::label(&top_label));

                    // Prologue
                    self.instrs.append(&mut vec![
                        Instr::push_reg(Register::RBP),
                        Instr::copy_reg_b_to_a(Register::RBP, Register::RSP, 8),
                    ]);

                    // Args and context
                    let mut ctx: Context = Context::new();
                    ctx.stack_ix = -8;

                    if let Some(args) = stmt.children.first() {
                        if args.children.len() > 6 {
                            panic!("{} Error: Currently only up to 6 function args are supported", stmt.tok.pos);
                        }

                        self.instrs.push(Instr::comment("Args"));

                        // Save args as local variables on the stack
                        for (i, arg) in args.children.iter().enumerate() {
                            let reg: Register = USER_REG_ORDER[i];
                            self.instrs.push(Instr::push_reg(reg));

                            assert!(!matches!(arg.datatype, Datatype::None | Datatype::Unknown), "Variable type must be known");
                            let var: Var = Var {
                                typ: arg.datatype.clone(),
                                offset: ctx.stack_ix,
                            };
                            ctx.locals.insert(arg.tok.val.clone(), var);
                            ctx.stack_ix -= 8;
                        }
                    }

                    // Body
                    if let Some(body) = stmt.children.get(1) {
                        self.instrs.push(Instr::comment("Body"));
                        for stmt in &body.children {
                            self.generate_from_statement(stmt, &mut ctx);
                        }
                    } else {
                        panic!("{} Error: Expected function body for `{}` but found nothing", stmt.tok.val_str(), stmt.tok.pos);
                    }

                    // Epilogue
                    self.record_label(&mut bottom_label);
                    self.instrs.push(Instr::label(&bottom_label));

                    // On a local stack frame so we automatically deallocate the locals when returning
                    self.instrs.append(&mut vec![
                        Instr::copy_reg_b_to_a(Register::RSP, Register::RBP, 8),
                        Instr::pop(Register::RBP),
                        Instr::returnn(),
                    ]);
                },
                _ => unreachable!()
            }
        }

        self.instrs.push(Instr::data_segment());
        for (i, s) in self.strs.iter().enumerate() {
            self.instrs.push(Instr::decl_str(i as i64, s.clone()));
        }
    }

    // Calling Convention:
    //      Int and Ptr args in:
    //          RDI 
    //          RSI
    //          RDX 
    //          RCX 
    //          R10
    //          R9
    //          R8 
    //      Return value in:
    //          RAX 
    fn generate_syscall(&mut self, call: &Node, _ctx: &mut Context) {
        if call.children.len() > 7 {
            panic!("{} Error: Passing to many args to syscall. Maximum is 7", call.tok.pos);
        }

        self.instrs.push(Instr::comment("Syscall"));

        // Calling convention puts first few args into registers
        for (i, reg) in KERNEL_REG_ORDER.iter().enumerate() {
            if i == call.children.len() {
                break;
            }
            self.instrs.push(Instr::pop(*reg));
        }

        self.instrs.append(&mut vec![
            Instr::syscall(),
            Instr::push_reg(Register::RAX)
        ]);
    }

    // Calling Convention:
    //      Int and Ptr args in:
    //          RDI 
    //          RSI
    //          RDX 
    //          RCX 
    //          R8 
    //          R9
    //      Excess args on stack
    //      Return value in:
    //          RAX 
    //          RAX + RDX (128 bit)
    fn generate_func_call(&mut self, call: &Node, _ctx: &mut Context) {
        if call.children.len() > 6 {
            panic!("{} Error: Passing to many args to function `{}`. Currently there is support for up to 6 args",
                   call.tok.pos, call.tok.val_str());
        }

        self.instrs.push(Instr::comment("Function Call"));

        // Calling convention puts first few args into registers
        for (i, reg) in USER_REG_ORDER.iter().enumerate() {
            if i == call.children.len() {
                break;
            }
            self.instrs.push(Instr::pop(*reg));
        }

        let label: Label = Label::new_named("func_", &call.tok.val);
        self.instrs.push(Instr::call(&label));

        if call.datatype != Datatype::Void {
            self.instrs.push(Instr::push_reg(Register::RAX));
        }
    }

    fn generate_from_statement(&mut self, stmt: &Node, ctx: &mut Context) {
        match stmt.kind {
            NodeKind::VarDecl
            | NodeKind::Continue
            | NodeKind::Break
            | NodeKind::Return
            | NodeKind::Syscall
            | NodeKind::FuncCall => {
                for node in &stmt.post_order() {
                    self.generate_from_node(node, ctx);
                }
            },
            NodeKind::Assign => {
                self.instrs.push(Instr::comment("Assign"));

                // Evaluate RHS first
                if let Some(rhs) = stmt.children.get(1) {
                    for node in &rhs.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                } else {
                    panic!("{} Error: Failed to get rhs of assignment", stmt.tok.pos);
                }

                if let Some(lhs_group) = stmt.children.first() {
                    if lhs_group.kind == NodeKind::UnaryOp && lhs_group.tok.kind == TokKind::Deref {
                        let lhs: &Node = lhs_group.children.first().unwrap();
                        if lhs.kind == NodeKind::Var { // Dereferencing a var
                            if let Some(var) = ctx.locals.get(&lhs.tok.val) {
                                self.instrs.append(&mut vec![
                                    Instr::pop(Register::RAX),
                                    Instr::copy_var_to_reg(Register::RBX, var.clone(), var.typ.size()), // address into rbx
                                    Instr::copy_reg_to_adr_at_reg_b(Register::RAX, Register::RBX, var.typ.size()),
                                ]);
                            } else {
                                panic!("{} Error: Attempting to assign undeclared variable `{}`", stmt.tok.pos, lhs.tok.val_str());
                            }
                        } else { // Dereferencing some expression
                            for node in &lhs.post_order() {
                                self.generate_from_node(node, ctx);
                            }
                            self.instrs.append(&mut vec![
                                Instr::pop(Register::RBX), // lhs
                                Instr::pop(Register::RAX), // rhs
                                Instr::copy_reg_to_adr_at_reg_b(Register::RAX, Register::RBX, lhs.datatype.size()),
                            ]);
                        }
                    } else if lhs_group.kind == NodeKind::Var {
                        if let Some(var) = ctx.locals.get(&lhs_group.tok.val) {
                            self.instrs.append(&mut vec![
                                Instr::pop(Register::RAX),
                                Instr::copy_reg_to_var(var.clone(), Register::RAX, var.typ.size())
                            ]);
                        } else {
                            panic!("{} Error: Attempting to assign undeclared variable `{}`", stmt.tok.pos, lhs_group.tok.val_str());
                        }
                    } else if lhs_group.kind == NodeKind::BinaryOp && lhs_group.tok.kind == TokKind::Subscript {
                        let lhs: &Node = lhs_group.children.first().unwrap();
                        let rhs: &Node = lhs_group.children.last().unwrap();
                        // self.generate_from_node(rhs, ctx);
                        for node in &rhs.post_order() {
                            self.generate_from_node(node, ctx);
                        }
                        if let Some(var) = ctx.locals.get(&lhs.tok.val) {
                            let size: i64 = var.typ.base_type().unwrap().size();
                            self.instrs.append(&mut vec![
                                Instr::pop(Register::RBX), // index
                                Instr::pop(Register::RAX), // value
                                Instr::copy_int_lit_b_to_a(Register::RCX, size, 8), // sizeof type
                                Instr::mul_regs(Register::RBX, Register::RCX, 8), // offset
                                Instr::copy_var_to_reg(Register::RDX, var.clone(), 8), // address into rcx
                                Instr::add_regs(Register::RDX, Register::RBX, 8), // add offset to addy
                                Instr::copy_reg_to_adr_at_reg_b(Register::RAX, Register::RDX, size), // assign
                            ]);
                        } else {
                            panic!("{} Error: Attempting to assign undeclared variable `{}`", stmt.tok.pos, lhs.tok.val_str());
                        }
                    } else {
                        panic!("{} Error: Unexpected lhs in assignment `{}`", lhs_group.tok.pos, lhs_group.tok.val_str());
                    }
                } else {
                    panic!("{} Error: Failed to get lhs of assignment", stmt.tok.pos);
                }
            },
            NodeKind::WhileLoop => {
                self.instrs.push(Instr::comment("While"));
                let mut loop_label: Label = Label::new_at(".while_", &stmt.tok.pos());
                let mut continue_label: Label = Label::new_at(".continue_", &stmt.tok.pos());
                let mut break_label: Label = Label::new_at(".break_", &stmt.tok.pos());
                let mut end_label: Label = Label::new_at(".end_", &stmt.tok.pos());

                // Jump Point
                self.record_label(&mut loop_label);
                self.instrs.push(Instr::label(&loop_label));

                // Condition
                if let Some(cond) = stmt.children.first() {
                    for node in &cond.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                    self.instrs.push(Instr::pop(Register::RAX));
                    self.instrs.push(Instr::jzero(Register::RAX, &end_label, cond.datatype.size()));
                }

                // Body
                let mut while_ctx: Context = Context::under(&stmt.tok, ctx);
                if let Some(body) = stmt.children.get(1) && !body.is_null() {
                    for node in &body.children {
                        self.generate_from_statement(node, &mut while_ctx);
                    }
                }

                // Deallocate body locals
                let new_locals: usize = while_ctx.locals.len() - ctx.locals.len();
                if new_locals > 0 {
                    self.instrs.push(Instr::dealloc_stack(new_locals as i64 * 8));
                }

                self.record_label(&mut continue_label);
                self.instrs.push(Instr::label(&continue_label));
                self.instrs.push(Instr::jlabel(&loop_label));

                // Breaking out could skip deallocating locals, so we clean up here
                self.record_label(&mut break_label);
                self.instrs.push(Instr::label(&break_label));

                // Deallocate body locals
                let new_locals: usize = while_ctx.locals.len() - ctx.locals.len();
                if new_locals > 0 {
                    self.instrs.push(Instr::dealloc_stack(new_locals as i64 * 8));
                }

                self.record_label(&mut end_label);
                self.instrs.push(Instr::label(&end_label));
            }, 
            NodeKind::ForLoop => {
                self.instrs.push(Instr::comment("For"));
                // Labels
                let mut loop_label: Label = Label::new_at(".for_", &stmt.tok.pos());
                let mut continue_label: Label = Label::new_at(".continue_", &stmt.tok.pos());
                let mut break_label: Label = Label::new_at(".break_", &stmt.tok.pos());
                let mut end_label: Label = Label::new_at(".end_", &stmt.tok.pos());

                // Decl 
                let mut decl_tok: Option<&Tok> = None;
                if let Some(decl) = stmt.children.first() && !decl.is_null() {
                    self.instrs.push(Instr::comment("Decl"));
                    decl_tok = Some(&decl.tok);
                    for node in &decl.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                }

                // Init 
                if let Some(init) = stmt.children.get(1) && !init.is_null() {
                    self.instrs.push(Instr::comment("Init"));
                    self.generate_from_statement(init, ctx);
                }

                let mut for_ctx: Context = Context::under(&stmt.tok, ctx);

                // Condition 
                self.record_label(&mut loop_label);
                self.instrs.push(Instr::label(&loop_label));
                if let Some(cond) = stmt.children.get(2) && !cond.is_null() {
                    self.instrs.push(Instr::comment("Condition"));
                    for node in &cond.post_order() {
                        self.generate_from_node(node, ctx);
                    }

                    self.instrs.push(Instr::pop(Register::RAX));
                    self.instrs.push(Instr::jzero(Register::RAX, &end_label, cond.datatype.size()));
                }

                // Body
                if let Some(body) = stmt.children.get(4) && !body.is_null() {
                    self.instrs.push(Instr::comment("Body"));
                    for node in &body.children {
                        self.generate_from_statement(node, &mut for_ctx);
                    }
                }

                // Post 
                self.instrs.push(Instr::comment("Post"));

                let new_locals: usize = for_ctx.locals.len() - ctx.locals.len();
                if new_locals > 0 {
                    self.instrs.push(Instr::dealloc_stack(new_locals as i64 * 8));
                }

                // Continue handles stack dealloc
                self.record_label(&mut continue_label);
                self.instrs.push(Instr::label(&continue_label));

                if let Some(post) = stmt.children.get(3) && !post.is_null() {
                    self.generate_from_statement(post, &mut for_ctx);
                }
                self.instrs.push(Instr::jlabel(&loop_label));

                // Break handles stack dealloc and skips the loop jump
                self.record_label(&mut break_label);
                self.instrs.push(Instr::label(&break_label));

                // Deallocate body locals
                // let new_locals: usize = for_ctx.locals.len() - ctx.locals.len();
                // if new_locals > 0 {
                //     self.instrs.push(Instr::dealloc_stack(new_locals as i64 * 8));
                // }

                self.instrs.push(Instr::comment("End"));
                self.record_label(&mut end_label);
                self.instrs.push(Instr::label(&end_label));

                // If a variable was declared in the for init, we remove it here
                if let Some(tok) = decl_tok {
                    ctx.locals.remove(&tok.val);
                    ctx.stack_ix += 8;
                    self.instrs.push(Instr::dealloc_stack(8));
                }
            },
            NodeKind::Conditional => {
                self.instrs.push(Instr::comment("Conditional"));
                let mut end_label: Label = Label::new_at(".end_", &stmt.tok.pos());

                // Condition 
                let cond: &Node = stmt.children.first().unwrap();
                for node in &cond.post_order() {
                    self.generate_from_node(node, ctx);
                }
                self.instrs.push(Instr::pop(Register::RAX));

                if stmt.children.len() == 2 { // There is no else block
                    // Jump to end if condition fails
                    self.instrs.push(Instr::jzero(Register::RAX, &end_label, cond.datatype.size()));

                    // If body
                    self.instrs.push(Instr::comment("If"));
                    let mut if_ctx: Context = Context::from(ctx);
                    if let Some(if_block) = stmt.children.get(1) {
                        for sub_stmt in &if_block.children {
                            self.generate_from_statement(sub_stmt, &mut if_ctx);
                        }
                    }

                    // Deallocate if body locals
                    let new_locals: usize = if_ctx.locals.len() - ctx.locals.len();
                    if new_locals > 0 {
                        self.instrs.push(Instr::dealloc_stack(new_locals as i64 * 8));
                    }

                    // End label
                    self.record_label(&mut end_label);
                    self.instrs.push(Instr::label(&end_label));
                } else { // There is an else block
                    // Jump to else if condition fails
                    let mut else_label: Label = Label::new_at(".else_", &stmt.tok.pos());

                    self.instrs.push(Instr::jzero(Register::RAX, &else_label, cond.datatype.size()));

                    // If block
                    self.instrs.push(Instr::comment("If"));
                    let mut if_ctx: Context = Context::from(ctx);
                    if let Some(if_block) = stmt.children.get(1) {
                        for sub_stmt in &if_block.children {
                            self.generate_from_statement(sub_stmt, &mut if_ctx);
                        }
                    }

                    // Skip over `else` block if the `if` was hit
                    self.instrs.push(Instr::jlabel(&end_label));

                    // Else label to jump to if condition is false
                    self.record_label(&mut else_label);
                    self.instrs.push(Instr::label(&else_label));

                    // Else block
                    self.instrs.push(Instr::comment("Else"));
                    let mut else_ctx: Context = Context::from(ctx);
                    if let Some(else_block) = stmt.children.get(2) {
                        for sub_stmt in &else_block.children {
                            self.generate_from_statement(sub_stmt, &mut else_ctx);
                        }
                    }
                    
                    // End label
                    self.record_label(&mut end_label);
                    self.instrs.push(Instr::label(&end_label));
                }
            },
            NodeKind::Null => {},
            _ => panic!("{} Error: Expected statement but got {:?}:{:?} `{}`", stmt.tok.pos, stmt.kind, stmt.tok.kind, stmt.tok.val_str())
        }
    }

    fn generate_from_node(&mut self, node: &Node, ctx: &mut Context) {
        match node.kind {
            NodeKind::BinaryOp => {
                match node.tok.kind {
                    TokKind::Mod => {
                        self.instrs.push(Instr::comment("BinOp Mod"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::div_regs_mangle_d(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RDX),
                        ]);
                    },
                    TokKind::Plus => {
                        self.instrs.push(Instr::comment("BinOp Plus"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RAX),
                            Instr::pop(Register::RBX),
                            Instr::add_regs(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Minus => {
                        self.instrs.push(Instr::comment("BinOp Minus"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::sub_regs(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Mul => {
                        self.instrs.push(Instr::comment("BinOp Mul"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RAX),
                            Instr::pop(Register::RBX),
                            Instr::mul_regs(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Div => {
                        self.instrs.push(Instr::comment("BinOp Div"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::div_regs_mangle_d(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Equal => {
                        self.instrs.push(Instr::comment("BinOp Eq"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::reg_b_eq_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::NotEqual => {
                        self.instrs.push(Instr::comment("BinOp NEq"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::reg_b_ne_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::GT => {
                        self.instrs.push(Instr::comment("BinOp GT"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::reg_b_gt_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::LT => {
                        self.instrs.push(Instr::comment("BinOp LT"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::reg_b_lt_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::GE => {
                        self.instrs.push(Instr::comment("BinOp GE"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::reg_b_ge_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::LE => {
                        self.instrs.push(Instr::comment("BinOp LE"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::reg_b_le_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Shl => {
                        self.instrs.push(Instr::comment("BinOp Shl"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::shl(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Shr => {
                        self.instrs.push(Instr::comment("BinOp Shr"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::shr(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::BitOr => {
                        self.instrs.push(Instr::comment("BinOp Bitwise Or"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::bor(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::BitAnd => {
                        self.instrs.push(Instr::comment("BinOp Bitwise And"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RBX),
                            Instr::pop(Register::RAX),
                            Instr::band(Register::RAX, Register::RBX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::LogOr => {
                        self.instrs.push(Instr::comment("BinOp Logical Or"));

                        let mut rhs_label: Label = Label::new_at(".rhs_", &node.tok.pos());
                        let mut end_label: Label = Label::new_at(".end_", &node.tok.pos());

                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RAX),
                            Instr::pop(Register::RBX),
                            Instr::jzero(Register::RAX, &rhs_label, node.datatype.size()),
                            Instr::jlabel(&end_label)
                        ]);

                        self.record_label(&mut rhs_label);
                        self.instrs.push(Instr::label(&rhs_label));
                        
                        self.instrs.push(Instr::int_lit_a_ne_reg_b(0, Register::RBX, node.datatype.size()));
                        self.instrs.push(Instr::copy_reg_b_to_a(Register::RAX, Register::RBX, node.datatype.size()));

                        self.record_label(&mut end_label);
                        self.instrs.push(Instr::label(&end_label));

                        self.instrs.push(Instr::push_reg(Register::RAX));
                    },
                    TokKind::LogAnd => {
                        self.instrs.push(Instr::comment("BinOp Logical And"));

                        let mut false_label: Label = Label::new_at(".false_", &node.tok.pos());
                        let mut end_label: Label = Label::new_at(".end_", &node.tok.pos());
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RAX),
                            Instr::pop(Register::RBX),
                            Instr::jzero(Register::RAX, &false_label, node.datatype.size()),
                            Instr::jzero(Register::RBX, &false_label, node.datatype.size()),
                            Instr::push_int_lit(1),
                            Instr::jlabel(&end_label),
                        ]);

                        self.record_label(&mut false_label);
                        self.instrs.push(Instr::label(&false_label));
                        self.instrs.push(Instr::push_int_lit(0));

                        self.record_label(&mut end_label);
                        self.instrs.push(Instr::label(&end_label));
                    },
                    TokKind::Subscript => {
                        // Stack contains var, index
                        let lhs: &Node = node.children.first().unwrap();
                        let size: i64;
                        if let Some(var) = ctx.locals.get(&lhs.tok.val) {
                            size = var.typ.base_type().unwrap().size();
                        } else {
                            panic!("{} Error: Attempting to assign undeclared variable `{}`", lhs.tok.pos, lhs.tok.val_str());
                        }
                        self.instrs.append(&mut vec![
                            Instr::comment("BinOp Subscript"),
                            Instr::pop(Register::RBX), // index
                            Instr::pop(Register::RAX), // var
                            Instr::copy_int_lit_b_to_a(Register::RCX, size, 8), // sizeof var type
                            Instr::mul_regs(Register::RBX, Register::RCX, 8), // offset
                            Instr::add_regs(Register::RAX, Register::RBX, 8), // add offset to addy
                            Instr::deref_reg(Register::RAX, 8),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    _ => unreachable!(),
                }
            },
            NodeKind::UnaryOp => {
                match node.tok.kind {
                    TokKind::Not => {
                        self.instrs.push(Instr::comment("UnOp Not"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RAX),
                            Instr::reg_b_eq_int_lit_a(Register::RAX, 0, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Minus => {
                        self.instrs.push(Instr::comment("UnOp Minus"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RAX),
                            Instr::neg_reg(Register::RAX, node.datatype.size()),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    TokKind::Deref => {
                        self.instrs.push(Instr::comment("UnOp Dereference"));
                        self.instrs.append(&mut vec![
                            Instr::pop(Register::RAX),
                            Instr::deref_reg(Register::RAX, 8),
                            Instr::push_reg(Register::RAX),
                        ]);
                    },
                    _ => unreachable!(),
                }
            },
            NodeKind::LiteralInt => {
                self.instrs.push(Instr::comment("Literal Int"));
                let lit: i64 = Lexer::vec_val(&node.tok.val);
                self.instrs.push(Instr::push_int_lit(lit));
            },
            NodeKind::LiteralString => {
                self.instrs.push(Instr::comment("Literal String"));
                self.instrs.push(Instr::push_str_lit(self.strs.len() as i64));
                self.strs.push(node.tok.val.clone());
            },
            NodeKind::LiteralChar => {
                self.instrs.push(Instr::comment("Literal Char"));
                let lit: i64 = *node.tok.val.first().unwrap() as i64;
                self.instrs.push(Instr::push_int_lit(lit));
            },
            NodeKind::Var => {
                self.instrs.push(Instr::comment("Var"));
                let var: Option<&Var> = ctx.locals.get(&node.tok.val);
                if let Some(var) = var {
                    self.instrs.push(Instr::copy_var_to_reg(Register::RAX, var.clone(), var.typ.size()));
                    self.instrs.push(Instr::push_reg(Register::RAX));
                } else {
                    panic!("{} Error: Could not find variable `{}`", node.tok.pos, node.tok.val_str());
                }
            },
            NodeKind::VarDecl => {
                self.instrs.push(Instr::comment("Var Decl"));
                if ctx.locals.contains_key(&node.tok.val) {
                    panic!("{} Error: Attempting to redeclare variable `{}`", node.tok.pos, node.tok.val_str());
                }

                let var: Var = Var {
                    typ: node.datatype.clone(),
                    offset: ctx.stack_ix,
                };
                ctx.locals.insert(node.tok.val.clone(), var);
                ctx.stack_ix -= 8;

                self.instrs.push(Instr::sub_from_reg(8, Register::RSP, 8));
            },
            NodeKind::Syscall => self.generate_syscall(node, ctx),
            NodeKind::FuncCall => self.generate_func_call(node, ctx),
            NodeKind::Continue => {
                if ctx.ident.kind == TokKind::None {
                    panic!("{} Error: Unexpected continue in invalid scope `{}`", ctx.ident.pos, ctx.ident.val_str());
                }

                // Iterate over statements in func to find this continue to see how 
                // many local variables have been declared, hence how many to dealloc
                let mut var_cnt: i64 = 0;
                let func: Node = self.ast.find_node(&ctx.ident);
                for n in &func.children.last().unwrap().in_order() {
                    if n.kind == NodeKind::VarDecl { var_cnt += 1; }
                    if *n == *node { break; }
                }

                if var_cnt > 0 {
                    self.instrs.push(Instr::dealloc_stack(var_cnt * 8));
                }               

                self.instrs.push(Instr::comment("Continue"));

                // This label should exist at this point.
                let label: Label = Label::new_at(".continue_", &ctx.ident.pos());
                self.instrs.push(Instr::jlabel(&label));
            },
            NodeKind::Break => {
                if ctx.ident.kind == TokKind::None {
                    panic!("{} Error: Unexpected break in invalid scope `{}`", ctx.ident.pos, ctx.ident.val_str());
                }

                // Iterate over statements in func to find this break to see how 
                // many local variables have been declared, hence how many to dealloc
                let mut var_cnt: i64 = 0;
                let func: Node = self.ast.find_node(&ctx.ident);
                for n in &func.children.last().unwrap().in_order() {
                    if n.kind == NodeKind::VarDecl { var_cnt += 1; }
                    if n.kind == NodeKind::Continue && n.tok.pos == node.tok.pos { break; }
                }
                if var_cnt > 0 {
                    self.instrs.push(Instr::dealloc_stack(var_cnt * 8));
                }

                self.instrs.push(Instr::comment("Break"));

                // This label should exist at this point.
                let label: Label = Label::new_at(".break_", &ctx.ident.pos());
                self.instrs.push(Instr::jlabel(&label));
            },
            NodeKind::Return => {
                self.instrs.push(Instr::comment("Return"));

                // Return type in RAX
                self.instrs.push(Instr::pop(Register::RAX));
                self.instrs.push(Instr::jlabel(&Label::new(".epilogue")));
            },
            _ => panic!("{} Error: Unexpected node type in IR gen `{:?}`", node.tok.pos, node.kind)
        }
    }

    fn record_label(&mut self, label: &mut Label) {
        label.index = self.labels.len();
        self.labels.insert(label.clone());
    }
}
