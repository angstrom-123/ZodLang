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
    OpMod,
    OpEqual,
    OpNotEqual,
    OpGreaterThan,
    OpLessThan,
    OpGreaterEqual,
    OpLessEqual,
    OpLogicalOr,
    OpLogicalAnd,
    OpDereference,
    OpSubscript,
    KeywordReturn,
    KeywordIf,
    KeywordElse,
    KeywordFor,
    KeywordBreak,
    KeywordContinue,
    KeywordWhile,
    KeywordInclude,
    TypeVoid,
    TypeInt64,
    TypeInt64Ptr,
    TypeChr,
    TypeChrPtr,
    TypeAnyPtr,
    Int,
    String,
    Char,
    Syscall,
    End,
    Separator,
    OpenParen,
    CloseParen,
    OpenSquare,
    CloseSquare,
    OpenScope,
    CloseScope,
    Identifier,
}
impl TokenType {
    pub fn val_str(&self) -> &'static str {
        match self {
            TokenType::None                  => "",
            TokenType::OpPlus                => "+",
            TokenType::OpMinus               => "-",
            TokenType::OpMul                 => "*",
            TokenType::OpDiv                 => "/",
            TokenType::OpAssign              => "=",
            TokenType::OpMod                 => "%",
            TokenType::OpEqual               => "==",
            TokenType::OpNotEqual            => "!=",
            TokenType::OpGreaterThan         => ">",
            TokenType::OpLessThan            => "<",
            TokenType::OpGreaterEqual        => ">=",
            TokenType::OpLessEqual           => "<=",
            TokenType::OpLogicalOr           => "||",
            TokenType::OpLogicalAnd          => "&&",
            TokenType::OpDereference         => "@",
            TokenType::OpSubscript           => "[",
            TokenType::KeywordReturn         => "return",
            TokenType::KeywordIf             => "if",
            TokenType::KeywordElse           => "else",
            TokenType::KeywordFor            => "for",
            TokenType::KeywordBreak          => "break",
            TokenType::KeywordContinue       => "continue",
            TokenType::KeywordWhile          => "while",
            TokenType::KeywordInclude        => "include",
            TokenType::TypeVoid              => "void",
            TokenType::TypeInt64             => "i64",
            TokenType::TypeInt64Ptr          => "i64^",
            TokenType::TypeChr               => "chr",
            TokenType::TypeChrPtr            => "chr^",
            TokenType::TypeAnyPtr            => "any^",
            TokenType::Int                   => "literal int",
            TokenType::String                => "literal string",
            TokenType::Char                  => "literal char",
            TokenType::Syscall               => "syscall",
            TokenType::End                   => ";",
            TokenType::Separator             => ",",
            TokenType::OpenParen             => "(",
            TokenType::CloseParen            => ")",
            TokenType::OpenSquare            => "[",
            TokenType::CloseSquare           => "]",
            TokenType::OpenScope             => "{",
            TokenType::CloseScope            => "}",
            TokenType::Identifier            => "identifier",
        }
    }
}

#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(Clone)]
pub struct Pos {
    pub row: u32,
    pub col: u32,
    pub file: String,
}
impl fmt::Display for Pos {
    // NOTE: Stored row and column are indices starting from 0, whereas in files, we count from 1.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.row == u32::MAX || self.col == u32::MAX {
            return write!(f, "[NULL     ]");
        }
        write!(f, "[{:>4}:{:>4}]:{}", self.row + 1, self.col + 1, self.file)
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
    pub val: Vec<u8>,
    pub pos: Pos,
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?} `{}`", self.pos(), self.kind, self.val_str())
    }
}
impl Token {
    pub fn val_str(&self) -> String {
        String::from_utf8(self.val.clone()).expect("Error: Failed to convert token value to string")
    }

    pub fn pos(&self) -> Pos {
        Pos {
            row: self.pos.row + 1,
            col: self.pos.col + 1,
            file: self.pos.file.clone()
        }
    }

    pub fn null() -> Self {
        Token {
            kind: TokenType::None,
            val: vec![],
            pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("NULL") },
        }
    }

    pub fn null_at(pos: Pos) -> Self {
        Token {
            kind: TokenType::None,
            val: vec![],
            pos: pos.clone(),
        }
    }
}

pub struct Lexer {
    pub toks: Vec<Token>,
    pub pos: Pos,
    src: Vec<u8>,
    pub cur: usize,
    pub rune: u8,
}
impl Lexer {
    pub fn new(src: Vec<u8>, file: &str) -> Self {
        let first = *src.first().expect("Error: Provided source file is empty");
        Lexer { 
            toks: Vec::new(),
            pos: Pos { row: 0, col: 0, file: file.to_string() },
            src,
            cur: 0,
            rune: first,
        }
    }

    pub fn dump(&self) {
        for tok in &self.toks {
            eprintln!("{}", tok);
        }
    }

