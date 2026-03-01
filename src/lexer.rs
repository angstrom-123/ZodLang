use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenType {
    None,
    OpPlus,
    OpMinus,
    OpMul,
    OpDiv,
    OpAssign,
    OpEqual,
    OpNotEqual,
    OpGreaterThan,
    OpLessThan,
    OpGreaterEqual,
    OpLessEqual,
    OpLogicalOr,
    OpLogicalAnd,
    KeywordFunctionDecl,
    KeywordExit,
    KeywordDebugDump,
    KeywordVariableDecl,
    KeywordIf,
    KeywordElse,
    KeywordFor,
    KeywordBreak,
    KeywordContinue,
    KeywordWhile,
    End,
    OpenParen,
    CloseParen,
    OpenScope,
    CloseScope,
    Identifier,
    LiteralInt,
}

#[derive(Clone)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}
impl fmt::Display for Pos {
    // NOTE: Stored row and column are indices starting from 0, whereas in files, we count from 1.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", self.row + 1, self.col + 1)
    }
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenType,
    pub val:  Vec<u8>,
    pub pos:  Pos,
}
impl Token {
    pub fn val_str(&self) -> String {
        String::from_utf8(self.val.clone()).expect("Error: Failed to convert token value to string")
    }

    pub fn new_null() -> Self {
        Token {
            kind: TokenType::None,
            val: vec![],
            pos: Pos { row: usize::MAX - 1, col: usize::MAX - 1 },
        }
    }
}

pub struct Lexer {
    pub toks: Vec<Token>,
    pub pos:  Pos,
    src:      Vec<u8>,
    cur:      usize,
    rune:     u8,
}
impl Lexer {
    pub fn new(src: Vec<u8>) -> Self {
        let first = *src.first().expect("Error: Provided source file is empty");
        Lexer { 
            toks: Vec::new(),
            pos: Pos { row: 0, col: 0 },
            src,
            cur: 0,
            rune: first,
        }
    }

    pub fn has_token(&self) -> bool {
        self.cur < self.toks.len() - 1
    }

    pub fn consume_token(&mut self) -> Token {
        let tok = self.toks.get(self.cur).expect("Error: Lexer failed to consume next token");
        self.cur += 1;
        tok.clone()
    }

    pub fn peek_token(&mut self) -> Token {
        let tok = self.toks.get(self.cur).expect("Error: Lexer failed to peek next token");
        tok.clone()
    }

    pub fn previous_token(&mut self) -> Token {
        let tok = self.toks.get(self.cur - 1).expect("Error: Lexer failed to peek previous token");
        tok.clone()
    }

    pub fn dump_remaining_tokens(&mut self) {
        eprintln!("Lexer Dump:");
        for i in self.cur..self.toks.len() {
            let tok = self.toks.get(i).expect("Error: Lexer failed to dump next token");
            eprintln!("{}", tok.val_str());
        }
    }

