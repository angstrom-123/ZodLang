use std::collections::HashMap;

use crate::lex::{TokKind, Tok};
use crate::parse::{NodeKind, Node, AST};

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
pub enum Datatype {
    None,
    Unknown,
    Void,
    I64,
    I64Ptr,
    Chr,
    ChrPtr,
    AnyPtr,
}
impl Datatype {
    pub fn from_token(tok: &Tok) -> Datatype {
        match tok.kind {
            TokKind::TypeVoid     => Datatype::Void,
            TokKind::TypeInt64    => Datatype::I64,
            TokKind::TypeInt64Ptr => Datatype::I64Ptr,
            TokKind::TypeChr      => Datatype::Chr,
            TokKind::TypeChrPtr   => Datatype::ChrPtr,
            TokKind::TypeAnyPtr   => Datatype::AnyPtr,
            _ => panic!("{} Error: Failed to convert token `{}` to datatype", tok.pos, tok.val_str())
        }
    }

    pub fn size(&self) -> i64 {
        match self {
            Datatype::I64    => 8,
            Datatype::I64Ptr => 8,
            Datatype::Chr    => 1,
            Datatype::ChrPtr => 8,
            Datatype::AnyPtr => 8,
            _ => -1,
        }
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self, Datatype::I64Ptr | Datatype::ChrPtr | Datatype::AnyPtr)
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Datatype::I64 | Datatype::Chr)
    }

    pub fn is_compatible(&self, other: &Datatype) -> bool {
        match self {
            Datatype::I64    => matches!(other, Datatype::AnyPtr | Datatype::I64 | Datatype::Chr),
            Datatype::I64Ptr => matches!(other, Datatype::AnyPtr | Datatype::I64Ptr | Datatype::I64),
            Datatype::Chr    => matches!(other, Datatype::Chr | Datatype::I64),
            Datatype::ChrPtr => matches!(other, Datatype::AnyPtr | Datatype::ChrPtr | Datatype::I64),
            Datatype::AnyPtr => matches!(other, Datatype::AnyPtr | Datatype::ChrPtr | Datatype::I64Ptr | Datatype::I64),
            Datatype::Void   => matches!(other, Datatype::Void),
            _ => false
        }
    }

    pub fn base_type(&self) -> Option<Datatype> {
        match self {
            Datatype::I64Ptr => Some(Datatype::I64),
            Datatype::ChrPtr => Some(Datatype::Chr),
            _ => None
        }
    }
}

struct Context {
    locals: HashMap<Vec<u8>, Datatype>,
    outer_func: Node,
}
impl Context {
    pub fn get_var_type(&self, node: &Node) -> Datatype {
        let res: Option<&Datatype> = self.locals.get(&node.tok.val);
        if let Some(res) = res {
            if *res == Datatype::Unknown {
                panic!("{} Error: Unknown type of variable `{}`", node.tok.pos, node.tok.val_str());
            }
            res.clone()
        } else {
            panic!("{} Error: Could not find variable `{}`", node.tok.pos, node.tok.val_str());
        }
    }
}

#[derive(Clone)]
struct FunctionSignature {  
    ret_type: Datatype,
    arg_types: Vec<Datatype>,
}

pub struct Analyser {
    funcs: HashMap<Vec<u8>, FunctionSignature>,
}
impl Default for Analyser {
    fn default() -> Self {
        Self::new()
    }
}
impl Analyser {
    pub fn new() -> Self {
        Analyser {
            funcs: HashMap::new(),
        }
    }

    pub fn typecheck_ast(&mut self, ast: &mut AST) {
        // Record the signatures of all functions first
        for top_lvl_stmt in &ast.root.children {
            if top_lvl_stmt.kind == NodeKind::FuncDecl && let Some(args) = top_lvl_stmt.children.first() {
                let sig: FunctionSignature = FunctionSignature {
                    arg_types: args.children.iter().map(|x| x.datatype.clone()).collect(),
                    ret_type: top_lvl_stmt.datatype.clone(),
                };
                self.funcs.insert(top_lvl_stmt.tok.val.clone(), sig);
            }
        }

        // Typecheck bodies of all functions
        for top_lvl_stmt in &mut ast.root.children {
            if top_lvl_stmt.kind == NodeKind::FuncDecl {
                self.typecheck_func_decl(top_lvl_stmt);
            }
        }
    }

