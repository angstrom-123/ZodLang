use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::slice::Iter;
use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenType;
use crate::parser::NodeType;
use crate::parser::ParseNode;
use crate::parser::ParseTree;

pub mod lexer;
pub mod parser;

#[cfg(test)]
pub mod tests;

#[derive(PartialEq)]
enum Flag {
    EmitTokens,
    EmitParseTree,
    EmitAsm,
    Run
}

struct Context {
    vars:       HashMap<Vec<u8>, i64>,
    stack_ix:   i64,
    loop_scope: Token,
}

fn generate_node_nasm_x86(f: &mut fs::File, ctx: &mut Context, node: &ParseNode) -> std::io::Result<()> {
    match node.kind {
        NodeType::Continue => {
            if ctx.loop_scope.kind == TokenType::None {
                panic!("{} Error: Unexpected `continue` outside of breakable scope", node.tok.pos);
            }
            writeln!(f, "; --- Continue ---")?;
            writeln!(f, "   jmp _post_{}_{}", ctx.loop_scope.pos.row, ctx.loop_scope.pos.col)?;
        },
        NodeType::Break => {
            if ctx.loop_scope.kind == TokenType::None {
                panic!("{} Error: Unexpected `break` outside of breakable scope", node.tok.pos);
            }
            writeln!(f, "; --- Break ---")?;
            writeln!(f, "   jmp _end_{}_{}", ctx.loop_scope.pos.row, ctx.loop_scope.pos.col)?;
        },
        NodeType::Null => {
            writeln!(f, "; --- Null Node ---")?;
        },
        NodeType::FuncCall => {
            writeln!(f, "; --- FuncCall {} ---", node.tok.val_str())?;
            writeln!(f, "    call {}", node.tok.val_str())?;
        },
        NodeType::Literal => {
            writeln!(f, "; --- Literal {} ---", node.tok.val_str())?;
            writeln!(f, "    mov rax, {}", node.tok.val_str())?;
            writeln!(f, "    push rax")?;
        },
        NodeType::Assign => {
            match ctx.vars.get(&node.tok.val) {
                None => panic!("{} Error: No such variable `{}` in local scope", node.tok.pos, node.tok.val_str()),
                Some(ofst) => {
                    writeln!(f, "; --- Assign {} ---", node.tok.val_str())?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    mov [rbp {}], rax", ofst)?;
                }
            }
        },
        NodeType::VarDecl => {
            if ctx.vars.contains_key(&node.tok.val) {
                panic!("{} Error: Variable with this name is already declared `{}`", node.tok.pos, node.tok.val_str());
            }
            writeln!(f, "; --- VarDecl {} ---", node.tok.val_str())?;
            // NOTE: This relies on the variable value being atop the stack already.
            ctx.vars.insert(node.tok.val.clone(), ctx.stack_ix);
            ctx.stack_ix -= 8;
        },
        NodeType::Var => {
            match ctx.vars.get(&node.tok.val) {
                None => panic!("{} Error: No such variable `{}` in local scope", node.tok.pos, node.tok.val_str()),
                Some(ofst) => {
                    writeln!(f, "; --- Var {} ---", node.tok.val_str())?;
                    writeln!(f, "    mov rax, [rbp {}]", ofst)?;
                    writeln!(f, "    push rax")?;
                }
            }
        },
        NodeType::Exit => {
            writeln!(f, "; --- Exit ---")?;
            writeln!(f, "    pop rdi")?;
            writeln!(f, "    mov rax, 60")?;
            writeln!(f, "    syscall")?;
        },
        NodeType::DebugDump => {
            writeln!(f, "; --- DebugDump ---")?;
            writeln!(f, "    pop rdi")?;
            writeln!(f, "    call dump")?;
        },
        NodeType::UnOp => {
            match node.tok.kind {
                TokenType::OpMinus => {
                    writeln!(f, "; --- UnOp::OpMinus ---")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    neg rax")?;
                    writeln!(f, "    push rax")?;
                },
                _ => panic!("Error: Unknown unary operator kind `{:?}`", node.tok.kind)
            }
        },
        NodeType::BinOp => {
            match node.tok.kind {
                TokenType::OpPlus => {
                    writeln!(f, "; --- BinOp::OpPlus ---")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    add rax, rbx")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpMinus => {
                    writeln!(f, "; --- BinOp::OpMinus---")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    sub rax, rbx")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpMul => {
                    writeln!(f, "; --- BinOp::OpMul ---")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    imul rax, rbx")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpDiv => {
                    writeln!(f, "; --- BinOp::OpDiv ---")?;
                    writeln!(f, "    pop rcx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    xor rdx, rdx")?;
                    writeln!(f, "    idiv rcx")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpLessThan => {
                    writeln!(f, "; --- BinOp::OpLessThan ---")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, rbx")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    setl al")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpLessEqual => {
                    writeln!(f, "; --- BinOp::OpLessEqual ---")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, rbx")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    setle al")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpGreaterThan => {
                    writeln!(f, "; --- BinOp::OpGreaterThan ---")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, rbx")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    setg al")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpGreaterEqual => {
                    writeln!(f, "; --- BinOp::OpGreaterEqual ---")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, rbx")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    setge al")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpEqual => {
                    writeln!(f, "; --- BinOp::OpEqual ---")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, rbx")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    sete al")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpNotEqual => {
                    writeln!(f, "; --- BinOp::OpNotEqual ---")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, rbx")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    setne al")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpLogicalOr => {
                    writeln!(f, "; --- BinOp::OpLogicalOr ---")?;
                    writeln!(f, "_or_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    cmp rax, 0")?;
                    writeln!(f, "    je ._rhs")?;    // If lhs is false, check rhs
                    writeln!(f, "    mov rax, 1")?;
                    writeln!(f, "    jmp ._end")?;   // If lhs is true, short circuit
                    writeln!(f, "._rhs:")?;
                    writeln!(f, "    cmp rbx, 0")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    setne al")?;   // If rhs is true, set al to 1
                    writeln!(f, "._end:")?;
                    writeln!(f, "    push rax")?;
                },
                TokenType::OpLogicalAnd => {
                    writeln!(f, "; --- BinOp::OpLogicalAnd ---")?;
                    writeln!(f, "_and_{}_{}:", node.tok.pos.col, node.tok.pos.row)?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    pop rbx")?;
                    writeln!(f, "    cmp rax, 0")?;
                    writeln!(f, "    jne ._rhs")?;  // If lhs is true, check rhs
                    writeln!(f, "    jmp ._end")?;  // If lhs is false, short circuit
                    writeln!(f, "._rhs:")?;
                    writeln!(f, "    cmp rbx, 0")?;
                    writeln!(f, "    mov rax, 0")?;
                    writeln!(f, "    setne al")?;   // If rhs is true, set al to 1
                    writeln!(f, "._end:")?;
                    writeln!(f, "    push rax")?;
                },
                _ => unimplemented!("Generating assembly for other bin ops"),
            }
        },
        _ => {
            panic!("{} Error: Invalid node in statement ({:?}) `{}`", node.tok.pos, node.kind, node.tok.val_str())
        }
    }

    Ok(())
}

fn generate_block_nasm_x86(f: &mut fs::File, ctx: &mut Context, block: &ParseNode) -> std::io::Result<()> {
    let mut block_ctx: Context = Context {
        vars: HashMap::new(),
        stack_ix: ctx.stack_ix,
        loop_scope: ctx.loop_scope.clone(),
    };
    for var in ctx.vars.clone() {
        block_ctx.vars.insert(var.0, var.1);
    }

    for block_item in &block.children {
        generate_block_item_nasm_x86(f, &mut block_ctx, block_item)?;
    }

    let block_var_cnt: usize = block_ctx.vars.len() - ctx.vars.len();
    writeln!(f, "; --- Deallocate block locals ---")?;
    writeln!(f, "    add rsp, {}", block_var_cnt * 8)?;

    Ok(())
}

fn generate_block_item_nasm_x86(f: &mut fs::File, ctx: &mut Context, block_item: &ParseNode) -> std::io::Result<()> {
    match block_item.kind {
        NodeType::WhileLoop => {
            let tok: &Token = &block_item.tok;
            let mut it: Iter<ParseNode> = block_item.children.iter();
            let cond: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get condition in `while` loop", tok.pos));
            let body: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get body in `while` loop", tok.pos));

            let mut loop_ctx: Context = Context {
                vars: HashMap::new(),
                stack_ix: ctx.stack_ix,
                loop_scope: tok.clone(),
            };
            for var in ctx.vars.clone() {
                loop_ctx.vars.insert(var.0, var.1);
            }

            writeln!(f, "; --- While Loop ---")?;
            writeln!(f, "; --- While Condition ---")?;
            writeln!(f, "_loop_{}_{}:", tok.pos.row, tok.pos.col)?;
            for node in &cond.post_order() {
                generate_node_nasm_x86(f, ctx, node)?;
            }
            writeln!(f, "    pop rax")?;
            writeln!(f, "    cmp rax, 0")?;
            writeln!(f, "    je _end_{}_{}", tok.pos.row, tok.pos.col)?;
            writeln!(f, "; --- While Body ---")?;
            generate_block_nasm_x86(f, &mut loop_ctx, body)?;
            writeln!(f, "; --- While Repeat ---")?;
            writeln!(f, "_post_{}_{}:", tok.pos.row, tok.pos.col)?;
            writeln!(f, "    jmp _loop_{}_{}", tok.pos.row, tok.pos.col)?;
            writeln!(f, "_end_{}_{}:", tok.pos.row, tok.pos.col)?;
        },
        NodeType::ForLoop => {
            let tok: &Token = &block_item.tok;
            let mut it: Iter<ParseNode> = block_item.children.iter();
            let init: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get initialiser in `for` loop", tok.pos));
            let cond: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get condition in `for` loop", tok.pos));
            let post: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get post condition in `for` loop", tok.pos));
            let body: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get body in `for` loop", tok.pos));

            writeln!(f, "; --- For Loop ---")?;
            writeln!(f, "; --- For Initialisation ---")?;
            for node in &init.post_order() {
                generate_node_nasm_x86(f, ctx, node)?;
            }
            // Initialize loop context here because a variable can be declared in initialisation.
            let mut loop_ctx: Context = Context {
                vars: HashMap::new(),
                stack_ix: ctx.stack_ix,
                loop_scope: tok.clone(),
            };
            for var in ctx.vars.clone() {
                loop_ctx.vars.insert(var.0, var.1);
            }
            writeln!(f, "; --- For Condition ---")?;
            writeln!(f, "_loop_{}_{}:", tok.pos.row, tok.pos.col)?;
            for node in &cond.post_order() {
                generate_node_nasm_x86(f, ctx, node)?;
            }
            writeln!(f, "    pop rax")?;
            writeln!(f, "    cmp rax, 0")?;
            writeln!(f, "    je _end_{}_{}", tok.pos.row, tok.pos.col)?;
            writeln!(f, "; --- For Body ---")?;
            generate_block_nasm_x86(f, &mut loop_ctx, body)?;
            writeln!(f, "; --- For Post ---")?;
            writeln!(f, "_post_{}_{}:", tok.pos.row, tok.pos.col)?;
            for node in &post.post_order() {
                generate_node_nasm_x86(f, ctx, node)?;
            }
            writeln!(f, "; --- For Repeat ---")?;
            writeln!(f, "    jmp _loop_{}_{}", tok.pos.row, tok.pos.col)?;
            writeln!(f, "_end_{}_{}:", tok.pos.row, tok.pos.col)?;
            if init.kind == NodeType::VarDecl {
                writeln!(f, "; --- Deallocate for variable ---")?;
                writeln!(f, "    add rsp, 8")?;
                ctx.stack_ix += 8;
                ctx.vars.remove(&init.tok.val);
            }
        },
        NodeType::Conditional => {
            let tok: &Token = &block_item.tok;
            let mut it: Iter<ParseNode> = block_item.children.iter();
            let guard: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get condition in `if` statement", tok.pos));
            let if_body: &ParseNode = it.next().unwrap_or_else(|| panic!("{} Error: Failed to get else branch in `if` statement", tok.pos));
            let else_body: Option<&ParseNode> = it.next();

            writeln!(f, "; --- Conditional ---")?;
            for node in &guard.post_order() {
                generate_node_nasm_x86(f, ctx, node)?;
            }
            writeln!(f, "_if_{}_{}:", tok.pos.row, tok.pos.col)?;
            match else_body {
                None => {
                    writeln!(f, "; --- If (No Else) ---")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, 0")?;
                    writeln!(f, "    je _end_{}_{}", tok.pos.row, tok.pos.col)?;
                    generate_block_nasm_x86(f, ctx, if_body)?;
                    writeln!(f, "_end_{}_{}:", tok.pos.row, tok.pos.col)?;
                },
                Some(else_body) => {
                    writeln!(f, "; --- If ---")?;
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    cmp rax, 0")?;
                    writeln!(f, "    je _false_{}_{}", tok.pos.row, tok.pos.col)?;
                    generate_block_nasm_x86(f, ctx, if_body)?;
                    writeln!(f, "    jmp _end_{}_{}", tok.pos.row, tok.pos.col)?;
                    writeln!(f, "; --- Else ---")?;
                    writeln!(f, "_false_{}_{}:", tok.pos.row, tok.pos.col)?;
                    generate_block_nasm_x86(f, ctx, else_body)?;
                    writeln!(f, "_end_{}_{}:", tok.pos.row, tok.pos.col)?;
                }
            }
        },
        NodeType::Assign | NodeType::Exit | NodeType::DebugDump | NodeType::VarDecl | NodeType::FuncCall | 
        NodeType::Break | NodeType::Continue => {
            for node in &block_item.post_order() {
                generate_node_nasm_x86(f, ctx, node)?;
            }
        },
        _ => panic!("{} Error: Expected block item but got `{}`", block_item.tok.pos, block_item.tok.val_str()),
    }

    Ok(())
}

fn generate_nasm_x86(out_path: &String, ast: &mut ParseTree) -> std::io::Result<()> {
    let mut f = fs::File::create(out_path)?;
    writeln!(f, "; --- Header ---")?;
    writeln!(f, "global _start")?;
    writeln!(f, "section .text")?;
    writeln!(f, "; --- Debug Dump ---")?;
    writeln!(f, "dump:")?;
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

    for func in &ast.root.children {
        assert!(func.kind == NodeType::FuncDecl, "{} Error: Children of root must be functions", func.tok.pos);

        writeln!(f, "; --- FuncDecl {} ---", func.tok.val_str())?;
        writeln!(f, "{}:", func.tok.val_str())?;
        writeln!(f, "; --- Prologue {} ---", func.tok.val_str())?;
        writeln!(f, "    push rbp")?;
        writeln!(f, "    mov rbp, rsp")?;

        let mut ctx: Context = Context {
            vars: HashMap::new(),
            stack_ix: -8,
            loop_scope: Token::new_null(),
        };
        for block_item in &func.children {
            generate_block_item_nasm_x86(&mut f, &mut ctx, block_item)?;
        }

        writeln!(f, "; --- Epilogue {} ---", func.tok.val_str())?;
        writeln!(f, "    mov rsp, rbp")?;
        writeln!(f, "    pop rbp")?;
        writeln!(f, "    ret")?;
    }

    writeln!(f, "; --- Footer ---")?;
    writeln!(f, "_start:")?;
    writeln!(f, "    call main")?;
    writeln!(f, "    mov rdi, 0")?;
    writeln!(f, "    mov rax, 60")?;
    writeln!(f, "    syscall")?;

    Ok(())
}

fn compile(src_code: Vec<u8>, src_path: String, _res_path: String, flags: Vec<Flag>) {
    eprintln!("\nInfo: Compiling program");
    let mut obj_path: String = _res_path.clone();
    obj_path.push_str(".o");

    let mut asm_path: String = _res_path.clone();
    asm_path.push_str(".asm");

    let mut res_path = _res_path.clone();
    res_path.insert_str(0, "./");

    let mut lexer: Lexer = Lexer::new(src_code);
    lexer.tokenize();
    lexer.lex();
    if flags.contains(&Flag::EmitTokens) {
        eprintln!("Info: Emitting Tokens:");
        for tok in &lexer.toks {
            eprintln!("    Token: {}: {:?} `{}`", tok.pos, tok.kind, tok.val_str());
        }
        eprintln!();
    }
    let ast = &mut parser::ParseTree::new(src_path.clone());
    ast.construct(&mut lexer);
    if flags.contains(&Flag::EmitParseTree) {
        eprintln!("Info: Emitting Parse Tree:");
        ast.dump();
        eprintln!();
    }

    let generate = generate_nasm_x86(&asm_path, ast);
    let _ = generate.inspect_err(|e| panic!("Error: Failed to generate assembly: {e}"));

    eprintln!("Info: Calling `nasm -f elf64 -o {} {}`", &obj_path, &asm_path);
    let assemble = Command::new("nasm").arg("-f").arg("elf64").arg("-o").arg(&obj_path).arg(&asm_path).output();
    let assemble_err: String = String::from_utf8(assemble.ok().unwrap().stderr).expect("");
    if !assemble_err.is_empty() {
        panic!("\n\x1b[31mCOMPILATION FAILED (assembler) \n{}\x1b[0m", assemble_err);
    }

    eprintln!("Info: Calling `ld -o {} {}`", &res_path, &obj_path);
    let link = Command::new("ld").arg("-o").arg(&res_path).arg(&obj_path).output();
    let link_err: String = String::from_utf8(link.ok().unwrap().stderr).expect("");
    if !link_err.is_empty() {
        panic!("\n\x1b[31mCOMPILATION FAILED (linker) \n{}\x1b[0m", link_err);
    }

    if !flags.contains(&Flag::EmitAsm) {
        eprintln!("Info: Calling `rm {}`", &asm_path);
        let rm_asm = Command::new("rm").arg(&asm_path).output();
        let rm_asm_err: String = String::from_utf8(rm_asm.expect("Error: Failed to retrieve output of assembling").stderr).expect("Error: Failed to convert stderr to string");
        if !rm_asm_err.is_empty() {
            panic!("\n\x1b[31mCOMPILATION FAILED (delete intermediate .asm) \n{}\x1b[0m", rm_asm_err);
        }
    }

    eprintln!("Info: Calling `rm {}`", &obj_path);
    let rm_o = Command::new("rm").arg(&obj_path).output();
    let rm_o_err: String = String::from_utf8(rm_o.expect("Error: Failed to retrieve result of linking").stderr).expect("Error: Failed to convert stderr to string");
    if !rm_o_err.is_empty() {
        panic!("\n\x1b[31mCOMPILATION FAILED (delete intermediate .o) \n{}\x1b[0m", rm_o_err);
    }

    eprintln!("\n\x1b[92mCOMPILATION COMPLETE\x1b[0m");

    if flags.contains(&Flag::Run) {
        eprintln!("Info: Calling `{}`", &res_path);
        let run = Command::new(&res_path).spawn().expect("Error: Failed to run executable").wait_with_output();
        let status = run.expect("Error: Failed to retrieve output of running").status;
        eprintln!("Info: Exit code {}", status.code().expect("Error: Failed to retrieve exit code of executable"));
    }
}

pub fn usage(com: &str) -> String {
    format!("
\x1b[31mCOMPILATION FAILED\x1b[0m

\x1b[92mUSAGE:\x1b[0m
  {} \x1b[33m<input-file> <flags>\x1b[0m 

\x1b[92mFLAGS:\x1b[0m
  \x1b[33m-r     --run\x1b[0m:          Run after compiling
  \x1b[33m-pt    --parse-tree\x1b[0m:   Print parse tree
  \x1b[33m-t     --tokens\x1b[0m:       Print tokens
  \x1b[33m-a     --assembly\x1b[0m:     Keep intermediate assembly
", com)
}                  
                   
pub fn main() {
    // let args: Vec<String> = env::args().collect();

    // TODO: make the args come in any order then make sure tests run with new name

    let mut flags: Vec<Flag> = Vec::new();

    let mut it = env::args();
    let com: String = it.next().unwrap_or_else(|| panic!("Error: Failed to get command name from args"));
    let mut out_path: Option<String> = None;
    let mut in_path: Option<String> = None;
    // for arg in it {
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-r" | "--run" => flags.push(Flag::Run),
            "-a" | "--assembly" => flags.push(Flag::EmitAsm),
            "-pt" | "--parse-tree" => flags.push(Flag::EmitParseTree),
            "-t" | "--tokens" => flags.push(Flag::EmitTokens),
            "-o" | "--output" => out_path = it.next(),
            _ => {
                match in_path {
                    None => in_path = Some(arg),
                    Some(_) => panic!("{}", usage(&com)),
                }
            }
        }
    }

    let mut out: String = "output".to_string();
    if let Some(path) = out_path {
        out = path;
    }

    match in_path {
        None => panic!("{}", usage(&com)),
        Some(path) => {
            let src: Vec<u8> = fs::read(&path).unwrap_or_else(|_| panic!("{}", usage(&com)));
            compile(src, path.to_string(), out, flags);
        }
    }
}
