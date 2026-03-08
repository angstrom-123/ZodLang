use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::lexer::{Pos, Token, TokenType};
use crate::parser::{DataType, NodeType, ParseNode, ParseTree};

const REG_ORDER: [RegisterName; 6] = [
    RegisterName::RDI,
    RegisterName::RSI,
    RegisterName::RDX,
    RegisterName::RCX,
    RegisterName::R8,
    RegisterName::R9,
];

#[derive(Clone)]
#[derive(Debug)]
pub enum InstructionType {
    // Temporary,
    // Until i have an stdlib, these functions are hardcoded in asm, and are 
    // embedded in the binary be default. Instructions tell asmgen to define them.
    DefineIntrinsicDump,
    DefineIntrinsicExit,
    DefineIntrinsicMMap,
    DefineIntrinsicMUnmap,

    Comment,

    PushStackRegister,             // Push a register onto the stack
    PushStackLiteralInt,           // Push a register onto the stack
    PopStack,                      // Pop the stack into a register
    AddRegisterBToA,               // Adds registers A and B, result in A
    MulRegisterAByB,               // Multiplies registers A by B, result in A
    DivRCXByRAXManglingRDX,        // Divides RCX by RAX, mangles RDX, result in RAX
    SubRegisterBFromA,             // Subtracts registers B from A, result in A
    CopyRegisterBToA,              // Copy value from register B to register A 
    CopyRegisterToVar,             // Copy value from a register to variable 
    CopyLiteralIntToRegister,      // Sets a register to a value
    CopyVarValToRegister,          // Copies the value of a variable to a register
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
    CopyRegisterAToAdrAtRegisterB, // Copies a variable to the adress in the register
    DereferenceRegister,           // Dereferences the registers adress into itself
    SubLiteralIntFromRegister,     // Subtracts an immediate value from a register
}

#[derive(PartialEq)]
pub enum ByteRegister {
    AL,
    CL,
    DL,
    BL,
    SPL,
    BPL,
    SIL,
    DIL,
    R8B,
    R9B,
    R10B,
    R11B,
    R12B,
    R13B,
    R14B,
    R15B
}
impl fmt::Display for ByteRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ByteRegister::AL => write!(f, "al"),
            ByteRegister::CL => write!(f, "cl"),
            ByteRegister::DL => write!(f, "dl"),
            ByteRegister::BL => write!(f, "bl"),
            ByteRegister::SPL => write!(f, "spl"),
            ByteRegister::BPL => write!(f, "bpl"),
            ByteRegister::SIL => write!(f, "sil"),
            ByteRegister::DIL => write!(f, "dil"),
            ByteRegister::R8B => write!(f, "r8b"),
            ByteRegister::R9B => write!(f, "r9b"),
            ByteRegister::R10B  => write!(f, "r10b"),
            ByteRegister::R11B  => write!(f, "r11b"),
            ByteRegister::R12B  => write!(f, "r12b"),
            ByteRegister::R13B  => write!(f, "r13b"),
            ByteRegister::R14B  => write!(f, "r14b"),
            ByteRegister::R15B  => write!(f, "r15b"),
        }
    }
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum RegisterName {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15
}
impl fmt::Display for RegisterName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RegisterName::RAX => write!(f, "rax"),
            RegisterName::RCX => write!(f, "rcx"),
            RegisterName::RDX => write!(f, "rdx"),
            RegisterName::RBX => write!(f, "rbx"),
            RegisterName::RSP => write!(f, "rsp"),
            RegisterName::RBP => write!(f, "rbp"),
            RegisterName::RSI => write!(f, "rsi"),
            RegisterName::RDI => write!(f, "rdi"),
            RegisterName::R8 => write!(f, "r8"),
            RegisterName::R9 => write!(f, "r9"),
            RegisterName::R10 => write!(f, "r10"),
            RegisterName::R11 => write!(f, "r11"),
            RegisterName::R12 => write!(f, "r12"),
            RegisterName::R13 => write!(f, "r13"),
            RegisterName::R14 => write!(f, "r14"),
            RegisterName::R15 => write!(f, "r15"),
        }
    }
}
impl RegisterName {
    pub fn as_byte(&self) -> ByteRegister {
        match *self {
            RegisterName::RAX => ByteRegister::AL,
            RegisterName::RCX => ByteRegister::CL,
            RegisterName::RDX => ByteRegister::DL,
            RegisterName::RBX => ByteRegister::BL,
            RegisterName::RSP => ByteRegister::SPL,
            RegisterName::RBP => ByteRegister::BPL,
            RegisterName::RSI => ByteRegister::SIL,
            RegisterName::RDI => ByteRegister::DIL,
            RegisterName::R8 => ByteRegister::R8B,
            RegisterName::R9 => ByteRegister::R9B,
            RegisterName::R10 => ByteRegister::R10B,
            RegisterName::R11 => ByteRegister::R11B,
            RegisterName::R12 => ByteRegister::R12B,
            RegisterName::R13 => ByteRegister::R13B,
            RegisterName::R14 => ByteRegister::R14B,
            RegisterName::R15 => ByteRegister::R15B,
        }
    }
}