    fn typecheck_func_decl(&mut self, func_decl: &mut Node) {
        if let Some(args) = func_decl.children.first() {
            // Record types of vars in signature
            let mut ctx: Context = Context {
                locals: HashMap::new(),
                outer_func: func_decl.clone(),
            };
            for arg in &args.children {
                ctx.locals.insert(arg.tok.val.clone(), arg.datatype.clone());
            }

            // Typecheck body
            if let Some(body) = func_decl.children.get_mut(1) {
                self.typecheck_block(body, &mut ctx);
            }
        } else {
            panic!("{} Error: Failed to read function signature for `{}`", 
                   func_decl.tok.pos, func_decl.tok.val_str());
        }
    }

    fn typecheck_block(&mut self, block: &mut Node, ctx: &mut Context) {
        for stmt in &mut block.children {
            match stmt.kind {
                NodeKind::VarDecl     => self.typecheck_var_decl(stmt, ctx),
                NodeKind::Assign      => self.typecheck_assign(stmt, ctx),
                NodeKind::FuncCall    => _ = self.typecheck_func_call(stmt, ctx),
                NodeKind::Syscall     => _ = self.typecheck_syscall(stmt, ctx),
                NodeKind::Conditional => self.typecheck_conditional(stmt, ctx),
                NodeKind::ForLoop     => self.typecheck_for_loop(stmt, ctx),
                NodeKind::WhileLoop   => self.typecheck_while_loop(stmt, ctx),
                NodeKind::Return      => self.typecheck_return(stmt, ctx),
                _ => {},
            }
        }
    }

    fn typecheck_expr(&mut self, expr: &Node, ctx: &mut Context) -> Datatype {
        let mut expr_typ: Datatype = Datatype::Unknown;
        for node in &mut expr.post_order() {
            let node_typ: Datatype = match node.kind {
                NodeKind::LiteralInt => Datatype::I64,
                NodeKind::LiteralChar => Datatype::Chr,
                NodeKind::LiteralString => Datatype::ChrPtr,
                NodeKind::Var => ctx.get_var_type(node),
                NodeKind::UnaryOp 
                | NodeKind::BinaryOp 
                | NodeKind::FuncCall 
                | NodeKind::Syscall => self.typecheck_factor(node, ctx),
                _ => panic!("{} Error: Unexpected node to typecheck expression `{:?}`", 
                            node.tok.pos, node.kind)
            };

            node.datatype = node_typ.clone();

            if expr_typ == Datatype::Unknown {
                expr_typ = node_typ.clone();
            }

            if !expr_typ.is_compatible(&node_typ) {
                panic!("{} Error: Unexpected mixed types in expression. Expected `{:?}` but got `{:?}`", 
                       node.tok.pos, expr_typ, node_typ);
            }
        }
        if expr_typ == Datatype::Unknown {
            panic!("{} Error: Could not find type of expression.", expr.tok.pos);
        }

        expr_typ
    }

    fn typecheck_factor(&mut self, node: &mut Node, ctx: &mut Context) -> Datatype {
        let typ: Datatype = match node.kind {
            NodeKind::FuncCall => self.typecheck_func_call(node, ctx),
            NodeKind::Syscall => self.typecheck_syscall(node, ctx),
            NodeKind::BinaryOp => match node.tok.kind {
                TokKind::Subscript => {
                    let rhs: &mut Node = node.children.last_mut().unwrap();
                    rhs.datatype = self.typecheck_factor(rhs, ctx);
                    if !rhs.datatype.is_numeric() {
                        panic!("{} Error: Subscripts can only be integers. Expected int but got `{:?}`",
                               rhs.tok.pos, rhs.datatype);
                    }

                    let lhs: &mut Node = node.children.first_mut().unwrap();
                    lhs.datatype = ctx.get_var_type(lhs);
                    match lhs.datatype.base_type() {
                        Some(lhs_typ) => lhs_typ,
                        None => panic!("{} Error: Cannot subscript a `{:?}`", 
                                       lhs.tok.pos, lhs.datatype)
                    }
                },
                _ => {
                    let lhs: &mut Node = node.children.first_mut().unwrap();
                    lhs.datatype = self.typecheck_factor(lhs, ctx);
                    let ltyp: Datatype = lhs.datatype.clone();

                    let rhs: &mut Node = node.children.last_mut().unwrap();
                    rhs.datatype = self.typecheck_factor(rhs, ctx);

                    if !ltyp.is_compatible(&rhs.datatype) {
                        panic!("{} Error: Unexpected mixed types in expression. Expected `{:?}` but got `{:?}`", 
                               node.tok.pos, ltyp, rhs.datatype);
                    }

                    ltyp
                },
            },
            NodeKind::UnaryOp => {
                let child: &mut Node = node.children.first_mut().unwrap();
                // match node.tok.kind {
                match node.tok.kind {
                    TokKind::Minus => self.typecheck_factor(child, ctx),
                    TokKind::Deref => {
                        let typ: Datatype = self.typecheck_factor(child, ctx);
                        match typ.base_type() {
                            Some(typ) => typ,
                            None => panic!("{} Error: Cannot dereferene a `{:?}`", child.tok.pos, typ)
                        }
                    },
                    _ => unreachable!()
                }
            },
            _ => self.typecheck_expr(node, ctx),
        };
        node.datatype = typ.clone();
        typ
    }

