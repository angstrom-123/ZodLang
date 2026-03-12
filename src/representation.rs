use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::lexer::{Lexer, Pos, Token, TokenType};
use crate::parser::{DataType, NodeType, ParseNode, ParseTree};

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
pub enum InstructionType {
    // Temporary,
    // Until i have an stdlib, these functions are hardcoded in asm, and are 
    // embedded in the binary be default. Instructions tell asmgen to define them.
    DefineIntrinsicDump,
    DefineIntrinsicExit,
    DefineIntrinsicMMap,
    DefineIntrinsicMUnmap,

    Comment,

    StartDataSegment,

    PushStackLiteralString,        // Pushes the adress of a literal string onto the stack
    DeclareString,                 // Declare a new string (intended to be in the data segment)
    Syscall,                       // Make a syscall
    PushStackRegister,             // Push a register onto the stack
    PushStackLiteralInt,           // Push a register onto the stack
    PopStack,                      // Pop the stack into a register
    AddRegisterBToA,               // Adds registers A and B, result in A
    MulRegisterAByB,               // Multiplies registers A by B, result in A
    DivAByBManglingD,              // Divides A by B, mangles D, result in RAX
    SubRegisterBFromA,             // Subtracts registers B from A, result in A
    CopyRegisterBToA,              // Copy value from register B to register A 
    CopyRegisterToVar,             // Copy value from a register to variable 
    CopyLiteralIntToRegister,      // Sets a register to a value
    CopyVarValToRegister,          // Copies the value of a variable to a register
    CopyRegisterAToAdrAtRegisterB, // Copies a variable to the adress in the register
    MakeLabel,                     // Make a new label
    ReturnToCaller,                // Return to caller of function
    JumpIfZero,                    // Jumps to a label if the operand is 0
    RegisterBLessA,                // Stores 1 in register A if register B < A, else 0
    RegisterBLessEqA,              // Stores 1 in register A if register B <= A, else 0
    RegisterBGreaterA,             // Stores 1 in register A if register B > A, else 0
    RegisterBGreaterEqA,           // Stores 1 in register A if register B >= A, else 0
    RegisterBEqA,                  // Stores 1 in register A if register B == A, else 0
    RegisterBNEqA,                 // Stores 1 in register A if register B != A, else 0
    RegisterBNEqLiteralIntA,       // Stores 1 in register A if literal B != A, else 0
    DeallocateStackBytes,          // Deallocates some bytes from the stack
    JumpToLabel,                   // Jumps unconditionally to a label
    CallFunction,                  // Calls a function by its label
    ZeroRegister,                  // Sets a register to 0
    NegateRegister,                // Negates the value in a register
    DereferenceRegister,           // Dereferences the registers adress into itself
    SubLiteralIntFromRegister,     // Subtracts an immediate value from a register
}
#[derive(Clone, Copy, PartialEq)]
pub enum RegisterSize {
    Byte,
    Word,
    DWord,
    QWord
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

#[derive(Clone)]
pub enum Operand {
    None,
    Register {
        name: Register,
        size: i64
    },
    StackOffset {
        value: i64,
    },
    LiteralInt {
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
            Operand::StackOffset { value } => write!(f, "{}{}", if *value < 0 { "" } else { "+" }, value),
            Operand::LiteralInt { value } => write!(f, "{}", value),
            Operand::Comment { comment } => write!(f, "{}", comment),
            Operand::Name { name } => write!(f, "{}", name),
            Operand::Bytes { bytes } => write!(f, "{}", bytes.iter().map(|b| format!("0x{:02X}", b).to_string()).collect::<Vec<String>>().join(","))
        }
    }
}
impl Operand {
    pub fn byte(&self) -> String {
        if let Operand::Register { name, size: _ } = self {
            format!("{}", Operand::Register { name: *name, size: 1 })
        } else { panic!("Cannot convert non-register operands to byte") }
    }
}

#[derive(Clone)]
pub struct Instruction {
    pub kind: InstructionType,
    pub opera: Operand,
    pub operb: Operand,
}
impl Instruction {
    pub fn dump(&self) {
        eprintln!("{:?}\n    a: {}\n    b: {}", self.kind, self.opera, self.operb);
    }