#[derive(Clone)]
pub enum Operand {
    None,
    Register {
        name: RegisterName,
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
    Comment {
        comment: String,
    },
}
impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::None => write!(f, "None"),
            Operand::Register { name } => write!(f, "Register `{}`", name),
            Operand::StackOffset { value } => write!(f, "Stack Offset `{}`", value),
            Operand::LiteralInt { value } => write!(f, "Literal Int `{}`", value),
            Operand::Comment { comment } => write!(f, "Comment `{}`", comment),
            Operand::Name { name } => write!(f, "Name `{}`", name),
        }
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

    fn _sub_literal_int_from_register(value: i64, name: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::SubLiteralIntFromRegister,
            opera: Operand::LiteralInt { value },
            operb: Operand::Register { name },
        }
    }


    fn _push_stack_register(name: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::PushStackRegister,
            opera: Operand::Register { name },
            operb: Operand::None,
        }
    }

    fn _copy_var_val_to_register(name: RegisterName, var: Var) -> Self {
        Instruction {
            kind: InstructionType::CopyVarValToRegister,
            opera: Operand::Register { name },
            operb: Operand::StackOffset { value: var.offset },
        }
    }

    fn _copy_literal_int_to_register(name: RegisterName, val: i64) -> Self {
        Instruction {
            kind: InstructionType::CopyLiteralIntToRegister,
            opera: Operand::Register { name },
            operb: Operand::LiteralInt { value: val },
        }
    }

    fn _zero_register(name: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::ZeroRegister,
            opera: Operand::Register { name },
            operb: Operand::None,
        }
    }

    fn _negate_register(name: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::NegateRegister,
            opera: Operand::Register { name },
            operb: Operand::None,
        }
    }

    fn _pop_stack(name: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::PopStack,
            opera: Operand::Register { name },
            operb: Operand::None,
        }
    }

    fn _dereference_register(name: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::DereferenceRegister,
            opera: Operand::Register { name },
            operb: Operand::None,
        }
    }

    fn _copy_register_to_var(var: Var, register: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::CopyRegisterToVar,
            opera: Operand::StackOffset { value: var.offset },
            operb: Operand::Register { name: register },
        }
    }

    fn _sub_register_b_from_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::SubRegisterBFromA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _add_register_b_to_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::AddRegisterBToA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _mul_register_a_by_b(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::MulRegisterAByB,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _div_rcx_by_rax_mangling_rdx() -> Self {
        Instruction {
            kind: InstructionType::DivRCXByRAXManglingRDX,
            opera: Operand::None,
            operb: Operand::None,
        }
    }

    fn _copy_register_b_to_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::CopyRegisterBToA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
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

    fn _jump_if_zero(reg: RegisterName, label: &Label) -> Self {
        Instruction {
            kind: InstructionType::JumpIfZero,
            opera: Operand::Register { name: reg },
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

    fn _push_stack_literal_int(val: i64) -> Self {
        Instruction { 
            kind: InstructionType::PushStackLiteralInt,
            opera: Operand::LiteralInt { value: val },
            operb: Operand::None,
        }
    }

    fn _register_b_less_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::RegisterBLessA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _register_b_less_eq_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::RegisterBLessEqA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _register_b_greater_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::RegisterBGreaterA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _register_b_greater_eq_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::RegisterBGreaterEqA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _register_b_eq_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::RegisterBEqA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _register_b_neq_a(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::RegisterBNEqA,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _literal_int_a_neq_register_b(a: i64, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::RegisterBNEqLiteralIntA,
            opera: Operand::LiteralInt { value: a },
            operb: Operand::Register { name: b },
        }
    }

    fn _copy_register_a_to_adr_at_register_b(a: RegisterName, b: RegisterName) -> Self {
        Instruction {
            kind: InstructionType::CopyRegisterAToAdrAtRegisterB,
            opera: Operand::Register { name: a },
            operb: Operand::Register { name: b },
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

    // vars: HashSet<Var>,
    labels: HashSet<Label>,
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
            // vars: HashSet::new(),
            labels: HashSet::new(),
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

        for func in &ast.root.children {
            if func.kind != NodeType::FuncDecl {
                panic!("{} Error: {:?} `{}` is not allowed at the top level", func.tok.pos, func.kind, func.tok.val_str());
            }

            self.instrs.push(Instruction::_comment("Function Declaration"));

            let mut top_label: Label = Label::new_named("func_", &func.tok.val);
            let mut bottom_label: Label = Label::new(".epilogue");

            // Save function label
            self.record_label(&mut top_label);
            self.instrs.push(Instruction::_label(&top_label));

            // Prologue
            self.instrs.append(&mut vec![
                Instruction::_push_stack_register(RegisterName::RBP),
                Instruction::_copy_register_b_to_a(RegisterName::RBP, RegisterName::RSP),
            ]);

            // Args and context
            let mut ctx: Context = Context::new();
            ctx.stack_ix = -8;

            if let Some(args) = func.children.first() {
                if args.children.len() > 6 {
                    panic!("{} Error: Currently only up to 6 function args are supported", func.tok.pos);
                }

                // Save args as local variables on the stack
                for (i, arg) in args.children.iter().enumerate() {
                    let reg: RegisterName = REG_ORDER[i];
                    self.instrs.push(Instruction::_push_stack_register(reg));

                    let var: Var = Var {
                        typ: arg.datatype.clone(),
                        offset: ctx.stack_ix,
                    };
                    ctx.locals.insert(arg.tok.val.clone(), var);
                    ctx.stack_ix -= 8;
                }
            }


            // Body
            if let Some(body) = func.children.get(1) {
                for stmt in &body.children {
                    self.generate_from_statement(stmt, &mut ctx);
                }
            } else {
                panic!("{} Error: Expected function body for `{}` but found nothing", func.tok.val_str(), func.tok.pos);
            }

            // Epilogue
            self.record_label(&mut bottom_label);
            self.instrs.push(Instruction::_label(&bottom_label));

            // On a local stack frame so we automatically deallocate the locals when returning
            self.instrs.append(&mut vec![
                Instruction::_copy_register_b_to_a(RegisterName::RSP, RegisterName::RBP),
                Instruction::_pop_stack(RegisterName::RBP),
                Instruction::_return_to_caller(),
            ]);
        }
    }

    // Calling Convention:
    //      Int and Ptr args in:
    //          RDI 
    //          RSI
    //          RDX 
    //          RCX 
    //          R8 
    //          R9
    //
    //      Excess args on stack
    //          Pushed in normal order, so args need to be pulled in right to left 
    //          from the stack.
    //          
    //          Maybe no stack arg passing for now.
    //
    //      Return value in:
    //          RAX 
    //          RAX + RDX (128 bit)

    fn generate_func_call(&mut self, call: &ParseNode, _ctx: &mut Context) {
        if call.children.len() > 6 {
            panic!("{} Error: Passing to many args to function `{}`. Currently there is support for up to 6 args", call.tok.pos, call.tok.val_str());
        }

        self.instrs.push(Instruction::_comment("Function Call"));

        // Calling convention puts first few args into registers
        for (i, reg) in REG_ORDER.iter().enumerate() {
            if i == call.children.len() {
                break;
            }
            self.instrs.push(Instruction::_pop_stack(*reg));
        }

        let label: Label = Label::new_named("func_", &call.tok.val);
        self.instrs.push(Instruction::_call_function(&label));

        if call.datatype != DataType::None {
            self.instrs.push(Instruction::_push_stack_register(RegisterName::RAX));
        }
    }

    fn generate_from_statement(&mut self, stmt: &ParseNode, ctx: &mut Context) {
        match stmt.kind {
            NodeType::VarDecl
            | NodeType::Continue
            | NodeType::Break
            | NodeType::Return
            | NodeType::FuncCall
            | NodeType::MMap
            | NodeType::MUnmap
            | NodeType::Exit
            | NodeType::DebugDump => {
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
                        let lhs: &ParseNode = lhs_group.children.first().expect("");
                        if lhs.kind == NodeType::Var { // Dereferencing a var
                            if let Some(var) = ctx.locals.get(&lhs.tok.val) {
                                self.instrs.append(&mut vec![
                                    Instruction::_pop_stack(RegisterName::RAX),
                                    Instruction::_copy_var_val_to_register(RegisterName::RBX, var.clone()), // address into rbx
                                    Instruction::_copy_register_a_to_adr_at_register_b(RegisterName::RAX, RegisterName::RBX),
                                ]);
                            } else {
                                panic!("{} Error: Attempting to assign undeclared variable `{}`", stmt.tok.pos, lhs.tok.val_str());
                            }
                        } else { // Dereferencing some expression
                            for node in &lhs.post_order() {
                                self.generate_from_node(node, ctx);
                            }
                            self.instrs.append(&mut vec![
                                Instruction::_pop_stack(RegisterName::RBX), // lhs
                                Instruction::_pop_stack(RegisterName::RAX), // rhs
                                Instruction::_copy_register_a_to_adr_at_register_b(RegisterName::RAX, RegisterName::RBX),
                            ]);
                        }
                    } else if lhs_group.kind == NodeType::Var {
                        if let Some(var) = ctx.locals.get(&lhs_group.tok.val) {
                            self.instrs.append(&mut vec![
                                Instruction::_pop_stack(RegisterName::RAX),
                                Instruction::_copy_register_to_var(var.clone(), RegisterName::RAX)
                            ]);
                        } else {
                            panic!("{} Error: Attempting to assign undeclared variable `{}`", stmt.tok.pos, lhs_group.tok.val_str());
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
                }
                self.instrs.push(Instruction::_pop_stack(RegisterName::RAX));
                self.instrs.push(Instruction::_jump_if_zero(RegisterName::RAX, &end_label));

                // Body
                let mut while_ctx: Context = Context::under(&stmt.tok, ctx);
                if let Some(body) = stmt.children.get(1) && body.kind != NodeType::Null {
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
                if let Some(decl) = stmt.children.first() && decl.kind != NodeType::Null {
                    self.instrs.push(Instruction::_comment("Decl"));
                    decl_tok = Some(&decl.tok);
                    for node in &decl.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                }

                // Init 
                if let Some(init) = stmt.children.get(1) && init.kind != NodeType::Null {
                    self.instrs.push(Instruction::_comment("Init"));
                    self.generate_from_statement(init, ctx);
                }

                let mut for_ctx: Context = Context::under(&stmt.tok, ctx);

                // Condition 
                self.record_label(&mut loop_label);
                self.instrs.push(Instruction::_label(&loop_label));
                if let Some(cond) = stmt.children.get(2) && cond.kind != NodeType::Null {
                    self.instrs.push(Instruction::_comment("Condition"));
                    for node in &cond.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                }
                self.instrs.push(Instruction::_pop_stack(RegisterName::RAX));
                self.instrs.push(Instruction::_jump_if_zero(RegisterName::RAX, &end_label));

                // Body
                if let Some(body) = stmt.children.get(4) && body.kind != NodeType::Null {
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

                if let Some(post) = stmt.children.get(3) && post.kind != NodeType::Null {
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
                if let Some(cond) = stmt.children.first() {
                    for node in &cond.post_order() {
                        self.generate_from_node(node, ctx);
                    }
                }
                self.instrs.push(Instruction::_pop_stack(RegisterName::RAX));

                if stmt.children.len() == 2 { // There is no else block
                    // Jump to end if condition fails
                    self.instrs.push(Instruction::_jump_if_zero(RegisterName::RAX, &end_label));

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

                    self.instrs.push(Instruction::_jump_if_zero(RegisterName::RAX, &else_label));

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
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_add_register_b_to_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpMinus => {
                        self.instrs.push(Instruction::_comment("BinOp Minus"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_sub_register_b_from_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpMul => {
                        self.instrs.push(Instruction::_comment("BinOp Mul"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_mul_register_a_by_b(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpDiv => {
                        self.instrs.push(Instruction::_comment("BinOp Div"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RCX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_div_rcx_by_rax_mangling_rdx(),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpEqual => {
                        self.instrs.push(Instruction::_comment("BinOp Eq"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_register_b_eq_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpNotEqual => {
                        self.instrs.push(Instruction::_comment("BinOp NEq"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_register_b_neq_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpGreaterThan => {
                        self.instrs.push(Instruction::_comment("BinOp GT"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_register_b_greater_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpLessThan => {
                        self.instrs.push(Instruction::_comment("BinOp LT"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_register_b_less_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpGreaterEqual => {
                        self.instrs.push(Instruction::_comment("BinOp GE"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_register_b_greater_eq_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpLessEqual => {
                        self.instrs.push(Instruction::_comment("BinOp LE"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_register_b_less_eq_a(RegisterName::RAX, RegisterName::RBX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpLogicalOr => {
                        self.instrs.push(Instruction::_comment("BinOp Logical Or"));

                        let mut rhs_label: Label = Label::new_at(".rhs_", &node.tok.pos());
                        let mut end_label: Label = Label::new_at(".end_", &node.tok.pos());

                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_jump_if_zero(RegisterName::RAX, &rhs_label),
                            Instruction::_jump_to_label(&end_label)
                        ]);

                        self.record_label(&mut rhs_label);
                        self.instrs.push(Instruction::_label(&rhs_label));
                        
                        self.instrs.push(Instruction::_literal_int_a_neq_register_b(0, RegisterName::RBX));
                        self.instrs.push(Instruction::_copy_register_b_to_a(RegisterName::RAX, RegisterName::RBX));

                        self.record_label(&mut end_label);
                        self.instrs.push(Instruction::_label(&end_label));

                        self.instrs.push(Instruction::_push_stack_register(RegisterName::RAX));
                    },
                    TokenType::OpLogicalAnd => {
                        self.instrs.push(Instruction::_comment("BinOp Logical And"));

                        let mut rhs_label: Label = Label::new_at(".rhs_", &node.tok.pos());
                        let mut end_label: Label = Label::new_at(".end_", &node.tok.pos());

                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_pop_stack(RegisterName::RBX),
                            Instruction::_jump_if_zero(RegisterName::RAX, &end_label),
                            Instruction::_jump_to_label(&rhs_label)
                        ]);

                        self.record_label(&mut rhs_label);
                        self.instrs.push(Instruction::_label(&rhs_label));
                        
                        self.instrs.push(Instruction::_literal_int_a_neq_register_b(0, RegisterName::RBX));

                        self.record_label(&mut end_label);
                        self.instrs.push(Instruction::_label(&end_label));

                        self.instrs.push(Instruction::_push_stack_register(RegisterName::RAX));
                    },
                    _ => unreachable!(),
                }
            },
            NodeType::UnaryOp => {
                match node.tok.kind {
                    TokenType::OpMinus => {
                        self.instrs.push(Instruction::_comment("UnOp Minus"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_negate_register(RegisterName::RAX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    TokenType::OpDereference => {
                        self.instrs.push(Instruction::_comment("UnOp Dereference"));
                        self.instrs.append(&mut vec![
                            Instruction::_pop_stack(RegisterName::RAX),
                            Instruction::_dereference_register(RegisterName::RAX),
                            Instruction::_push_stack_register(RegisterName::RAX),
                        ]);
                    },
                    _ => unreachable!(),
                }
            },
            NodeType::LiteralInt => {
                self.instrs.push(Instruction::_comment("Literal Int"));
                let lit: i64 = node.tok.val_str().parse::<i64>().unwrap();
                self.instrs.push(Instruction::_push_stack_literal_int(lit));
            },
            NodeType::Var => {
                self.instrs.push(Instruction::_comment("Var"));
                let var: Option<&Var> = ctx.locals.get(&node.tok.val);
                if let Some(var) = var {
                    self.instrs.push(Instruction::_copy_var_val_to_register(RegisterName::RAX, var.clone()));
                    self.instrs.push(Instruction::_push_stack_register(RegisterName::RAX));
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

                self.instrs.push(Instruction::_sub_literal_int_from_register(8, RegisterName::RSP));
            },
            NodeType::FuncCall
            | NodeType::MMap
            | NodeType::MUnmap
            | NodeType::Exit
            | NodeType::DebugDump => {
                self.generate_func_call(node, ctx);
            },
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
                self.instrs.push(Instruction::_pop_stack(RegisterName::RAX));
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