    fn typecheck_return(&mut self, returnn: &mut Node, ctx: &mut Context) {
        let rhs: &mut Node = returnn.children.first_mut().unwrap();
        let rhs_typ: Datatype = self.typecheck_factor(rhs, ctx);
        let ret_typ: Datatype = self.get_func_sig(&ctx.outer_func).ret_type;
        if !rhs_typ.is_compatible(&ret_typ) {
            panic!("{} Error: Invalid return type for function `{}`. Expected `{:?}` but got `{:?}`",
                   returnn.tok.pos, ctx.outer_func.tok.val_str(), ret_typ, rhs_typ);
        }
    }

    fn typecheck_var_decl(&mut self, var_decl: &Node, ctx: &mut Context) {
        ctx.locals.insert(var_decl.tok.val.clone(), var_decl.datatype.clone());
    }

    fn typecheck_assign(&mut self, assign: &mut Node, ctx: &mut Context) {
        let lhs: &mut Node = assign.children.first_mut().unwrap();
        let lhs_typ: Datatype = match lhs.kind {
            NodeKind::UnaryOp => {
                let child: &mut Node = lhs.children.first_mut().unwrap();
                match lhs.tok.kind {
                    TokKind::Minus => self.typecheck_expr(child, ctx),
                    TokKind::Deref => {
                        let child_typ: Datatype = self.typecheck_expr(child, ctx);
                        child.datatype = child_typ.clone();
                        let typ: Option<Datatype> = child_typ.base_type();
                        match typ {
                            Some(typ) => typ,
                            None => panic!("{} Error: Cannot dereferene a `{:?}`", child.tok.pos, typ)
                        }
                    },
                    _ => unreachable!()
                }
            },
            NodeKind::BinaryOp => {
                match lhs.tok.kind {
                    TokKind::Subscript => {
                        let r_child: &mut Node = lhs.children.last_mut().unwrap();
                        self.typecheck_factor(r_child, ctx);

                        let l_child: &mut Node = lhs.children.first_mut().unwrap();
                        self.typecheck_factor(l_child, ctx);
                        let typ: Option<Datatype> = l_child.datatype.base_type();
                        match typ {
                            Some(typ) => typ,
                            None => panic!("{} Error: Cannot subscript a `{:?}`", l_child.tok.pos, l_child.datatype)
                        }
                    },
                    _ => unreachable!()
                }
            },
            _ => self.typecheck_expr(lhs, ctx)
        };
        lhs.datatype = lhs_typ.clone();

        let rhs: &mut Node = assign.children.last_mut().unwrap();
        let rhs_typ: Datatype = self.typecheck_factor(rhs, ctx);
        rhs.datatype = rhs_typ.clone();

        if !lhs_typ.is_compatible(&rhs_typ) {
            panic !("{} Error: Cannot assign `{:?}` to `{:?}`", assign.tok.pos, rhs_typ, lhs_typ);
        }         

        assign.datatype = lhs_typ.clone();
    }             
                  
    fn typecheck_syscall(&mut self, syscall: &mut Node, ctx: &mut Context) -> Datatype {
        for arg in &mut syscall.children {
            let arg_typ: Datatype = self.typecheck_factor(arg, ctx);
            if !arg_typ.is_numeric() && !arg_typ.is_ptr() {
                panic!("{} Error: Syscalls only accept integers and pointers as arguments", 
                       syscall.tok.pos);
            }
        }

        Datatype::I64
    }

