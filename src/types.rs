use std::collections::HashMap;

use crate::{lexer::TokenType, parser::{DataType, NodeType, ParseNode, ParseTree}};

struct Context {
    locals: HashMap<Vec<u8>, DataType>,
    outer_func: ParseNode,
}
impl Context {
    pub fn get_var_type(&self, node: &ParseNode) -> DataType {
        let res: Option<&DataType> = self.locals.get(&node.tok.val);
        if let Some(res) = res {
            if *res == DataType::Unknown {
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
    ret_type: DataType,
    arg_types: Vec<DataType>,
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

    pub fn typecheck_ast(&mut self, ast: &mut ParseTree) {
        // Record the signatures of all functions first
        for top_lvl_stmt in &ast.root.children {
            if top_lvl_stmt.kind == NodeType::FuncDecl && let Some(args) = top_lvl_stmt.children.first() {
                let sig: FunctionSignature = FunctionSignature {
                    arg_types: args.children.iter().map(|x| x.datatype.clone()).collect(),
                    ret_type: top_lvl_stmt.datatype.clone(),
                };
                self.funcs.insert(top_lvl_stmt.tok.val.clone(), sig);
            }
        }

        // Typecheck bodies of all functions
        for top_lvl_stmt in &mut ast.root.children {
            if top_lvl_stmt.kind == NodeType::FuncDecl {
                self.typecheck_func_decl(top_lvl_stmt);
            }
        }
    }

    fn typecheck_func_decl(&mut self, func_decl: &mut ParseNode) {
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

    fn typecheck_block(&mut self, block: &mut ParseNode, ctx: &mut Context) {
        for stmt in &mut block.children {
            match stmt.kind {
                NodeType::VarDecl     => self.typecheck_var_decl(stmt, ctx),
                NodeType::Assign      => self.typecheck_assign(stmt, ctx),
                NodeType::FuncCall    => _ = self.typecheck_func_call(stmt, ctx),
                NodeType::Conditional => self.typecheck_conditional(stmt, ctx),
                NodeType::ForLoop     => self.typecheck_for_loop(stmt, ctx),
                NodeType::WhileLoop   => self.typecheck_while_loop(stmt, ctx),
                NodeType::Return      => self.typecheck_return(stmt, ctx),
                _ => {},
            }
        }
    }

    fn typecheck_expr(&mut self, expr: &ParseNode, ctx: &mut Context) -> DataType {
        let mut expr_typ: DataType = DataType::Unknown;
        for node in &mut expr.post_order() {
            let node_typ: DataType = match node.kind {
                NodeType::LiteralInt => DataType::I64,
                NodeType::LiteralChar => DataType::Chr,
                NodeType::LiteralString => DataType::ChrPtr,
                NodeType::Var => ctx.get_var_type(node),
                NodeType::UnaryOp 
                | NodeType::BinaryOp 
                | NodeType::FuncCall => self.typecheck_factor(node, ctx),
                _ => panic!("{} Error: Unexpected node to typecheck expression `{:?}`", 
                            node.tok.pos, node.kind)
            };

            node.datatype = node_typ.clone();

            if expr_typ == DataType::Unknown {
                expr_typ = node_typ.clone();
            }

            if !expr_typ.is_compatible(&node_typ) {
                panic!("{} Error: Unexpected mixed types in expression. Expected `{:?}` but got `{:?}`", 
                       node.tok.pos, expr_typ, node_typ);
            }
        }
        if expr_typ == DataType::Unknown {
            panic!("{} Error: Could not find type of expression.", expr.tok.pos);
        }

        expr_typ
    }

    fn typecheck_factor(&mut self, node: &mut ParseNode, ctx: &mut Context) -> DataType {
        let typ: DataType = match node.kind {
            NodeType::FuncCall => self.typecheck_func_call(node, ctx),
            NodeType::Syscall => self.typecheck_syscall(node, ctx),
            NodeType::BinaryOp => match node.tok.kind {
                TokenType::OpSubscript => {
                    let rhs: &mut ParseNode = node.children.last_mut().unwrap();
                    rhs.datatype = self.typecheck_factor(rhs, ctx);
                    if !rhs.datatype.is_int() {
                        panic!("{} Error: Subscripts can only be integers. Expected int but got `{:?}`",
                               rhs.tok.pos, rhs.datatype);
                    }

                    let lhs: &mut ParseNode = node.children.first_mut().unwrap();
                    lhs.datatype = ctx.get_var_type(lhs);
                    match lhs.datatype.base_type() {
                        Some(lhs_typ) => lhs_typ,
                        None => panic!("{} Error: Cannot subscript a `{:?}`", 
                                       lhs.tok.pos, lhs.datatype)
                    }
                },
                _ => {
                    let lhs: &mut ParseNode = node.children.first_mut().unwrap();
                    lhs.datatype = self.typecheck_factor(lhs, ctx);
                    let ltyp: DataType = lhs.datatype.clone();

                    let rhs: &mut ParseNode = node.children.last_mut().unwrap();
                    rhs.datatype = self.typecheck_factor(rhs, ctx);

                    if !ltyp.is_compatible(&rhs.datatype) {
                        panic!("{} Error: Unexpected mixed types in expression. Expected `{:?}` but got `{:?}`", 
                               node.tok.pos, ltyp, rhs.datatype);
                    }

                    if matches!(node.tok.kind, TokenType::OpEqual | TokenType::OpNotEqual | 
                                               TokenType::OpGreaterThan | TokenType::OpGreaterEqual | 
                                               TokenType::OpLessThan | TokenType::OpLessEqual) {
                        DataType::I64
                    } else {
                        ltyp
                    }
                },
            },
            NodeType::UnaryOp => {
                let child: &mut ParseNode = node.children.first_mut().unwrap();
                // match node.tok.kind {
                match node.tok.kind {
                    TokenType::OpMinus => self.typecheck_factor(child, ctx),
                    TokenType::OpDereference => {
                        let typ: DataType = self.typecheck_factor(child, ctx);
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

    fn typecheck_return(&mut self, returnn: &mut ParseNode, ctx: &mut Context) {
        let rhs: &mut ParseNode = returnn.children.first_mut().unwrap();
        let rhs_typ: DataType = self.typecheck_factor(rhs, ctx);
        let ret_typ: DataType = self.get_func_sig(&ctx.outer_func).ret_type;
        if !rhs_typ.is_compatible(&ret_typ) {
            panic!("{} Error: Invalid return type for function `{}`. Expected `{:?}` but got `{:?}`",
                   returnn.tok.pos, ctx.outer_func.tok.val_str(), ret_typ, rhs_typ);
        }
    }

    fn typecheck_var_decl(&mut self, var_decl: &ParseNode, ctx: &mut Context) {
        ctx.locals.insert(var_decl.tok.val.clone(), var_decl.datatype.clone());
    }

    fn typecheck_assign(&mut self, assign: &mut ParseNode, ctx: &mut Context) {
        let lhs: &mut ParseNode = assign.children.first_mut().unwrap();
        let lhs_typ: DataType = match lhs.kind {
            NodeType::UnaryOp => {
                let child: &mut ParseNode = lhs.children.first_mut().unwrap();
                match lhs.tok.kind {
                    TokenType::OpMinus => self.typecheck_expr(child, ctx),
                    TokenType::OpDereference => {
                        let child_typ: DataType = self.typecheck_expr(child, ctx);
                        child.datatype = child_typ.clone();
                        let typ: Option<DataType> = child_typ.base_type();
                        match typ {
                            Some(typ) => typ,
                            None => panic!("{} Error: Cannot dereferene a `{:?}`", child.tok.pos, typ)
                        }
                    },
                    _ => unreachable!()
                }
            },
            NodeType::BinaryOp => {
                match lhs.tok.kind {
                    TokenType::OpSubscript => {
                        let r_child: &mut ParseNode = lhs.children.last_mut().unwrap();
                        self.typecheck_factor(r_child, ctx);

                        let l_child: &mut ParseNode = lhs.children.first_mut().unwrap();
                        self.typecheck_factor(l_child, ctx);
                        let typ: Option<DataType> = l_child.datatype.base_type();
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

        let rhs: &mut ParseNode = assign.children.last_mut().unwrap();
        let rhs_typ: DataType = self.typecheck_factor(rhs, ctx);
        rhs.datatype = rhs_typ.clone();

        if !lhs_typ.is_assignable(&rhs_typ) {
            panic !("{} Error: Cannot assign `{:?}` to `{:?}`", assign.tok.pos, rhs_typ, lhs_typ);
        }         

        assign.datatype = lhs_typ.clone();
    }             
                  
    fn typecheck_syscall(&mut self, syscall: &mut ParseNode, ctx: &mut Context) -> DataType {
        for arg in &mut syscall.children {
            let arg_typ: DataType = self.typecheck_factor(arg, ctx);
            if !arg_typ.is_int() && !arg_typ.is_ptr() {
                panic!("{} Error: Syscalls only accept integers and pointers as arguments", 
                       syscall.tok.pos);
            }
        }

        DataType::I64
    }

    fn typecheck_func_call(&mut self, func_call: &mut ParseNode, ctx: &mut Context) -> DataType {
        let sig: FunctionSignature = self.get_func_sig(func_call);

        if sig.arg_types.len() != func_call.children.len() {
            panic!("{} Error: Incorrect number of arguments for function `{}`. Expected {} but got {}", 
                   func_call.tok.pos, func_call.tok.val_str(), sig.arg_types.len(), func_call.children.len());
        }

        // Args parsed in reverse order, reversed back here to check against signature 
        for i in 0..func_call.children.len() {
            let arg_ix: usize = func_call.children.len() - 1 - i;
            let arg_node: &mut ParseNode = func_call.children.get_mut(arg_ix).unwrap();
            let arg_typ: DataType = self.typecheck_factor(arg_node, ctx);
            let expected_typ: &DataType = sig.arg_types.get(i).unwrap();
            // if arg_typ != *expected_typ {
            if !arg_typ.is_compatible(expected_typ) {
                panic!("{} Error: Incorrect argument type for function `{}`. Expected `{:?}` but got `{:?}`",
                       func_call.tok.pos, func_call.tok.val_str(), expected_typ, arg_typ);
            }
        }

        func_call.datatype = sig.ret_type.clone();
        sig.ret_type
    }

    fn typecheck_conditional(&mut self, conditional: &mut ParseNode, ctx: &mut Context) {
        let cond: &mut ParseNode = conditional.children.first_mut().unwrap();
        let cond_typ: DataType = self.typecheck_factor(cond, ctx);
        if !cond_typ.is_int() {
            panic!("{} Error: Invalid type for `if` condition. Expected int but got `{:?}`",
                   cond.tok.pos, cond_typ);
        }

        let if_body: &mut ParseNode = conditional.children.get_mut(1).unwrap();
        self.typecheck_block(if_body, ctx);

        if let Some(else_body) = conditional.children.get_mut(2) {
            self.typecheck_block(else_body, ctx);
        }
    }

    fn typecheck_for_loop(&mut self, for_loop: &mut ParseNode, ctx: &mut Context) {
        // Any of the fields except the body could be null
        let decl: &ParseNode = for_loop.children.first().unwrap();
        if !decl.is_null() {
            self.typecheck_var_decl(decl, ctx);
        }

        let init: &mut ParseNode = for_loop.children.get_mut(1).unwrap();
        if !init.is_null() {
            self.typecheck_assign(init, ctx);
        }

        let cond: &mut ParseNode = for_loop.children.get_mut(2).unwrap();
        if !cond.is_null() {
            let cond_typ: DataType = self.typecheck_factor(cond, ctx);
            if !cond_typ.is_int() {
                panic!("{} Error: Invalid type for `for` condition. Expected int but got `{:?}`",
                       cond.tok.pos, cond_typ);
            }
        }

        let post: &mut ParseNode = for_loop.children.get_mut(3).unwrap();
        if !post.is_null() {
            match post.kind {
                NodeType::Assign => self.typecheck_assign(post, ctx),
                NodeType::FuncCall => _ = self.typecheck_func_call(post, ctx),
                _ => unreachable!("")
            }
        }

        let body: &mut ParseNode = for_loop.children.get_mut(4).unwrap();
        self.typecheck_block(body, ctx);
    }

    fn typecheck_while_loop(&mut self, while_loop: &mut ParseNode, ctx: &mut Context) {
        let cond: &mut ParseNode = while_loop.children.first_mut().unwrap();
        if !cond.is_null() {
            let cond_typ: DataType = self.typecheck_factor(cond, ctx);
            if !cond_typ.is_int() {
                panic!("{} Error: Invalid type for `while` condition. Expected int but got `{:?}`",
                       cond.tok.pos, cond_typ);
            }
        }

        let body: &mut ParseNode = while_loop.children.last_mut().unwrap();
        self.typecheck_block(body, ctx);
    }

    fn get_func_sig(&self, node: &ParseNode) -> FunctionSignature {
        let res: Option<&FunctionSignature> = self.funcs.get(&node.tok.val);
        if let Some(res) = res {
            res.clone()
        } else {
            panic!("{} Error: Could not find function `{}`", node.tok.pos, node.tok.val_str());
        }
    }
}
