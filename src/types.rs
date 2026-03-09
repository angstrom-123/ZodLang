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

    pub fn typecheck_ast(&mut self, ast: &ParseTree) {
        // Record the signatures of all functions first
        for func_decl in &ast.root.children {
            if let Some(args) = func_decl.children.first() {
                let sig: FunctionSignature = FunctionSignature {
                    arg_types: args.children.iter().map(|x| x.datatype.clone()).collect(),
                    ret_type: func_decl.datatype.clone(),
                };
                self.funcs.insert(func_decl.tok.val.clone(), sig);
            }
        }

        // Typecheck bodies of all functions
        for func_decl in &ast.root.children {
            self.typecheck_func_decl(func_decl);
        }
    }

    fn typecheck_func_decl(&mut self, func_decl: &ParseNode) {
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
            if let Some(body) = func_decl.children.get(1) {
                self.typecheck_block(body, &mut ctx);
            }
        } else {
            panic!("{} Error: Failed to read function signature for `{}`", 
                   func_decl.tok.pos, func_decl.tok.val_str());
        }
    }

    fn typecheck_block(&mut self, block: &ParseNode, ctx: &mut Context) {
        for stmt in &block.children {
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
        for node in &expr.post_order() {
            let node_typ: DataType = match node.kind {
                NodeType::LiteralInt => DataType::I64,
                NodeType::LiteralPointer => DataType::I64Ptr,
                NodeType::Var => ctx.get_var_type(node),
                NodeType::BinaryOp => {
                    let lhs: &ParseNode = node.children.first().unwrap();
                    let rhs: &ParseNode = node.children.last().unwrap();
                    let lhs_typ: DataType = self.typecheck_factor(lhs, ctx);
                    let rhs_typ: DataType = self.typecheck_factor(rhs, ctx);

                    if lhs_typ != rhs_typ {
                        panic!("{} Error: Unexpected mixed types in expression. Expected `{:?}` but got `{:?}`", 
                               node.tok.pos, lhs_typ, rhs_typ);
                    }
                    lhs_typ
                },
                NodeType::UnaryOp 
                | NodeType::FuncCall => self.typecheck_factor(node, ctx),
                _ => panic!("{} Error: Unexpected node to typecheck expression `{:?}`", 
                            node.tok.pos, node.kind)
            };

            if expr_typ == DataType::Unknown {
                expr_typ = node_typ.clone();
            }

            if expr_typ != node_typ.clone() {
                panic!("{} Error: Unexpected mixed types in expression. Expected `{:?}` but got `{:?}`", 
                       node.tok.pos, expr_typ, node_typ);
            }
        }

        expr_typ
    }

    fn typecheck_factor(&mut self, rhs: &ParseNode, ctx: &mut Context) -> DataType {
        match rhs.kind {
            NodeType::FuncCall => self.typecheck_func_call(rhs, ctx),
            NodeType::UnaryOp => {
                let child: &ParseNode = rhs.children.first().unwrap();
                match rhs.tok.kind {
                    TokenType::OpMinus => self.typecheck_expr(child, ctx),
                    TokenType::OpDereference => {
                        let typ: Option<DataType> = DataType::base_type(&self.typecheck_expr(child, ctx));
                        match typ {
                            Some(typ) => typ,
                            None => panic!("{} Error: Cannot dereferene a `{:?}`", child.tok.pos, typ)
                        }
                    },
                    _ => unreachable!()
                }
            },
            _ => self.typecheck_expr(rhs, ctx),
        }
    }

    fn typecheck_return(&mut self, returnn: &ParseNode, ctx: &mut Context) {
        let rhs: &ParseNode = returnn.children.first().unwrap();
        let rhs_typ: DataType = self.typecheck_factor(rhs, ctx);
        let ret_typ: DataType = self.get_func_sig(&ctx.outer_func).ret_type;
        if rhs_typ != ret_typ {
            panic!("{} Error: Invalid return type for function `{}`. Expected `{:?}` but got `{:?}`",
                   returnn.tok.pos, ctx.outer_func.tok.val_str(), ret_typ, rhs_typ);
        }
    }

    fn typecheck_var_decl(&mut self, var_decl: &ParseNode, ctx: &mut Context) {
        ctx.locals.insert(var_decl.tok.val.clone(), var_decl.datatype.clone());
    }

    fn typecheck_assign(&mut self, assign: &ParseNode, ctx: &mut Context) {
        let lhs: &ParseNode = assign.children.first().unwrap();
        let lhs_typ: DataType = match lhs.kind {
            NodeType::UnaryOp => {
                let child: &ParseNode = lhs.children.first().unwrap();
                match lhs.tok.kind {
                    TokenType::OpMinus => self.typecheck_expr(child, ctx),
                    TokenType::OpDereference => {
                        let typ: Option<DataType> = DataType::base_type(&self.typecheck_expr(child, ctx));
                        match typ {
                            Some(typ) => typ,
                            None => panic!("{} Error: Cannot dereferene a `{:?}`", child.tok.pos, typ)
                        }
                    },
                    _ => unreachable!()
                }
            },
            _ => self.typecheck_expr(lhs, ctx)
        };

        let rhs: &ParseNode = assign.children.last().unwrap();
        let rhs_typ: DataType = self.typecheck_factor(rhs, ctx);

        if lhs_typ != rhs_typ {
            panic!("{} Error: Cannot assign `{:?}` to `{:?}`", assign.tok.pos, rhs_typ, lhs_typ);
        }
    }

    fn typecheck_func_call(&mut self, func_call: &ParseNode, ctx: &mut Context) -> DataType {
        let sig: FunctionSignature = self.get_func_sig(func_call);

        if sig.arg_types.len() != func_call.children.len() {
            panic!("{} Error: Incorrect number of arguments for function `{}`. Expected {} but got {}", 
                   func_call.tok.pos, func_call.tok.val_str(), sig.arg_types.len(), func_call.children.len());
        }

        // Args parsed in reverse order, reversed back here to check against signature 
        for (node, expected) in func_call.children.iter().rev().zip(sig.arg_types.iter()) {
            let arg_typ: DataType = self.typecheck_factor(node, ctx);
            if arg_typ != *expected {
                panic!("{} Error: Incorrect argument type for function `{}`. Expected `{:?}` but got `{:?}`",
                       func_call.tok.pos, func_call.tok.val_str(), expected, arg_typ);
            }
        }

        sig.ret_type
    }

    fn typecheck_conditional(&mut self, conditional: &ParseNode, ctx: &mut Context) {
        let cond: &ParseNode = conditional.children.first().unwrap();
        let cond_typ: DataType = self.typecheck_factor(cond, ctx);
        if !matches!(cond_typ, DataType::I64) {
            panic!("{} Error: Invalid type for `if` condition. Expected `{:?}` but got `{:?}`",
                   cond.tok.pos, DataType::I64, cond_typ);
        }

        let if_body: &ParseNode = conditional.children.get(1).unwrap();
        self.typecheck_block(if_body, ctx);

        // This is an optional field, so we don't care if it fails
        if let Some(else_body) = conditional.children.get(2) {
            self.typecheck_block(else_body, ctx);
        }
    }

    fn typecheck_for_loop(&mut self, for_loop: &ParseNode, ctx: &mut Context) {
        // Any of the fields except the body could be null
        let decl: &ParseNode = for_loop.children.first().unwrap();
        if !decl.is_null() {
            self.typecheck_var_decl(decl, ctx);
        }

        let init: &ParseNode = for_loop.children.get(1).unwrap();
        if !init.is_null() {
            self.typecheck_assign(init, ctx);
        }

        let cond: &ParseNode = for_loop.children.get(2).unwrap();
        if !cond.is_null() {
            let cond_typ: DataType = self.typecheck_factor(cond, ctx);
            if !matches!(cond_typ, DataType::I64) {
                panic!("{} Error: Invalid type for `for` condition. Expected `{:?}` but got `{:?}`",
                       cond.tok.pos, DataType::I64, cond_typ);
            }
        }

        let post: &ParseNode = for_loop.children.get(3).unwrap();
        if !post.is_null() {
            match post.kind {
                NodeType::Assign => self.typecheck_assign(post, ctx),
                NodeType::FuncCall => _ = self.typecheck_func_call(post, ctx),
                _ => unreachable!("")
            }
        }

        let body: &ParseNode = for_loop.children.get(4).unwrap();
        self.typecheck_block(body, ctx);
    }

    fn typecheck_while_loop(&mut self, while_loop: &ParseNode, ctx: &mut Context) {
        let cond: &ParseNode = while_loop.children.first().unwrap();
        if !cond.is_null() {
            let cond_typ: DataType = self.typecheck_factor(cond, ctx);
            if !matches!(cond_typ, DataType::I64) {
                panic!("{} Error: Invalid type for `while` condition. Expected `{:?}` but got `{:?}`",
                       cond.tok.pos, DataType::I64, cond_typ);
            }
        }

        let body: &ParseNode = while_loop.children.last().unwrap();
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
