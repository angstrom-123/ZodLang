// use std::collections::HashMap;
// use std::fs;
// use std::io::Write;
// use std::slice::Iter;
// use crate::lexer::{Token, TokenType};
// use crate::parser::{DataType, NodeType, ParseNode, ParseTree};
//
// #[derive(Debug)]
// #[derive(Clone)]
// #[derive(PartialEq)]
// struct Var {
//     offset: i64,
//     kind: DataType,
// }
//
// struct Context {
//     vars:       HashMap<Vec<u8>, Var>,
//     stack_ix:   i64,
//     loop_scope: Token,
// }
//
// fn generate_node_nasm_x86(f: &mut fs::File, ctx: &mut Context, node: &ParseNode) -> std::io::Result<()> {
//     match node.kind {
//         NodeType::Continue => {
//             if ctx.loop_scope.kind == TokenType::None {
//                 panic!("{} Error: Unexpected `continue` outside of breakable scope", node.tok.pos);
//             }
//             writeln!(f, "; --- Continue ---")?;
//             writeln!(f, "   jmp ._post_{}_{}", ctx.loop_scope.pos.row, ctx.loop_scope.pos.col)?;
//         },
//         NodeType::Break => {
//             if ctx.loop_scope.kind == TokenType::None {
//                 panic!("{} Error: Unexpected `break` outside of breakable scope", node.tok.pos);
//             }
//             writeln!(f, "; --- Break ---")?;
//             writeln!(f, "   jmp ._end_{}_{}", ctx.loop_scope.pos.row, ctx.loop_scope.pos.col)?;
//         },
//         NodeType::Null => {
//             writeln!(f, "; --- Null Node ---")?;
//         },
//         NodeType::FuncCall => {
//             writeln!(f, "; --- FuncCall {} ---", node.tok.val_str())?;
//             writeln!(f, "    call {}", node.tok.val_str())?;
//             writeln!(f, "; --- Deallocate params ---")?;
//             writeln!(f, "    add rsp, {}", node.children.len() * 8)?;
//             writeln!(f, "    push rax")?;
//         },
//         NodeType::LiteralInt => {
//             writeln!(f, "; --- Literal {} ---", node.tok.val_str())?;
//             writeln!(f, "    mov rax, {}", node.tok.val_str())?;
//             writeln!(f, "    push rax")?;
//         },
//         NodeType::DerefAssign => {
//             // TODO:
//             //      Change this and regular assign to accept a ParseNode for LHS
//             //      Then this whole node can be replaced by just putting a unary 
//             //      op dereference under the LHS of a regular assignment.
//             match ctx.vars.get(&node.tok.val) {
//                 None => panic!("{} Error: No such variable `{}` in local scope", node.tok.pos, node.tok.val_str()),
//                 Some(var) => {
//                     if var.kind != DataType::I64Ptr {
//                         panic!("{} Error: Expected type `Int64Ptr` but got {:?}", node.tok.pos, var.kind)
//                     }
//                     writeln!(f, "; --- Deref Assign {} ---", node.tok.val_str())?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    lea rbx, [rbp {}]", var.offset)?;
//                     writeln!(f, "    mov [rbx], rax")?;
//                 }
//             }
//         },
//         NodeType::Assign => {
//             match ctx.vars.get(&node.tok.val) {
//                 None => panic!("{} Error: No such variable `{}` in local scope", node.tok.pos, node.tok.val_str()),
//                 Some(var) => {
//                     writeln!(f, "; --- Assign {} ---", node.tok.val_str())?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    mov [rbp {}], rax", var.offset)?;
//                 }
//             }
//         },
//         NodeType::VarDecl => {
//             if ctx.vars.contains_key(&node.tok.val) {
//                 panic!("{} Error: Variable with this name is already declared `{}`", node.tok.pos, node.tok.val_str());
//             }
//             writeln!(f, "; --- VarDecl {} ---", node.tok.val_str())?;
//             // NOTE: This relies on the variable value being on top of the stack already.
//             ctx.vars.insert(node.tok.val.clone(), Var { kind: node.datatype.clone(), offset: ctx.stack_ix });
//             ctx.stack_ix -= 8;
//         },
//         NodeType::Var => {
//             match ctx.vars.get(&node.tok.val) {
//                 None => panic!("{} Error: No such variable `{}` in local scope", node.tok.pos, node.tok.val_str()),
//                 Some(var) => {
//                     writeln!(f, "; --- Var {} ---", node.tok.val_str())?;
//                     if var.kind == DataType::I64 {
//                         if var.offset < 0 {
//                             writeln!(f, "    mov rax, [rbp {}]", var.offset)?;
//                         } else {
//                             writeln!(f, "    mov rax, [rbp + {}]", var.offset)?;
//                         }
//                     } else if var.kind == DataType::I64Ptr {
//                         if var.offset < 0 {
//                             writeln!(f, "    lea rax, [rbp {}]", var.offset)?;
//                         } else {
//                             writeln!(f, "    lea rax, [rbp + {}]", var.offset)?;
//                         }
//                     } else {
//                         panic!("{} Error: Expected a variable type but got `{:?}`", node.tok.pos, var.kind);
//                     }
//
//                     writeln!(f, "    push rax")?;
//                 }
//             }
//         },
//         NodeType::Return => {
//             writeln!(f, "; --- Return ---")?;
//             writeln!(f, "    pop rax")?; // Return value in rax
//             writeln!(f, "    jmp .epilogue")?;
//         },
//         NodeType::MMap => {
//             writeln!(f, "; --- MMap ---")?;
//             writeln!(f, "    pop rsi")?;
//             writeln!(f, "    xor rdi, rdi")?;
//             writeln!(f, "    mov r8, -1")?;
//             writeln!(f, "    xor r9, r9")?;
//             writeln!(f, "; 34 = 32 | 2 = MAP_ANOYMOUS | MAP_PRIVATE")?;
//             writeln!(f, "    mov r10, 34")?;
//             writeln!(f, "    mov rax, 9")?;
//             writeln!(f, "    syscall")?;
//             writeln!(f, "    push rax")?;
//         },
//         NodeType::Exit => {
//             writeln!(f, "; --- Exit ---")?;
//             writeln!(f, "    pop rdi")?;
//             writeln!(f, "    mov rax, 60")?;
//             writeln!(f, "    syscall")?;
//         },
//         NodeType::DebugDump => {
//             writeln!(f, "; --- DebugDump ---")?;
//             writeln!(f, "    pop rdi")?;
//             writeln!(f, "    call dump")?;
//         },
//         NodeType::UnaryOp => {
//             match node.tok.kind {
//                 TokenType::OpMinus => {
//                     writeln!(f, "; --- UnOp::OpMinus ---")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    neg rax")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpDereference => {
//                     // TODO
//                     writeln!(f, "; --- UnOp::OpDereference ---")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    mov rax, [rax]")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 _ => panic!("Error: Unknown unary operator kind `{:?}`", node.tok.kind)
//             }
//         },
//         NodeType::BinaryOp => {
//             match node.tok.kind {
//                 TokenType::OpPlus => {
//                     writeln!(f, "; --- BinOp::OpPlus ---")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    add rax, rbx")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpMinus => {
//                     writeln!(f, "; --- BinOp::OpMinus---")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    sub rax, rbx")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpMul => {
//                     writeln!(f, "; --- BinOp::OpMul ---")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    imul rax, rbx")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpDiv => {
//                     writeln!(f, "; --- BinOp::OpDiv ---")?;
//                     writeln!(f, "    pop rcx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    xor rdx, rdx")?;
//                     writeln!(f, "    idiv rcx")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpLessThan => {
//                     writeln!(f, "; --- BinOp::OpLessThan ---")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, rbx")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    setl al")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpLessEqual => {
//                     writeln!(f, "; --- BinOp::OpLessEqual ---")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, rbx")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    setle al")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpGreaterThan => {
//                     writeln!(f, "; --- BinOp::OpGreaterThan ---")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, rbx")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    setg al")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpGreaterEqual => {
//                     writeln!(f, "; --- BinOp::OpGreaterEqual ---")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, rbx")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    setge al")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpEqual => {
//                     writeln!(f, "; --- BinOp::OpEqual ---")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, rbx")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    sete al")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpNotEqual => {
//                     writeln!(f, "; --- BinOp::OpNotEqual ---")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, rbx")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    setne al")?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpLogicalOr => {
//                     writeln!(f, "; --- BinOp::OpLogicalOr ---")?;
//                     writeln!(f, "._or_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    cmp rax, 0")?;
//                     writeln!(f, "    je ._rhs_{}_{}", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    mov rax, 1")?;
//                     writeln!(f, "    jmp ._end_{}_{}", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "._rhs_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    cmp rbx, 0")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    setne al")?;   // If rhs is true, set al to 1
//                     writeln!(f, "._end_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 TokenType::OpLogicalAnd => {
//                     writeln!(f, "; --- BinOp::OpLogicalAnd ---")?;
//                     writeln!(f, "._and_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    pop rbx")?;
//                     writeln!(f, "    cmp rax, 0")?;
//                     writeln!(f, "    jne ._rhs_{}_{}", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    jmp ._end_{}_{}", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "._rhs_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    cmp rbx, 0")?;
//                     writeln!(f, "    mov rax, 0")?;
//                     writeln!(f, "    setne al")?;
//                     writeln!(f, "._end_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
//                     writeln!(f, "    push rax")?;
//                 },
//                 _ => unimplemented!("Generating assembly for other bin ops"),
//             }
//         },
//         _ => {
//             panic!("{} Error: Invalid node in statement ({:?}) `{}`", node.tok.pos, node.kind, node.tok.val_str())
//         }
//     }
//
//     Ok(())
// }
//
// fn generate_block_nasm_x86(f: &mut fs::File, ctx: &mut Context, block: &ParseNode) -> std::io::Result<()> {
//     let mut block_ctx: Context = Context {
//         vars: HashMap::new(),
//         stack_ix: ctx.stack_ix,
//         loop_scope: ctx.loop_scope.clone(),
//     };
//     for var in ctx.vars.clone() {
//         block_ctx.vars.insert(var.0, var.1);
//     }
//
//     for block_item in &block.children {
//         generate_block_item_nasm_x86(f, &mut block_ctx, block_item)?;
//     }
//
//     let block_var_cnt: usize = block_ctx.vars.len() - ctx.vars.len();
//     writeln!(f, "; --- Deallocate block locals ---")?;
//     writeln!(f, "    add rsp, {}", block_var_cnt * 8)?;
//
//     Ok(())
// }
//
// fn generate_block_item_nasm_x86(f: &mut fs::File, ctx: &mut Context, block_item: &ParseNode) -> std::io::Result<()> {
//     match block_item.kind {
//         NodeType::WhileLoop => {
//             let tok: &Token = &block_item.tok;
//             let mut it: Iter<ParseNode> = block_item.children.iter();
//             let cond: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get condition in `while` loop", tok.pos));
//             let body: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get body in `while` loop", tok.pos));
//
//             let mut loop_ctx: Context = Context {
//                 vars: HashMap::new(),
//                 stack_ix: ctx.stack_ix,
//                 loop_scope: tok.clone(),
//             };
//             for var in ctx.vars.clone() {
//                 loop_ctx.vars.insert(var.0, var.1);
//             }
//
//             writeln!(f, "; --- While Loop ---")?;
//             writeln!(f, "; --- While Condition ---")?;
//             writeln!(f, "._loop_{}_{}:", tok.pos.row, tok.pos.col)?;
//             for node in &cond.post_order() {
//                 generate_node_nasm_x86(f, ctx, node)?;
//             }
//             writeln!(f, "    pop rax")?;
//             writeln!(f, "    cmp rax, 0")?;
//             writeln!(f, "    je ._end_{}_{}", tok.pos.row, tok.pos.col)?;
//             writeln!(f, "; --- While Body ---")?;
//             generate_block_nasm_x86(f, &mut loop_ctx, body)?;
//             writeln!(f, "; --- While Repeat ---")?;
//             writeln!(f, "._post_{}_{}:", tok.pos.row, tok.pos.col)?;
//             writeln!(f, "    jmp ._loop_{}_{}", tok.pos.row, tok.pos.col)?;
//             writeln!(f, "._end_{}_{}:", tok.pos.row, tok.pos.col)?;
//         },
//         NodeType::ForLoop => {
//             let tok: &Token = &block_item.tok;
//             let mut it: Iter<ParseNode> = block_item.children.iter();
//             let init: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get initialiser in `for` loop", tok.pos));
//             let cond: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get condition in `for` loop", tok.pos));
//             let post: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get post condition in `for` loop", tok.pos));
//             let body: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get body in `for` loop", tok.pos));
//
//             writeln!(f, "; --- For Loop ---")?;
//             writeln!(f, "; --- For Initialisation ---")?;
//             for node in &init.post_order() {
//                 generate_node_nasm_x86(f, ctx, node)?;
//             }
//             // Initialize loop context here because a variable can be declared in initialisation.
//             let mut loop_ctx: Context = Context {
//                 vars: HashMap::new(),
//                 stack_ix: ctx.stack_ix,
//                 loop_scope: tok.clone(),
//             };
//             for var in ctx.vars.clone() {
//                 loop_ctx.vars.insert(var.0, var.1);
//             }
//             writeln!(f, "; --- For Condition ---")?;
//             writeln!(f, "._loop_{}_{}:", tok.pos.row, tok.pos.col)?;
//             for node in &cond.post_order() {
//                 generate_node_nasm_x86(f, ctx, node)?;
//             }
//             writeln!(f, "    pop rax")?;
//             writeln!(f, "    cmp rax, 0")?;
//             writeln!(f, "    je ._end_{}_{}", tok.pos.row, tok.pos.col)?;
//             writeln!(f, "; --- For Body ---")?;
//             generate_block_nasm_x86(f, &mut loop_ctx, body)?;
//             writeln!(f, "; --- For Post ---")?;
//             writeln!(f, "._post_{}_{}:", tok.pos.row, tok.pos.col)?;
//             for node in &post.post_order() {
//                 generate_node_nasm_x86(f, ctx, node)?;
//             }
//             writeln!(f, "; --- For Repeat ---")?;
//             writeln!(f, "    jmp ._loop_{}_{}", tok.pos.row, tok.pos.col)?;
//             writeln!(f, "._end_{}_{}:", tok.pos.row, tok.pos.col)?;
//             if init.kind == NodeType::VarDecl {
//                 writeln!(f, "; --- Deallocate for variable ---")?;
//                 writeln!(f, "    add rsp, 8")?;
//                 ctx.stack_ix += 8;
//                 ctx.vars.remove(&init.tok.val);
//             }
//         },
//         NodeType::Conditional => {
//             let tok: &Token = &block_item.tok;
//             let mut it: Iter<ParseNode> = block_item.children.iter();
//             let guard: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get condition in `if` statement", tok.pos));
//             let if_body: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get else branch in `if` statement", tok.pos));
//             let else_body: Option<&ParseNode> = it.next();
//
//             writeln!(f, "; --- Conditional ---")?;
//             for node in &guard.post_order() {
//                 generate_node_nasm_x86(f, ctx, node)?;
//             }
//             match else_body {
//                 None => {
//                     writeln!(f, "; --- If (No Else) ---")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, 0")?;
//                     writeln!(f, "    je ._end_{}_{}", tok.pos.row, tok.pos.col)?;
//                     generate_block_nasm_x86(f, ctx, if_body)?;
//                     writeln!(f, "._end_{}_{}:", tok.pos.row, tok.pos.col)?;
//                 },
//                 Some(else_body) => {
//                     writeln!(f, "; --- If ---")?;
//                     writeln!(f, "    pop rax")?;
//                     writeln!(f, "    cmp rax, 0")?;
//                     writeln!(f, "    je ._false_{}_{}", tok.pos.row, tok.pos.col)?;
//                     generate_block_nasm_x86(f, ctx, if_body)?;
//                     writeln!(f, "    jmp ._end_{}_{}", tok.pos.row, tok.pos.col)?;
//                     writeln!(f, "; --- Else ---")?;
//                     writeln!(f, "._false_{}_{}:", tok.pos.row, tok.pos.col)?;
//                     generate_block_nasm_x86(f, ctx, else_body)?;
//                     writeln!(f, "._end_{}_{}:", tok.pos.row, tok.pos.col)?;
//                 }
//             }
//         },
//         NodeType::Assign | NodeType::Exit | NodeType::DebugDump | NodeType::VarDecl | NodeType::FuncCall | 
//         NodeType::Break | NodeType::Continue | NodeType::Return | NodeType::DerefAssign => {
//             for node in &block_item.post_order() {
//                 generate_node_nasm_x86(f, ctx, node)?;
//             }
//         },
//         _ => panic!("{} Error: Expected block item but got `{}` ({:?})", block_item.tok.pos, block_item.tok.val_str(), block_item.kind),
//     }
//
//     Ok(())
// }
//
// fn generate_nasm_x86(out_path: &String, ast: &mut ParseTree) -> std::io::Result<()> {
//     let mut f = fs::File::create(out_path)?;
//     writeln!(f, "; --- Header ---")?;
//     writeln!(f, "global _start")?;
//     writeln!(f, "section .text")?;
//     writeln!(f, "_start:")?;
//     writeln!(f, "    call main")?;
//     writeln!(f, "    mov rdi, 0")?;
//     writeln!(f, "    mov rax, 60")?;
//     writeln!(f, "    syscall")?;
//     writeln!(f, "; --- Debug Dump ---")?;
//     writeln!(f, "dump:")?;
//     writeln!(f, "    sub rsp, 40")?;
//     writeln!(f, "    lea rsi, [rsp + 31]")?;
//     writeln!(f, "    mov byte [rsp + 31], 10")?;
//     writeln!(f, "    mov ecx, 1")?;
//     writeln!(f, "    mov r8, -3689348814741910323")?;
//     writeln!(f, ".LBB0_1:")?;
//     writeln!(f, "    mov rax, rdi")?;
//     writeln!(f, "    mul r8")?;
//     writeln!(f, "    shr rdx, 3")?;
//     writeln!(f, "    lea eax, [rdx + rdx]")?;
//     writeln!(f, "    lea eax, [rax + 4*rax]")?;
//     writeln!(f, "    mov r9d, edi")?;
//     writeln!(f, "    sub r9d, eax")?;
//     writeln!(f, "    or r9b, 48")?;
//     writeln!(f, "    mov byte [rsi - 1], r9b")?;
//     writeln!(f, "    dec rsi")?;
//     writeln!(f, "    inc rcx")?;
//     writeln!(f, "    cmp rdi, 9")?;
//     writeln!(f, "    mov rdi, rdx")?;
//     writeln!(f, "    ja .LBB0_1")?;
//     writeln!(f, "    mov edi, 1")?;
//     writeln!(f, "    mov rdx, rcx")?;
//     writeln!(f, "    mov rax, 1")?;
//     writeln!(f, "    syscall")?;
//     writeln!(f, "    add rsp, 40")?;
//     writeln!(f, "    ret")?;
//
//     for func in &ast.root.children {
//         assert!(func.kind == NodeType::FuncDecl, "{} Error: Children of root must be functions", func.tok.pos);
//
//         writeln!(f, "; --- FuncDecl {} ---", func.tok.val_str())?;
//         writeln!(f, "{}:", func.tok.val_str())?;
//         writeln!(f, "; --- Prologue {} ---", func.tok.val_str())?;
//         writeln!(f, "    push rbp")?;
//         writeln!(f, "    mov rbp, rsp")?;
//         if let Some((body, params)) = func.children.split_last() {
//             // Params start at rbp + 16, each offset by 8.
//             let mut ctx: Context = Context {
//                 vars: HashMap::new(),
//                 stack_ix: 16,
//                 loop_scope: Token::null(),
//             };
//
//             writeln!(f, "; --- Pull In Params ---")?;
//             for param in params {
//                 ctx.vars.insert(param.tok.val.clone(), Var { kind: param.datatype.clone(), offset: ctx.stack_ix } );
//                 ctx.stack_ix += 8;
//             }
//             // Reset stack index to be just above rbp for locals.
//             ctx.stack_ix = -8;
//             writeln!(f, "; --- Declaring Params Done ---")?;
//
//             writeln!(f, "; --- Function Body ---")?;
//             for block_item in &body.children {
//                 generate_block_item_nasm_x86(&mut f, &mut ctx, block_item)?;
//             }
//         } else { // No Body
//             eprintln!("{} Warning: function `{}` with no params or body is redundant", func.tok.pos, func.tok.val_str());
//         }
//
//         writeln!(f, "; --- Epilogue {} ---", func.tok.val_str())?;
//         writeln!(f, ".epilogue:")?;
//         writeln!(f, "    mov rsp, rbp")?;
//         writeln!(f, "    pop rbp")?;
//         writeln!(f, "    ret")?;
//     }
//
//     // writeln!(f, "section .data")?;
//     // writeln!(f, "section .align 8")?;
//
//     Ok(())
// }
