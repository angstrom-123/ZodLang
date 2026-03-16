use std::process::Command;

use crate::asmgen;
use crate::lex::{Lexer};
use crate::parse::AST;
use crate::preprocess::Processor;
use crate::represent::IR;
use crate::types::Analyser;
use crate::optimise::Optimiser;

#[derive(PartialEq)]
pub enum Flag {
    EmitTokens,
    EmitParseTree,
    EmitAsm,
    EmitIR,
    Optimise,
    Run,
    Verbose
}

pub struct Compiler {
    pub flags: Vec<Flag>,
    pub inc_paths: Vec<String>,
}
impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
impl Compiler {
    pub fn new() -> Self {
        Compiler {
            flags: Vec::new(),
            inc_paths: Vec::new(),
        }
    }
    
    pub fn compile(&self, src_code: Vec<u8>, src_path: String, _res_path: String) {
        self.info("Compiling program");
        let mut obj_path: String = _res_path.clone();
        obj_path.push_str(".o");
    
        let mut asm_path: String = _res_path.clone();
        asm_path.push_str(".asm");
    
        let mut res_path = _res_path.clone();
        res_path.insert_str(0, "./");
    
        let mut lexer: Lexer = Lexer::new(src_code, &src_path);
        self.info("Tokenizing Input File");
        lexer.tokenize();
        self.info("Lexing Tokens");
        lexer.lex();
        if self.flags.contains(&Flag::EmitTokens) {
            self.info("Emitting Tokens:");
            lexer.dump();
            eprintln!();
        }
    
        self.info("Resolving includes");
        let mut preprocessor: Processor = Processor::new();
        let mut include_paths: Vec<String> = self.inc_paths.clone();
        include_paths.push(String::from("./std/"));
        preprocessor.resolve_includes(&mut lexer, include_paths);
    
        self.info("Constructing AST");
        let mut ast: AST = AST::new();
        ast.construct(&mut lexer);

        self.info("Performing Static Type Analysis");
        let mut analyser: Analyser = Analyser::new();
        analyser.typecheck_ast(&mut ast);

        if self.flags.contains(&Flag::EmitParseTree) {
            self.info("Emitting Parse Tree:");
            ast.dump();
            eprintln!();
        }

        self.info("Generating Intermediate Representation");
        let mut ir: IR = IR::new(&ast);
        ir.generate();
        if self.flags.contains(&Flag::EmitIR) {
            self.info("Emitting Intermediate Representation:");
            ir.dump();
            eprintln!();
        }
        
        if self.flags.contains(&Flag::Optimise) {
            self.info("Optimising Intermediate Representation");
            let mut optimiser: Optimiser = Optimiser::new();
            optimiser.optimise(&mut ir);
        }
    
        self.info("Generating Assembly (nasm x86_64)");
        let generate = asmgen::nasm_x86_64::generate(&asm_path, &ir);
        let _ = generate.inspect_err(|e| panic!("Error: Failed to generate assembly: {e}"));
    
        if self.flags.contains(&Flag::Verbose) {
            eprintln!("Info: Calling `nasm -f elf64 -o {} {}`", &obj_path, &asm_path);
        }
        let assemble = Command::new("nasm").arg("-f").arg("elf64").arg("-o").arg(&obj_path).arg(&asm_path).output();
        let assemble_err: String = String::from_utf8(assemble.ok().unwrap().stderr).expect("");
        if !assemble_err.is_empty() {
            panic!("\n\x1b[31mCOMPILATION FAILED (assembler) \n{}\x1b[0m", assemble_err);
        }
    
        if self.flags.contains(&Flag::Verbose) {
            eprintln!("Info: Calling `ld -o {} {}`", &res_path, &obj_path);
        }
        let link = Command::new("ld").arg("-o").arg(&res_path).arg(&obj_path).output();
        let link_err: String = String::from_utf8(link.ok().unwrap().stderr).expect("");
        if !link_err.is_empty() {
            panic!("\n\x1b[31mCOMPILATION FAILED (linker) \n{}\x1b[0m", link_err);
        }
    
        if !self.flags.contains(&Flag::EmitAsm) {
            if self.flags.contains(&Flag::Verbose) {
                eprintln!("Info: Calling `rm {}`", &asm_path);
            }
            let rm_asm = Command::new("rm").arg(&asm_path).output();
            let rm_asm_err: String = String::from_utf8(rm_asm.expect("Error: Failed to retrieve output of assembling").stderr).expect("Error: Failed to convert stderr to string");
            if !rm_asm_err.is_empty() {
                panic!("\n\x1b[31mCOMPILATION FAILED (delete intermediate .asm) \n{}\x1b[0m", rm_asm_err);
            }
        }
    
        if self.flags.contains(&Flag::Verbose) {
            eprintln!("Info: Calling `rm {}`", &obj_path);
        }
        let rm_o = Command::new("rm").arg(&obj_path).output();
        let rm_o_err: String = String::from_utf8(rm_o.expect("Error: Failed to retrieve result of linking").stderr).expect("Error: Failed to convert stderr to string");
        if !rm_o_err.is_empty() {
            panic!("\n\x1b[31mCOMPILATION FAILED (delete intermediate .o) \n{}\x1b[0m", rm_o_err);
        }
    
        if self.flags.contains(&Flag::Verbose) {
            eprintln!("\n\x1b[92mCOMPILATION COMPLETE\x1b[0m\n");
        }
    
        if self.flags.contains(&Flag::Run) {
            eprintln!("Info: Calling `{}`", &res_path);
            let run = Command::new(&res_path).spawn().expect("Error: Failed to run executable").wait_with_output();
            let status = run.expect("Error: Failed to retrieve output of running").status;
            eprintln!("Info: Exit code {}", status.code().expect("Error: Failed to retrieve exit code of executable"));
        }
    }
    
    fn info(&self, msg: &'static str) {
        if self.flags.contains(&Flag::Verbose) {
            eprintln!("Info: {}", msg);
        }
    }
}

pub fn usage(com: &str) -> String {
format!("
\x1b[31mCOMPILATION FAILED\x1b[0m

\x1b[92mUSAGE:\x1b[0m
  {} \x1b[33m<input-file> <flags>\x1b[0m 

\x1b[92mFLAGS:\x1b[0m
  \x1b[33m-a              --assembly             \x1b[0m      Keep intermediate assembly
  \x1b[33m-ir             --inter-repr           \x1b[0m      Print intermediate representation
  \x1b[33m-I    <path>    --include     <path>   \x1b[0m      Specify path to search for includes
  \x1b[33m-o    <path>    --output      <path>   \x1b[0m      Specify output file path
  \x1b[33m-O              --optimise             \x1b[0m      Enable optimisation
  \x1b[33m-p              --parsetree            \x1b[0m      Print parse tree
  \x1b[33m-r              --run                  \x1b[0m      Run after compiling
  \x1b[33m-t              --tokens               \x1b[0m      Print tokens
  \x1b[33m-v              --verbose              \x1b[0m      Enable info logging
", com)
}
