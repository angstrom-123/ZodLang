use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
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
    OpDereference,
    KeywordReturn,
    KeywordIf,
    KeywordElse,
    KeywordFor,
    KeywordBreak,
    KeywordContinue,
    KeywordWhile,
    TypeInt64,
    TypeInt64Ptr,
    End,
    Separator,
    OpenParen,
    CloseParen,
    OpenScope,
    CloseScope,
    Identifier,
    Int,
    Pointer,
}
impl TokenType {
    pub fn val_str(&self) -> &'static str {
        match self {
            TokenType::None             => "",
            TokenType::OpPlus           => "+",
            TokenType::OpMinus          => "-",
            TokenType::OpMul            => "*",
            TokenType::OpDiv            => "/",
            TokenType::OpAssign         => "=",
            TokenType::OpEqual          => "==",
            TokenType::OpNotEqual       => "!=",
            TokenType::OpGreaterThan    => ">",
            TokenType::OpLessThan       => "<",
            TokenType::OpGreaterEqual   => ">=",
            TokenType::OpLessEqual      => "<=",
            TokenType::OpLogicalOr      => "||",
            TokenType::OpLogicalAnd     => "&&",
            TokenType::OpDereference    => "@",
            TokenType::KeywordReturn    => "return",
            TokenType::KeywordIf        => "if",
            TokenType::KeywordElse      => "else",
            TokenType::KeywordFor       => "for",
            TokenType::KeywordBreak     => "break",
            TokenType::KeywordContinue  => "continue",
            TokenType::KeywordWhile     => "while",
            TokenType::TypeInt64        => "i64",
            TokenType::TypeInt64Ptr     => "i64^",
            TokenType::End              => ";",
            TokenType::Separator        => ",",
            TokenType::OpenParen        => "(",
            TokenType::CloseParen       => ")",
            TokenType::OpenScope        => "{",
            TokenType::CloseScope       => "}",
            TokenType::Identifier       => "identifier",
            TokenType::Int              => "literal int",
            TokenType::Pointer          => "literal pointer",
        }
    }
}

#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(Clone)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}
impl fmt::Display for Pos {
    // NOTE: Stored row and column are indices starting from 0, whereas in files, we count from 1.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.row == usize::MAX || self.col == usize::MAX {
            return write!(f, "[NULL     ]");
        }
        write!(f, "[{:>4}:{:>4}]", self.row + 1, self.col + 1)
    }
}

impl Pos {
    pub fn as_vec(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        res.append(&mut self.row.to_string().into_bytes());
        res.push(b'_');
        res.append(&mut self.col.to_string().into_bytes());
        res
    }
}

#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
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

    pub fn pos(&self) -> Pos {
        Pos {
            row: self.pos.row + 1,
            col: self.pos.col + 1,
        }
    }

    pub fn null() -> Self {
        Token {
            kind: TokenType::None,
            val: vec![],
            pos: Pos { row: usize::MAX, col: usize::MAX },
        }
    }

    pub fn null_at(pos: Pos) -> Self {
        Token {
            kind: TokenType::None,
            val: vec![],
            pos,
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

    pub fn peek_next_token(&mut self) -> Token {
        let tok = self.toks.get(self.cur + 1).expect("Error: Lexer failed to peek next token");
        tok.clone()
    }

    pub fn expect_tokens(&mut self, kinds: Vec<TokenType>) -> Token {
        let tok = self.consume_token();
        if kinds.contains(&tok.kind) {
            return tok;
        }

        eprintln!("{} Error: Expected one of the following but got `{}`",  tok.pos, tok.val_str());
        for kind in kinds {
            eprintln!("    `{}`", kind.val_str());
        }
        panic!("");
    }

    pub fn expect_token(&mut self, kind: TokenType) -> Token {
        let tok = self.consume_token();
        if tok.kind == kind {
            return tok;
        }

        panic!("{} Error: Expected `{}` but got `{}`",  tok.pos, kind.val_str(), tok.val_str());
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
                b'@' | b';' | b'+' | b'-' | b'*' | b'(' | b')' | b'{' | b'}' => {
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
                b'>' | b'<' | b'!' => {
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
                        if matches!(last, b'>' | b'<' | b'=' | b'!') {
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
                    b'>' => tok.kind = TokenType::OpGreaterThan,
                    b'<' => tok.kind = TokenType::OpLessThan,
                    b'@' => tok.kind = TokenType::OpDereference,
                    b'(' => tok.kind = TokenType::OpenParen,
                    b')' => tok.kind = TokenType::CloseParen,
                    b'{' => tok.kind = TokenType::OpenScope,
                    b'}' => tok.kind = TokenType::CloseScope,
                    b';' => tok.kind = TokenType::End,
                    b',' => tok.kind = TokenType::Separator,
                    b'0'..=b'9' => tok.kind = TokenType::Int,
                    b'A'..=b'z' => tok.kind = TokenType::Identifier,
                    _ => panic!("{} Error: Invalid token `{}`", tok.pos, tok.val_str()),
                }
            } else {
                match tok.val_str().as_str() {
                    // Ops
                    "=="       => tok.kind = TokenType::OpEqual,
                    "!="       => tok.kind = TokenType::OpNotEqual,
                    ">="       => tok.kind = TokenType::OpGreaterEqual,
                    "<="       => tok.kind = TokenType::OpLessEqual,
                    "&&"       => tok.kind = TokenType::OpLogicalAnd,
                    "||"       => tok.kind = TokenType::OpLogicalOr,
                    // Keywords
                    "if"       => tok.kind = TokenType::KeywordIf,
                    "else"     => tok.kind = TokenType::KeywordElse,
                    "for"      => tok.kind = TokenType::KeywordFor,
                    "while"    => tok.kind = TokenType::KeywordWhile,
                    "continue" => tok.kind = TokenType::KeywordContinue,
                    "break"    => tok.kind = TokenType::KeywordBreak,
                    "return"   => tok.kind = TokenType::KeywordReturn,
                    // Types
                    "i64"      => tok.kind = TokenType::TypeInt64,
                    "i64^"     => tok.kind = TokenType::TypeInt64Ptr,
                    _ => { // Then match variable contents of words
                        if tok.val.iter().rev().skip(1).rev().all(|c| c.is_ascii_digit()) {
                            let last: &u8 = tok.val.last().unwrap();
                            match last {
                                b'0'..=b'9' => tok.kind = TokenType::Int,
                                b'p' => tok.kind = TokenType::Pointer,
                                _ => panic!("{} Error: Invalid token `{}` in numeric literal", tok.pos, last)
                            }
                        } else if first.is_ascii_alphabetic() {
                            for c in &tok.val {
                                if matches!(c, b'"' | b'$' | b'%' | b'^' | b'&' | b'~' | b'#' | b'\\' | b',' | b'.' | b'`'| b'!') {
                                    panic!("{} Error: Invalid token `{}` in identifier", tok.pos, String::from_utf8(vec![*c]).expect("Error: Failed to convert char to str"));
                                }
                            }
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
