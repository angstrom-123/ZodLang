use std::env;
use std::fs;

use crate::compiler::Compiler;
use crate::compiler::Flag;
use crate::compiler::usage;

#[cfg(test)]
pub mod tests;

pub mod lexer;
pub mod parser;
pub mod types;
pub mod representation;
pub mod asmgen;
pub mod preprocess;
pub mod compiler;
                   
fn main() {
    let mut compiler: Compiler = Compiler::new();

    let mut it = env::args();
    let com: String = it.next().unwrap_or_else(|| panic!("Error: Failed to get command name from args"));
    let mut out_path: Option<String> = None;
    let mut in_path: Option<String> = None;
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-a" | "--assembly"     => compiler.flags.push(Flag::EmitAsm),
            "-ir" | "--inter-repr"  => compiler.flags.push(Flag::EmitIR),
            "-o" | "--output"       => out_path = it.next(),
            "-pt" | "--parsetree"   => compiler.flags.push(Flag::EmitParseTree),
            "-r" | "--run"          => compiler.flags.push(Flag::Run),
            "-t" | "--tokens"       => compiler.flags.push(Flag::EmitTokens),
            "-v" | "--verbose"      => compiler.flags.push(Flag::Verbose),
            _ => match in_path {
                None => in_path = Some(arg),
                Some(_) => panic!("{}", usage(&com)),
            }
        }
    }

    let mut out: String = "output".to_string();
    if let Some(path) = out_path {
        out = path;
    }

    if let Some(path) = in_path {
        let src: Vec<u8> = fs::read(&path).unwrap_or_else(|_| panic!("{}", usage(&com)));
        compiler.compile(src, path, out);
    } else {
        panic!("{}", usage(&com));
    }
}
