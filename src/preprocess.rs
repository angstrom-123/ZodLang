use std::fs;

use crate::lexer::{Lexer, Token, TokenType};

pub struct Processor {

}
impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}
impl Processor {
    pub fn new() -> Self {
        Processor {
            
        }
    }

    pub fn resolve_includes(&mut self, lexer: &mut Lexer, include_paths: Vec<String>) -> bool {
        let mut resolved: bool = false;
        while lexer.has_token() {
            let tok: Token = lexer.consume_token();
            if tok.kind == TokenType::KeywordInclude {
                // Get String Literal
                let file: Token = lexer.expect_token(TokenType::String);

                // Find Source File
                let mut file_src: Option<Vec<u8>> = None;
                for path in &include_paths {
                    let mut file_path: String = path.clone();
                    file_path.push_str(&file.val_str());
                    file_path.pop(); // Remove null terminator

                    file_src = fs::read(file_path).ok();
                    if file_src.is_some() {
                        break;
                    }
                }
                if let Some(file_src) = file_src {
                    // Remove Include and File name tokens
                    lexer.cur -= 2;
                    lexer.toks.remove(lexer.cur);
                    lexer.toks.remove(lexer.cur);

                    // Tokenize file
                    let mut l: Lexer = Lexer::new(file_src, &file.val_str());
                    l.tokenize();
                    l.lex();
                    let mut p: Processor = Processor::new();
                    let mut subresolved: bool = false;
                    while !subresolved {
                        subresolved = !p.resolve_includes(&mut l, include_paths.clone());
                    }

                    // Copy in contents of included file
                    for tok in l.toks {
                        lexer.toks.insert(lexer.cur, tok);
                        lexer.cur += 1;
                    }
                    resolved = true;
                } else {
                    panic!("{} Error: Unable to resolve include. Could not find file `{}`", file.pos, file.val_str());
                }
            }
        }

        lexer.refresh();
        resolved
    }
}