    pub fn refresh(&mut self) {
        self.cur = 0;
        self.rune = *self.src.first().unwrap();
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

    pub fn expect_type(&mut self) -> Token {
        const KINDS: [TokenType; 6] = [
            TokenType::TypeVoid,
            TokenType::TypeInt64,
            TokenType::TypeInt64Ptr,
            TokenType::TypeChr,
            TokenType::TypeChrPtr,
            TokenType::TypeAnyPtr,
        ];

        let tok = self.consume_token();
        if KINDS.contains(&tok.kind) {
            return tok;
        }

        panic!("{} Error: Expected type but got `{}`",  tok.pos, tok.val_str());
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

    pub fn vec_val(v: &Vec<u8>) -> i64 {
        assert!(!v.is_empty(), "Cannot convert empty vector to int");
        let base: i64 = 10;
        let mut mul: i64 = base.pow((v.len() - 1) as u32);
        let mut res: i64 = 0;
        for c in v {
            let digit: u8 = *c - b'0';
            res += digit as i64 * mul;
            mul /= 10;
        }
        res
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
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
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
                                pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
                            });
                            lexeme.clear();
                        }
                    }

                    lexeme.push(self.rune);
                },
                b'\'' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
                        });
                        lexeme.clear();
                    }

                    lexeme.push(self.rune);
                    self.advance_char();
                    if self.rune == b'\\' {
                        self.advance_char();
                        match self.rune {
                            b'\\' => lexeme.push(b'\\'),
                            b'n'  => lexeme.push(b'\n'),
                            b'r'  => lexeme.push(b'\r'),
                            b'\"' => lexeme.push(b'\"'),
                            b'\'' => lexeme.push(b'\''),
                            b'0' => lexeme.push(b'\0'),
                            _ => panic!("{} Error: Invalid escape `\\{}`", self.pos, self.rune)
                        }
                    } else {
                        lexeme.push(self.rune);
                    }

                    self.advance_char();
                    lexeme.push(self.rune);
                    self.toks.push(Token {
                        kind: TokenType::None,
                        val: lexeme.clone(),
                        pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
                    });
                    lexeme.clear();
                },
                b'\"' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
                        });
                        lexeme.clear();
                    }

                    lexeme.push(self.rune);
                    while self.advance_char() {
                        if self.rune == b'\\' {
                            self.advance_char();
                            match self.rune {
                                b'\\' => lexeme.push(b'\\'),
                                b'n'  => lexeme.push(b'\n'),
                                b'r'  => lexeme.push(b'\r'),
                                b'\"' => lexeme.push(b'\"'),
                                b'\'' => lexeme.push(b'\''),
                                b'\0' => lexeme.push(b'\0'),
                                _ => panic!("{} Error: Invalid escape `\\{}`", self.pos, self.rune)
                            }

                            continue;
                        }

                        lexeme.push(self.rune);
                        if self.rune == b'\"' {
                            break;
                        }
                    }
                },
                b'@' | b';' | b'+' | b'*' | b'(' | b')' | b'{' | b'}' | b'[' | b']' | b'%' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
                        });
                        lexeme.clear();
                    }

                    lexeme.push(self.rune);
                    self.toks.push(Token {
                        kind: TokenType::None,
                        val: lexeme.clone(),
                        pos: Pos { row: self.pos.row, col: self.pos.col, file: self.pos.file.clone() },
                    });
                    lexeme.clear();
                },
                b'>' | b'<' | b'!' | b'-' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Token {
                            kind: TokenType::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
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
                                pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
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
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
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
                                pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
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
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
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
                    b'[' => tok.kind = TokenType::OpenSquare,
                    b']' => tok.kind = TokenType::CloseSquare,
                    b'{' => tok.kind = TokenType::OpenScope,
                    b'}' => tok.kind = TokenType::CloseScope,
                    b';' => tok.kind = TokenType::End,
                    b',' => tok.kind = TokenType::Separator,
                    b'%' => tok.kind = TokenType::OpMod,
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
                    "include"  => tok.kind = TokenType::KeywordInclude,
                    // Types
                    "void"     => tok.kind = TokenType::TypeVoid,
                    "i64"      => tok.kind = TokenType::TypeInt64,
                    "i64^"     => tok.kind = TokenType::TypeInt64Ptr,
                    "chr"      => tok.kind = TokenType::TypeChr,
                    "chr^"     => tok.kind = TokenType::TypeChrPtr,
                    "any^"     => tok.kind = TokenType::TypeAnyPtr,
                    // Intrinsics
                    "syscall" => tok.kind = TokenType::Syscall,
                    _ => { // Then match variable contents of words
                        // Allow for negative literals
                        if tok.val.iter().skip(1).all(|c| c.is_ascii_digit()) && (first.is_ascii_digit() || *first == b'-') {
                            tok.kind = TokenType::Int;
                        } else if *first == b'\'' {
                            tok.kind = TokenType::Char;
                            // Strip off speech marks
                            tok.val.pop();
                            tok.val.remove(0);
                        } else if *first == b'\"' {
                            tok.kind = TokenType::String;
                            // Strip off speech marks
                            tok.val.pop();
                            tok.val.remove(0);
                            // Add null terminator
                            tok.val.push(b'\0');
                        } else if first.is_ascii_alphabetic() {
                            for c in &tok.val {
                                if matches!(c, b'"' | b'$' | b'%' | b'^' | b'&' | b'~' | b'#' | 
                                               b'\\' | b',' | b'.' | b'`'| b'!') {
                                    panic!("{} Error: Invalid token `{}` in identifier", 
                                           tok.pos, String::from_utf8(vec![*c]).expect("Error: Failed to convert char to str"));
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