    fn typecheck_func_call(&mut self, func_call: &mut Node, ctx: &mut Context) -> Datatype {
        let sig: FunctionSignature = self.get_func_sig(func_call);

        if sig.arg_types.len() != func_call.children.len() {
            panic!("{} Error: Incorrect number of arguments for function `{}`. Expected {} but got {}", 
                   func_call.tok.pos, func_call.tok.val_str(), sig.arg_types.len(), func_call.children.len());
        }

        // Args parsed in reverse order, reversed back here to check against signature 
        for i in 0..func_call.children.len() {
            let arg_ix: usize = func_call.children.len() - 1 - i;
            let arg_node: &mut Node = func_call.children.get_mut(arg_ix).unwrap();
            let arg_typ: Datatype = self.typecheck_factor(arg_node, ctx);
            let expected_typ: &Datatype = sig.arg_types.get(i).unwrap();
            // if arg_typ != *expected_typ {
            if !arg_typ.is_compatible(expected_typ) {
                panic!("{} Error: Incorrect argument type for function `{}`. Expected `{:?}` but got `{:?}`",
                       func_call.tok.pos, func_call.tok.val_str(), expected_typ, arg_typ);
            }
        }

        func_call.datatype = sig.ret_type.clone();
        sig.ret_type
    }

    fn typecheck_conditional(&mut self, conditional: &mut Node, ctx: &mut Context) {
        let cond: &mut Node = conditional.children.first_mut().unwrap();
        let cond_typ: Datatype = self.typecheck_factor(cond, ctx);
        if !cond_typ.is_numeric() {
            panic!("{} Error: Invalid type for `if` condition. Expected int but got `{:?}`",
                   cond.tok.pos, cond_typ);
        }

        let if_body: &mut Node = conditional.children.get_mut(1).unwrap();
        self.typecheck_block(if_body, ctx);

        if let Some(else_body) = conditional.children.get_mut(2) {
            self.typecheck_block(else_body, ctx);
        }
    }

    fn typecheck_for_loop(&mut self, for_loop: &mut Node, ctx: &mut Context) {
        // Any of the fields except the body could be null
        let decl: &Node = for_loop.children.first().unwrap();
        if !decl.is_null() {
            self.typecheck_var_decl(decl, ctx);
        }

        let init: &mut Node = for_loop.children.get_mut(1).unwrap();
        if !init.is_null() {
            self.typecheck_assign(init, ctx);
        }

        let cond: &mut Node = for_loop.children.get_mut(2).unwrap();
        if !cond.is_null() {
            let cond_typ: Datatype = self.typecheck_factor(cond, ctx);
            if !cond_typ.is_numeric() {
                panic!("{} Error: Invalid type for `for` condition. Expected int but got `{:?}`",
                       cond.tok.pos, cond_typ);
            }
        }

        let post: &mut Node = for_loop.children.get_mut(3).unwrap();
        if !post.is_null() {
            match post.kind {
                NodeKind::Assign => self.typecheck_assign(post, ctx),
                NodeKind::FuncCall => _ = self.typecheck_func_call(post, ctx),
                NodeKind::Syscall => _ = self.typecheck_syscall(post, ctx),
                _ => unreachable!("")
            }
        }

        let body: &mut Node = for_loop.children.get_mut(4).unwrap();
        self.typecheck_block(body, ctx);
    }

    fn typecheck_while_loop(&mut self, while_loop: &mut Node, ctx: &mut Context) {
        let cond: &mut Node = while_loop.children.first_mut().unwrap();
        if !cond.is_null() {
            let cond_typ: Datatype = self.typecheck_factor(cond, ctx);
            if !cond_typ.is_numeric() {
                panic!("{} Error: Invalid type for `while` condition. Expected int but got `{:?}`",
                       cond.tok.pos, cond_typ);
            }
        }

        let body: &mut Node = while_loop.children.last_mut().unwrap();
        self.typecheck_block(body, ctx);
    }

    fn get_func_sig(&self, node: &Node) -> FunctionSignature {
        let res: Option<&FunctionSignature> = self.funcs.get(&node.tok.val);
        if let Some(res) = res {
            res.clone()
        } else {
            panic!("{} Error: Could not find function `{}`", node.tok.pos, node.tok.val_str());
        }
    }
}
