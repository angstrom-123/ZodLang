use std::env;
use std::fs;
use std::process::Command;
use crate::lexer::Lexer;
use crate::parser::ParseTree;
use crate::representation::IR;

pub mod lexer;
pub mod parser;
pub mod representation;
pub mod asmgen;

// TODO: Need to check if stack needs to be 16 byte aligned when calling a function.

#[cfg(test)]
pub mod tests;

#[derive(PartialEq)]
enum Flag {
    EmitTokens,
    EmitParseTree,
    EmitAsm,
    EmitIR,
    Run
}

fn compile(src_code: Vec<u8>, _res_path: String, flags: Vec<Flag>) {
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
    let mut ast: ParseTree = parser::ParseTree::new();
    ast.construct(&mut lexer);
    if flags.contains(&Flag::EmitParseTree) {
        eprintln!("Info: Emitting Parse Tree:");
        ast.dump();
        eprintln!();
    }

    let mut ir: IR = IR::new();
    ir.generate_from_ast(&ast);
    if flags.contains(&Flag::EmitIR) {
        eprintln!("Info: Emitting Intermediate Representation:");
        ir.dump();
        eprintln!();
    }

    // let generate = generate_nasm_x86(&asm_path, ast);
    let generate = asmgen::nasm_x86_64::generate(&asm_path, &ir);
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
            "-ir" | "--intermediate-representation" => flags.push(Flag::EmitIR),
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
            compile(src, out, flags);
        }
    }
}