    pub fn tokenize(&mut self) {
        let mut lexeme: Vec<u8> = Vec::new();
        loop {
            match self.rune {
                b' ' | b'\n' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                        });
                        lexeme.clear();
                    }
                },
                b'/' => {
                    if !lexeme.is_empty() {
                        let last: u8 = *lexeme.last().expect("Error: Failed to get last char in lexeme");
                        if last == b'/' {
                            lexeme.clear();
                            while self.advance_char() && self.rune != b'\n' {}
                            self.advance_char();
                            continue;
                        } else {
                            self.toks.push(Token {
                                kind: TokenType::None,
                                val: lexeme.clone(),
                                pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                            });
                            lexeme.clear();
                        }
                    }

                    lexeme.push(self.rune);
                },
                b';' | b'+' | b'-' | b'*' | b'(' | b')' | b'{' | b'}' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                        });
                        lexeme.clear();
                    }

                    lexeme.push(self.rune);
                    self.toks.push(Token {
                        kind: TokenType::None,
                        val: lexeme.clone(),
                        pos: Pos { row: self.pos.row, col: self.pos.col },
                    });
                    lexeme.clear();
                },
                b'>' | b'<' | b'~' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                        });
                        lexeme.clear();
                    }
                    lexeme.push(self.rune);
                },
                b'=' => {
                    if !lexeme.is_empty() {
                        let last: &u8 = lexeme.last().expect("Error: Failed to get last char in lexeme");
                        if matches!(last, b'>' | b'<' | b'=' | b'~') {
                            lexeme.push(self.rune);
                        } else {
                            self.toks.push(Token {
                                kind: TokenType::None,
                                val: lexeme.clone(),
                                pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                            });
                            lexeme.clear();
                            lexeme.push(self.rune);
                        }
                    } else {
                        lexeme.push(self.rune);
                    }
                },
                b'&' | b'|' => {
                    if !lexeme.is_empty() {
                        let last: &u8 = lexeme.last().expect("Error: Failed to get last char in lexeme");
                        if matches!(last, b'&' | b'|') {
                            lexeme.push(self.rune);
                        }
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                        });
                        lexeme.clear();
                    } else {
                        lexeme.push(self.rune);
                    }
                },
                b'A'..=b'z' | b'0'..=b'9' => {
                    if !lexeme.is_empty() {
                        let last: &u8 = lexeme.last().expect("Error: Failed to get last char in lexeme");
                        if matches!(last, b'A'..=b'z' | b'0'..=b'9') {
                            lexeme.push(self.rune);
                        } else {
                            self.toks.push(Token {
                                kind: TokenType::None,
                                val: lexeme.clone(),
                                pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                            });
                            lexeme.clear();
                            lexeme.push(self.rune);
                        }
                    } else {
                        lexeme.push(self.rune);
                    }
                },
                _ => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() },
                        });
                        lexeme.clear();
                    }
                    lexeme.push(self.rune);
                }
            }

            if !self.advance_char() {
                break;
            }
        }

        self.cur = 0;
    }

    fn advance_char(&mut self) -> bool {
        self.cur += 1;
        self.pos.col += 1;
        if self.cur >= self.src.len() {
            self.rune = 0;
            false
        } else {
            if self.rune == b'\n' {
                self.pos.row += 1;
                self.pos.col = 0;
            }
            self.rune = *self.src.get(self.cur).unwrap_or_else(|| panic!("{} Error: Failed to advance to next rune", self.pos));
            true
        }
    }

    pub fn lex(&mut self) {
        for tok in &mut self.toks {
            let len: usize = tok.val.len();
            let first: &u8 = tok.val.first().unwrap_or_else(|| panic!("{} Error: Failed to get first char in token", tok.pos));

            if len == 0 {
                panic!("{} Error: Cannot have an empty token", tok.pos);
            } else if len == 1 {
                match first { // First try to match using first byte
                    b'+' => tok.kind = TokenType::OpPlus,
                    b'-' => tok.kind = TokenType::OpMinus,
                    b'*' => tok.kind = TokenType::OpMul,
                    b'/' => tok.kind = TokenType::OpDiv,
                    b'=' => tok.kind = TokenType::OpAssign,
                    b'(' => tok.kind = TokenType::OpenParen,
                    b')' => tok.kind = TokenType::CloseParen,
                    b'>' => tok.kind = TokenType::OpGreaterThan,
                    b'<' => tok.kind = TokenType::OpLessThan,
                    b'{' => tok.kind = TokenType::OpenScope,
                    b'}' => tok.kind = TokenType::CloseScope,
                    b';' => tok.kind = TokenType::End,
                    b'0'..=b'9' => tok.kind = TokenType::LiteralInt,
                    b'A'..=b'z' => tok.kind = TokenType::Identifier,
                    _ => panic!("{} Error: Invalid token `{}`", tok.pos, tok.val_str()),
                }
            } else {
                match tok.val_str().as_str() {
                    "=="       => tok.kind = TokenType::OpEqual,
                    "~="       => tok.kind = TokenType::OpNotEqual,
                    ">="       => tok.kind = TokenType::OpGreaterEqual,
                    "<="       => tok.kind = TokenType::OpLessEqual,
                    "&&"       => tok.kind = TokenType::OpLogicalAnd,
                    "||"       => tok.kind = TokenType::OpLogicalOr,
                    "exit"     => tok.kind = TokenType::KeywordExit,
                    "func"     => tok.kind = TokenType::KeywordFunctionDecl,
                    "dump"     => tok.kind = TokenType::KeywordDebugDump,
                    "if"       => tok.kind = TokenType::KeywordIf,
                    "else"     => tok.kind = TokenType::KeywordElse,
                    "let"      => tok.kind = TokenType::KeywordVariableDecl,
                    "for"      => tok.kind = TokenType::KeywordFor,
                    "while"    => tok.kind = TokenType::KeywordWhile,
                    "continue" => tok.kind = TokenType::KeywordContinue,
                    "break"    => tok.kind = TokenType::KeywordBreak,
                    _ => { // Then match variable contents of words
                        if tok.val.iter().all(|c| c.is_ascii_digit()) {
                            tok.kind = TokenType::LiteralInt;
                        } else if first.is_ascii_alphabetic() {
                            tok.kind = TokenType::Identifier;
                        } else {
                            panic!("{} Error: Invalid token `{}`", tok.pos, tok.val_str());
                        }
                    }
                }
            }
        }
    }
}