    fn _define_intrinsic_munmap() -> Self {
        Instruction {
            kind: InstructionType::DefineIntrinsicMUnmap,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _define_intrinsic_dump() -> Self {
        Instruction {
            kind: InstructionType::DefineIntrinsicDump,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _define_intrinsic_exit() -> Self {
        Instruction {
            kind: InstructionType::DefineIntrinsicExit,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _define_intrinsic_mmap() -> Self {
        Instruction {
            kind: InstructionType::DefineIntrinsicMMap,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _comment(comment: &'static str) -> Self {
        Instruction {
            kind: InstructionType::Comment,
            opera: Operand::Comment { comment: comment.into() },
            operb: Operand::None,
        }
    }

    fn _declare_string(index: i64, bytes: Vec<u8>) -> Self {
        Instruction {
            kind: InstructionType::DeclareString,
            opera: Operand::LiteralInt { value: index },
            operb: Operand::Bytes { bytes },
        }
    }

    fn _push_stack_literal_string(index: i64) -> Self {
        Instruction {
            kind: InstructionType::PushStackLiteralString,
            opera: Operand::LiteralInt { value: index },
            operb: Operand::None,
        }
    }

    fn _start_data_segment() -> Self {
        Instruction {
            kind: InstructionType::StartDataSegment,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _sub_literal_int_from_register(value: i64, name: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::SubLiteralIntFromRegister,
            opera: Operand::LiteralInt { value },
            operb: Operand::Register { name, size },
        }
    }


    fn _push_stack_register(name: Register) -> Self {
        Instruction {
            kind: InstructionType::PushStackRegister,
            opera: Operand::Register { name, size: 8 },
            operb: Operand::None,
        }
    }

    fn _copy_var_val_to_register(name: Register, var: Var, size: i64) -> Self {
        Instruction {
            kind: InstructionType::CopyVarValToRegister,
            opera: Operand::Register { name, size },
            operb: Operand::StackOffset { value: var.offset },
        }
    }

    fn _copy_literal_int_to_register(name: Register, val: i64, size: i64) -> Self {
        Instruction {
            kind: InstructionType::CopyLiteralIntToRegister,
            opera: Operand::Register { name, size },
            operb: Operand::LiteralInt { value: val },
        }
    }

    fn _zero_register(name: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::ZeroRegister,
            opera: Operand::Register { name, size },
            operb: Operand::None,
        }
    }

    fn _negate_register(name: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::NegateRegister,
            opera: Operand::Register { name, size },
            operb: Operand::None,
        }
    }

    fn _pop_stack(name: Register) -> Self {
        Instruction {
            kind: InstructionType::PopStack,
            opera: Operand::Register { name, size: 8 },
            operb: Operand::None,
        }
    }

    fn _dereference_register(name: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::DereferenceRegister,
            opera: Operand::Register { name, size },
            operb: Operand::None,
        }
    }

    fn _copy_register_to_var(var: Var, name: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::CopyRegisterToVar,
            opera: Operand::StackOffset { value: var.offset },
            operb: Operand::Register { name, size },
        }
    }

    fn _sub_register_b_from_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::SubRegisterBFromA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _add_register_b_to_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::AddRegisterBToA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _mul_register_a_by_b(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::MulRegisterAByB,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _div_a_by_b_mangling_d(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::DivAByBManglingD,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _copy_register_b_to_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::CopyRegisterBToA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _label(l: &Label) -> Self {
        Instruction {
            kind: InstructionType::MakeLabel,
            opera: Operand::Name { name: l.clone() },
            operb: Operand::None,
        }
    }

    fn _return_to_caller() -> Self {
        Instruction {
            kind: InstructionType::ReturnToCaller,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _jump_if_zero(name: Register, label: &Label, size: i64) -> Self {
        Instruction {
            kind: InstructionType::JumpIfZero,
            opera: Operand::Register { name, size },
            operb: Operand::Name { name: label.clone() },
        }
    }

    fn _jump_to_label(label: &Label) -> Self {
        Instruction {
            kind: InstructionType::JumpToLabel,
            opera: Operand::Name { name: label.clone() },
            operb: Operand::None,
        }
    }

    fn _deallocate_stack_bytes(bytes: i64) -> Self {
        Instruction {
            kind: InstructionType::DeallocateStackBytes,
            opera: Operand::LiteralInt { value: bytes },
            operb: Operand::None,
        }
    }

    fn _call_function(label: &Label) -> Self {
        Instruction {
            kind: InstructionType::CallFunction,
            opera: Operand::Name { name: label.clone() },
            operb: Operand::None,
        }
    }

    fn _syscall() -> Self {
        Instruction {
            kind: InstructionType::Syscall,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _push_stack_literal_int(val: i64) -> Self {
        Instruction { 
            kind: InstructionType::PushStackLiteralInt,
            opera: Operand::LiteralInt { value: val },
            operb: Operand::None,
        }
    }

    fn _register_b_less_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::RegisterBLessA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _register_b_less_eq_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::RegisterBLessEqA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _register_b_greater_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::RegisterBGreaterA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _register_b_greater_eq_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::RegisterBGreaterEqA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _register_b_eq_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::RegisterBEqA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _register_b_neq_a(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::RegisterBNEqA,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _literal_int_a_neq_register_b(a: i64, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::RegisterBNEqLiteralIntA,
            opera: Operand::LiteralInt { value: a },
            operb: Operand::Register { name: b, size },
        }
    }

    fn _copy_register_a_to_adr_at_register_b(a: Register, b: Register, size: i64) -> Self {
        Instruction {
            kind: InstructionType::CopyRegisterAToAdrAtRegisterB,
            opera: Operand::Register { name: a, size },
            operb: Operand::Register { name: b, size },
        }
    }
}

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
pub struct Var {
    typ: DataType,
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
    ident: Token,
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
            ident: Token::null(),
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

    pub fn under(ident: &Token, ctx: &Context) -> Self {
        let mut res: Context = Context {
            ident: ident.clone(),
            locals: HashMap::new(),
            stack_ix: ctx.stack_ix,
        };
        res.locals.extend(ctx.locals.clone());
        res
    }
}

pub struct IR {
    pub instrs: Vec<Instruction>,
    labels: HashSet<Label>,
    strs: Vec<Vec<u8>>,
}
impl Default for IR {
    fn default() -> Self {
        Self::new()
    }
}
impl IR {
    pub fn new() -> Self {
        IR {
            instrs: Vec::new(),
            labels: HashSet::new(),
            strs: Vec::new(),
        }
    }

    pub fn dump(&self) {
        for instr in &self.instrs {
            instr.dump();
        }
    }

    pub fn generate_from_ast(&mut self, ast: &ParseTree) {
        // Function like intrinsics are hard coded in asm right now
        self.record_label(&mut Label::new("func_dump"));
        self.instrs.push(Instruction::_define_intrinsic_dump());

        self.record_label(&mut Label::new("func_exit"));
        self.instrs.push(Instruction::_define_intrinsic_exit());

        self.record_label(&mut Label::new("func_mmap"));
        self.instrs.push(Instruction::_define_intrinsic_mmap());

        self.record_label(&mut Label::new("func_munmap"));
        self.instrs.push(Instruction::_define_intrinsic_munmap());

        // NOTE: Skipping the first 4 declarations, because those are the hardcoded intrinsice
        for stmt in ast.root.children.iter().skip(4) {
            match stmt.kind {
                NodeType::FuncDecl => {
                    self.instrs.push(Instruction::_comment("Function Declaration"));

                    let mut top_label: Label = Label::new_named("func_", &stmt.tok.val);
                    let mut bottom_label: Label = Label::new(".epilogue");

                    // Save function label
                    self.record_label(&mut top_label);
                    self.instrs.push(Instruction::_label(&top_label));

                    // Prologue
                    self.instrs.append(&mut vec![
                        Instruction::_push_stack_register(Register::RBP),
                        Instruction::_copy_register_b_to_a(Register::RBP, Register::RSP, 8),
                    ]);

                    // Args and context
                    let mut ctx: Context = Context::new();
                    ctx.stack_ix = -8;

                    if let Some(args) = stmt.children.first() {
                        if args.children.len() > 6 {
                            panic!("{} Error: Currently only up to 6 function args are supported", stmt.tok.pos);
                        }

                        self.instrs.push(Instruction::_comment("Args"));

                        // Save args as local variables on the stack
                        for (i, arg) in args.children.iter().enumerate() {
                            let reg: Register = USER_REG_ORDER[i];
                            self.instrs.push(Instruction::_push_stack_register(reg));

                            assert!(!matches!(arg.datatype, DataType::None | DataType::Unknown), "Variable type must be known");
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
                        self.instrs.push(Instruction::_comment("Body"));
                        for stmt in &body.children {
                            self.generate_from_statement(stmt, &mut ctx);
                        }
                    } else {
                        panic!("{} Error: Expected function body for `{}` but found nothing", stmt.tok.val_str(), stmt.tok.pos);
                    }

                    // Epilogue
                    self.record_label(&mut bottom_label);
                    self.instrs.push(Instruction::_label(&bottom_label));

                    // On a local stack frame so we automatically deallocate the locals when returning
                    self.instrs.append(&mut vec![
                        Instruction::_copy_register_b_to_a(Register::RSP, Register::RBP, 8),
                        Instruction::_pop_stack(Register::RBP),
                        Instruction::_return_to_caller(),
                    ]);
                },
                _ => unreachable!()
            }
        }

        self.instrs.push(Instruction::_start_data_segment());
        for (i, s) in self.strs.iter().enumerate() {
            self.instrs.push(Instruction::_declare_string(i as i64, s.clone()));
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
    fn generate_syscall(&mut self, call: &ParseNode, _ctx: &mut Context) {
        if call.children.len() > 7 {
            panic!("{} Error: Passing to many args to syscall. Maximum is 7", call.tok.pos);
        }

        self.instrs.push(Instruction::_comment("Syscall"));

        // Calling convention puts first few args into registers
        for (i, reg) in KERNEL_REG_ORDER.iter().enumerate() {
            if i == call.children.len() {
                break;
            }
            self.instrs.push(Instruction::_pop_stack(*reg));
        }

        self.instrs.append(&mut vec![
            Instruction::_syscall(),
            Instruction::_push_stack_register(Register::RAX)
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
    fn generate_func_call(&mut self, call: &ParseNode, _ctx: &mut Context) {
        if call.children.len() > 6 {
            panic!("{} Error: Passing to many args to function `{}`. Currently there is support for up to 6 args",
                   call.tok.pos, call.tok.val_str());
        }

        self.instrs.push(Instruction::_comment("Function Call"));

        // Calling convention puts first few args into registers
        for (i, reg) in USER_REG_ORDER.iter().enumerate() {
            if i == call.children.len() {
                break;
            }
            self.instrs.push(Instruction::_pop_stack(*reg));
        }

        let label: Label = Label::new_named("func_", &call.tok.val);
        self.instrs.push(Instruction::_call_function(&label));

        if call.datatype != DataType::Void {
            self.instrs.push(Instruction::_push_stack_register(Register::RAX));
        }
    }

    fn generate_from_statement(&mut self, stmt: &ParseNode, ctx: &mut Context) {
        match stmt.kind {
            NodeType::VarDecl
            | NodeType::Continue
            | NodeType::Break
            | NodeType::Return
            | NodeType::Syscall
            | NodeType::FuncCall => {
                for node in &stmt.post_order() {
                    self.generate_from_node(node, ctx);
                }
            },
            NodeType::Assign => {
                self.instrs.push(Instruction::_comment("Assign"));

                // Evaluate RHS first
                if let Some(rhs) = stmt.children.get(1) {
                    for node in &rhs.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                } else {
                    panic!("{} Error: Failed to get rhs of assignment", stmt.tok.pos);
                }

                if let Some(lhs_group) = stmt.children.first() {
                    if lhs_group.kind == NodeType::UnaryOp && lhs_group.tok.kind == TokenType::OpDereference {
                        let lhs: &ParseNode = lhs_group.children.first().unwrap();
                        if lhs.kind == NodeType::Var { // Dereferencing a var
                            if let Some(var) = ctx.locals.get(&lhs.tok.val) {
                                self.instrs.append(&mut vec![
                                    Instruction::_pop_stack(Register::RAX),
                                    Instruction::_copy_var_val_to_register(Register::RBX, var.clone(), var.typ.size()), // address into rbx
                                    Instruction::_copy_register_a_to_adr_at_register_b(Register::RAX, Register::RBX, 8),
                                ]);
                            } else {
                                panic!("{} Error: Attempting to assign undeclared variable `{}`", stmt.tok.pos, lhs.tok.val_str());
                            }
                        } else { // Dereferencing some expression
                            for node in &lhs.post_order() {
                                self.generate_from_node(node, ctx);
                            }
                            self.instrs.append(&mut vec![
                                Instruction::_pop_stack(Register::RBX), // lhs
                                Instruction::_pop_stack(Register::RAX), // rhs
                                Instruction::_copy_register_a_to_adr_at_register_b(Register::RAX, Register::RBX, 8),
                            ]);
                        }
                    } else if lhs_group.kind == NodeType::Var {
                        if let Some(var) = ctx.locals.get(&lhs_group.tok.val) {
                            self.instrs.append(&mut vec![
                                Instruction::_pop_stack(Register::RAX),
                                Instruction::_copy_register_to_var(var.clone(), Register::RAX, var.typ.size())
                            ]);
                        } else {
                            panic!("{} Error: Attempting to assign undeclared variable `{}`", stmt.tok.pos, lhs_group.tok.val_str());
                        }
                    } else if lhs_group.kind == NodeType::BinaryOp && lhs_group.tok.kind == TokenType::OpSubscript {
                        let lhs: &ParseNode = lhs_group.children.first().unwrap();
                        let rhs: &ParseNode = lhs_group.children.last().unwrap();
                        self.generate_from_node(rhs, ctx);
                        if let Some(var) = ctx.locals.get(&lhs.tok.val) {
                            let size: i64 = var.typ.base_type().unwrap().size();
                            self.instrs.append(&mut vec![
                                Instruction::_pop_stack(Register::RBX), // index
                                Instruction::_pop_stack(Register::RAX), // value
                                Instruction::_copy_literal_int_to_register(Register::RCX, size, 8), // sizeof type
                                Instruction::_mul_register_a_by_b(Register::RBX, Register::RCX, 8), // offset
                                Instruction::_copy_var_val_to_register(Register::RDX, var.clone(), 8), // address into rcx
                                Instruction::_add_register_b_to_a(Register::RDX, Register::RBX, 8), // add offset to addy
                                Instruction::_copy_register_a_to_adr_at_register_b(Register::RAX, Register::RDX, 8), // assign
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
            NodeType::WhileLoop => {
                self.instrs.push(Instruction::_comment("While"));
                let mut loop_label: Label = Label::new_at(".while_", &stmt.tok.pos());
                let mut post_label: Label = Label::new_at(".post_", &stmt.tok.pos());
                let mut break_label: Label = Label::new_at(".break_", &stmt.tok.pos());
                let mut end_label: Label = Label::new_at(".end_", &stmt.tok.pos());

                // Jump Point
                self.record_label(&mut loop_label);
                self.instrs.push(Instruction::_label(&loop_label));

                // Condition
                if let Some(cond) = stmt.children.first() {
                    for node in &cond.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                    self.instrs.push(Instruction::_pop_stack(Register::RAX));
                    self.instrs.push(Instruction::_jump_if_zero(Register::RAX, &end_label, cond.datatype.size()));
                }

                // Body
                let mut while_ctx: Context = Context::under(&stmt.tok, ctx);
                if let Some(body) = stmt.children.get(1) && !body.is_null() {
                    for node in &body.children {
                        self.generate_from_statement(node, &mut while_ctx);
                    }
                }

                self.record_label(&mut post_label);
                // Deallocate body locals
                let new_locals: usize = while_ctx.locals.len() - ctx.locals.len();
                if new_locals > 0 {
                    self.instrs.push(Instruction::_deallocate_stack_bytes(new_locals as i64 * 8));
                }

                self.instrs.push(Instruction::_label(&post_label));

                self.instrs.push(Instruction::_jump_to_label(&loop_label));

                // Breaking out could skip deallocating locals, so we clean up here
                self.record_label(&mut break_label);
                self.instrs.push(Instruction::_label(&break_label));

                // Deallocate body locals
                let new_locals: usize = while_ctx.locals.len() - ctx.locals.len();
                if new_locals > 0 {
                    self.instrs.push(Instruction::_deallocate_stack_bytes(new_locals as i64 * 8));
                }

                self.record_label(&mut end_label);
                self.instrs.push(Instruction::_label(&end_label));
            }, 
            NodeType::ForLoop => {
                self.instrs.push(Instruction::_comment("For"));
                // Labels
                let mut loop_label: Label = Label::new_at(".for_", &stmt.tok.pos());
                let mut post_label: Label = Label::new_at(".post_", &stmt.tok.pos());
                let mut break_label: Label = Label::new_at(".break_", &stmt.tok.pos());
                let mut end_label: Label = Label::new_at(".end_", &stmt.tok.pos());

                // Decl 
                let mut decl_tok: Option<&Token> = None;
                if let Some(decl) = stmt.children.first() && !decl.is_null() {
                    self.instrs.push(Instruction::_comment("Decl"));
                    decl_tok = Some(&decl.tok);
                    for node in &decl.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                }

                // Init 
                if let Some(init) = stmt.children.get(1) && !init.is_null() {
                    self.instrs.push(Instruction::_comment("Init"));
                    self.generate_from_statement(init, ctx);
                }

                let mut for_ctx: Context = Context::under(&stmt.tok, ctx);

                // Condition 
                self.record_label(&mut loop_label);
                self.instrs.push(Instruction::_label(&loop_label));
                if let Some(cond) = stmt.children.get(2) && !cond.is_null() {
                    self.instrs.push(Instruction::_comment("Condition"));
                    for node in &cond.post_order() {
                        self.generate_from_node(node, ctx);
                    }

                    self.instrs.push(Instruction::_pop_stack(Register::RAX));
                    self.instrs.push(Instruction::_jump_if_zero(Register::RAX, &end_label, cond.datatype.size()));
                }

                // Body
                if let Some(body) = stmt.children.get(4) && !body.is_null() {
                    self.instrs.push(Instruction::_comment("Body"));
                    for node in &body.children {
                        self.generate_from_statement(node, &mut for_ctx);
                    }
                }

                // Post 
                self.instrs.push(Instruction::_comment("Post"));
                self.record_label(&mut post_label);
                self.instrs.push(Instruction::_label(&post_label));

                let new_locals: usize = for_ctx.locals.len() - ctx.locals.len();
                if new_locals > 0 {
                    self.instrs.push(Instruction::_deallocate_stack_bytes(new_locals as i64 * 8));
                }

                if let Some(post) = stmt.children.get(3) && !post.is_null() {
                    self.generate_from_statement(post, &mut for_ctx);
                }
                self.instrs.push(Instruction::_jump_to_label(&loop_label));

                // Breaking out could skip deallocating locals, so we clean up here
                self.record_label(&mut break_label);
                self.instrs.push(Instruction::_label(&break_label));

                // Deallocate body locals
                let new_locals: usize = for_ctx.locals.len() - ctx.locals.len();
                if new_locals > 0 {
                    self.instrs.push(Instruction::_deallocate_stack_bytes(new_locals as i64 * 8));
                }

                self.instrs.push(Instruction::_comment("End"));
                self.record_label(&mut end_label);
                self.instrs.push(Instruction::_label(&end_label));

                // If a variable was declared in the for init, we remove it here
                if let Some(tok) = decl_tok {
                    ctx.locals.remove(&tok.val);
                    ctx.stack_ix += 8;
                    self.instrs.push(Instruction::_deallocate_stack_bytes(8));
                }
            },
            NodeType::Conditional => {
                self.instrs.push(Instruction::_comment("Conditional"));
                let mut end_label: Label = Label::new_at(".end_", &stmt.tok.pos());

                // Condition 
                let cond: &ParseNode = stmt.children.first().unwrap();
                for node in &cond.post_order() {
                    self.generate_from_node(node, ctx);
                }
                self.instrs.push(Instruction::_pop_stack(Register::RAX));

                if stmt.children.len() == 2 { // There is no else block
                    // Jump to end if condition fails
                    self.instrs.push(Instruction::_jump_if_zero(Register::RAX, &end_label, cond.datatype.size()));

                    // If body
                    self.instrs.push(Instruction::_comment("If"));
                    let mut if_ctx: Context = Context::from(ctx);
                    if let Some(if_block) = stmt.children.get(1) {
                        for sub_stmt in &if_block.children {
                            self.generate_from_statement(sub_stmt, &mut if_ctx);
                        }
                    }

                    // Deallocate if body locals
                    let new_locals: usize = if_ctx.locals.len() - ctx.locals.len();
                    if new_locals > 0 {
                        self.instrs.push(Instruction::_deallocate_stack_bytes(new_locals as i64 * 8));
                    }

                    // End label
                    self.record_label(&mut end_label);
                    self.instrs.push(Instruction::_label(&end_label));
                } else { // There is an else block
                    // Jump to else if condition fails
                    let mut else_label: Label = Label::new_at(".else_", &stmt.tok.pos());

                    self.instrs.push(Instruction::_jump_if_zero(Register::RAX, &else_label, cond.datatype.size()));

                    // If block
                    self.instrs.push(Instruction::_comment("If"));
                    let mut if_ctx: Context = Context::from(ctx);
                    if let Some(if_block) = stmt.children.get(1) {
                        for sub_stmt in &if_block.children {
                            self.generate_from_statement(sub_stmt, &mut if_ctx);
                        }
                    }

                    // Skip over `else` block if the `if` was hit
                    self.instrs.push(Instruction::_jump_to_label(&end_label));

                    // Else label to jump to if condition is false
                    self.record_label(&mut else_label);
                    self.instrs.push(Instruction::_label(&else_label));

                    // Else block
                    self.instrs.push(Instruction::_comment("Else"));
                    let mut else_ctx: Context = Context::from(ctx);
                    if let Some(else_block) = stmt.children.get(2) {
                        for sub_stmt in &else_block.children {
                            self.generate_from_statement(sub_stmt, &mut else_ctx);
                        }
                    }
                    
                    // End label
                    self.record_label(&mut end_label);
                    self.instrs.push(Instruction::_label(&end_label));
                }
            },
            NodeType::Null => {},
            _ => panic!("{} Error: Expected statement but got {:?}:{:?} `{}`", stmt.tok.pos, stmt.kind, stmt.tok.kind, stmt.tok.val_str())
        }
    }

    fn generate_from_node(&mut self, node: &ParseNode, ctx: &mut Context) {
        match node.kind {
            NodeType::BinaryOp => {
                match node.tok.kind {
                    TokenType::OpPlus => {
                        self.instrs.push(Instruction::_comment("BinOp Plus"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_add_register_b_to_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpMinus => {
                        self.instrs.push(Instruction::_comment("BinOp Minus"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_sub_register_b_from_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpMul => {
                        self.instrs.push(Instruction::_comment("BinOp Mul"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_mul_register_a_by_b(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpDiv => {
                        self.instrs.push(Instruction::_comment("BinOp Div"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_div_a_by_b_mangling_d(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpEqual => {
                        self.instrs.push(Instruction::_comment("BinOp Eq"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_register_b_eq_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpNotEqual => {
                        self.instrs.push(Instruction::_comment("BinOp NEq"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_register_b_neq_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpGreaterThan => {
                        self.instrs.push(Instruction::_comment("BinOp GT"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_register_b_greater_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpLessThan => {
                        self.instrs.push(Instruction::_comment("BinOp LT"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_register_b_less_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpGreaterEqual => {
                        self.instrs.push(Instruction::_comment("BinOp GE"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_register_b_greater_eq_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpLessEqual => {
                        self.instrs.push(Instruction::_comment("BinOp LE"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_register_b_less_eq_a(Register::RAX, Register::RBX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpLogicalOr => {
                        self.instrs.push(Instruction::_comment("BinOp Logical Or"));

                        let mut rhs_label: Label = Label::new_at(".rhs_", &node.tok.pos());
                        let mut end_label: Label = Label::new_at(".end_", &node.tok.pos());

                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_jump_if_zero(Register::RAX, &rhs_label, node.datatype.size()),
                            Instruction::_jump_to_label(&end_label)
                        ]);

                        self.record_label(&mut rhs_label);
                        self.instrs.push(Instruction::_label(&rhs_label));
                        
                        self.instrs.push(Instruction::_literal_int_a_neq_register_b(0, Register::RBX, node.datatype.size()));
                        self.instrs.push(Instruction::_copy_register_b_to_a(Register::RAX, Register::RBX, node.datatype.size()));

                        self.record_label(&mut end_label);
                        self.instrs.push(Instruction::_label(&end_label));

                        self.instrs.push(Instruction::_push_stack_register(Register::RAX));
                    },
                    TokenType::OpLogicalAnd => {
                        self.instrs.push(Instruction::_comment("BinOp Logical And"));

                        let mut rhs_label: Label = Label::new_at(".rhs_", &node.tok.pos());
                        let mut end_label: Label = Label::new_at(".end_", &node.tok.pos());

                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_pop_stack(Register::RBX),
                            Instruction::_jump_if_zero(Register::RAX, &end_label, node.datatype.size()),
                            Instruction::_jump_to_label(&rhs_label)
                        ]);

                        self.record_label(&mut rhs_label);
                        self.instrs.push(Instruction::_label(&rhs_label));
                        
                        self.instrs.push(Instruction::_literal_int_a_neq_register_b(0, Register::RBX, node.datatype.size()));

                        self.record_label(&mut end_label);
                        self.instrs.push(Instruction::_label(&end_label));

                        self.instrs.push(Instruction::_push_stack_register(Register::RAX));
                    },
                    TokenType::OpSubscript => {
                        // Stack contains var, index
                        let lhs: &ParseNode = node.children.first().unwrap();
                        let size: i64;
                        if let Some(var) = ctx.locals.get(&lhs.tok.val) {
                            size = var.typ.base_type().unwrap().size();
                        } else {
                            panic!("{} Error: Attempting to assign undeclared variable `{}`", lhs.tok.pos, lhs.tok.val_str());
                        }
                        // TODO

                        self.instrs.append(&mut vec![
                            Instruction::_comment("BinOp Subscript"),
                            Instruction::_pop_stack(Register::RBX), // index
                            Instruction::_pop_stack(Register::RAX), // var
                            Instruction::_copy_literal_int_to_register(Register::RCX, size, 8), // sizeof var type
                            Instruction::_mul_register_a_by_b(Register::RBX, Register::RCX, 8), // offset
                            Instruction::_add_register_b_to_a(Register::RAX, Register::RBX, 8), // add offset to addy
                            Instruction::_dereference_register(Register::RAX, 8),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    _ => unreachable!(),
                }
            },
            NodeType::UnaryOp => {
                match node.tok.kind {
                    TokenType::OpMinus => {
                        self.instrs.push(Instruction::_comment("UnOp Minus"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_negate_register(Register::RAX, node.datatype.size()),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    TokenType::OpDereference => {
                        self.instrs.push(Instruction::_comment("UnOp Dereference"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(Register::RAX),
                            Instruction::_dereference_register(Register::RAX, 8),
                            Instruction::_push_stack_register(Register::RAX),
                        ]);
                    },
                    _ => unreachable!(),
                }
            },
            NodeType::LiteralInt => {
                self.instrs.push(Instruction::_comment("Literal Int"));
                let lit: i64 = Lexer::vec_val(&node.tok.val);
                self.instrs.push(Instruction::_push_stack_literal_int(lit));
            },
            NodeType::LiteralString => {
                self.instrs.push(Instruction::_comment("Literal String"));
                self.instrs.push(Instruction::_push_stack_literal_string(self.strs.len() as i64));
                self.strs.push(node.tok.val.clone());
            },
            NodeType::LiteralChar => {
                self.instrs.push(Instruction::_comment("Literal Char"));
                let lit: i64 = *node.tok.val.first().unwrap() as i64;
                self.instrs.push(Instruction::_push_stack_literal_int(lit));
            },
            NodeType::Var => {
                self.instrs.push(Instruction::_comment("Var"));
                let var: Option<&Var> = ctx.locals.get(&node.tok.val);
                if let Some(var) = var {
                    self.instrs.push(Instruction::_copy_var_val_to_register(Register::RAX, var.clone(), var.typ.size()));
                    self.instrs.push(Instruction::_push_stack_register(Register::RAX));
                } else {
                    panic!("{} Error: Could not find variable `{}`", node.tok.pos, node.tok.val_str());
                }
            },
            NodeType::VarDecl => {
                self.instrs.push(Instruction::_comment("Var Decl"));
                if ctx.locals.contains_key(&node.tok.val) {
                    panic!("{} Error: Attempting to redeclare variable `{}`", node.tok.pos, node.tok.val_str());
                }

                let var: Var = Var {
                    typ: node.datatype.clone(),
                    offset: ctx.stack_ix,
                };
                ctx.locals.insert(node.tok.val.clone(), var);
                ctx.stack_ix -= 8;

                self.instrs.push(Instruction::_sub_literal_int_from_register(8, Register::RSP, 8));
            },
            NodeType::Syscall => self.generate_syscall(node, ctx),
            NodeType::FuncCall => self.generate_func_call(node, ctx),
            NodeType::Continue => {
                if ctx.ident.kind == TokenType::None {
                    panic!("{} Error: Unexpected continue in invalid scope `{}`", ctx.ident.pos, ctx.ident.val_str());
                }

                self.instrs.push(Instruction::_comment("Continue"));

                // This label should exist at this point.
                let label: Label = Label::new_at(".post_", &ctx.ident.pos());
                self.instrs.push(Instruction::_jump_to_label(&label));
            },
            NodeType::Break => {
                if ctx.ident.kind == TokenType::None {
                    panic!("{} Error: Unexpected break in invalid scope `{}`", ctx.ident.pos, ctx.ident.val_str());
                }

                self.instrs.push(Instruction::_comment("Break"));

                // This label should exist at this point.
                let label: Label = Label::new_at(".break_", &ctx.ident.pos());
                self.instrs.push(Instruction::_jump_to_label(&label));
            },
            NodeType::Return => {
                self.instrs.push(Instruction::_comment("Return"));

                // Return type in RAX
                self.instrs.push(Instruction::_pop_stack(Register::RAX));
                self.instrs.push(Instruction::_jump_to_label(&Label::new(".epilogue")));
            },
            _ => panic!("{} Error: Unexpected node type in IR gen `{:?}`", node.tok.pos, node.kind)
        }
    }

    fn record_label(&mut self, label: &mut Label) {
        label.index = self.labels.len();
        self.labels.insert(label.clone());
    }
}
