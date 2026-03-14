use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(Clone)]
pub enum TokKind {
    None,

    Plus,
    Minus,
    Mul,
    Div,
    Assign,
    Mod,
    Equal,
    NotEqual,
    GT,
    LT,
    GE,
    LE,
    LogOr,
    LogAnd,
    Deref,
    Subscript,
    Return,
    If,
    Else,
    For,
    Break,
    Continue,
    While,
    Include,
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
    OParen,
    CParen,
    OSquare,
    CSquare,
    OScope,
    CScope,
    Ident,
}
impl TokKind {
    pub fn val_str(&self) -> &'static str {
        match self {
            TokKind::None         => "",
            TokKind::Plus         => "+",
            TokKind::Minus        => "-",
            TokKind::Mul          => "*",
            TokKind::Div          => "/",
            TokKind::Assign       => "=",
            TokKind::Mod          => "%",
            TokKind::Equal        => "==",
            TokKind::NotEqual     => "!=",
            TokKind::GT           => ">",
            TokKind::LT           => "<",
            TokKind::GE           => ">=",
            TokKind::LE           => "<=",
            TokKind::LogOr        => "||",
            TokKind::LogAnd       => "&&",
            TokKind::Deref        => "@",
            TokKind::Subscript    => "[",
            TokKind::Return       => "return",
            TokKind::If           => "if",
            TokKind::Else         => "else",
            TokKind::For          => "for",
            TokKind::Break        => "break",
            TokKind::Continue     => "continue",
            TokKind::While        => "while",
            TokKind::Include      => "include",
            TokKind::TypeVoid         => "void",
            TokKind::TypeInt64    => "i64",
            TokKind::TypeInt64Ptr => "i64^",
            TokKind::TypeChr      => "chr",
            TokKind::TypeChrPtr   => "chr^",
            TokKind::TypeAnyPtr   => "any^",
            TokKind::Int          => "literal int",
            TokKind::String       => "literal string",
            TokKind::Char         => "literal char",
            TokKind::Syscall      => "syscall",
            TokKind::End          => ";",
            TokKind::Separator    => ",",
            TokKind::OParen       => "(",
            TokKind::CParen       => ")",
            TokKind::OSquare      => "[",
            TokKind::CSquare      => "]",
            TokKind::OScope       => "{",
            TokKind::CScope       => "}",
            TokKind::Ident        => "identifier",
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
pub struct Tok {
    pub kind: TokKind,
    pub val: Vec<u8>,
    pub pos: Pos,
}
impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?} `{}`", self.pos(), self.kind, self.val_str())
    }
}
impl Tok {
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
        Tok {
            kind: TokKind::None,
            val: vec![],
            pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("NULL") },
        }
    }

    pub fn null_at(pos: Pos) -> Self {
        Tok {
            kind: TokKind::None,
            val: vec![],
            pos: pos.clone(),
        }
    }
}

pub struct Lexer {
    pub toks: Vec<Tok>,
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

    pub fn consume_token(&mut self) -> Tok {
        let tok = self.toks.get(self.cur).expect("Error: Lexer failed to consume next token");
        self.cur += 1;
        tok.clone()
    }

    pub fn peek_token(&mut self) -> Tok {
        let tok = self.toks.get(self.cur).expect("Error: Lexer failed to peek next token");
        tok.clone()
    }

    pub fn peek_next_token(&mut self) -> Tok {
        let tok = self.toks.get(self.cur + 1).expect("Error: Lexer failed to peek next token");
        tok.clone()
    }

    pub fn expect_type(&mut self) -> Tok {
        const KINDS: [TokKind; 6] = [
            TokKind::TypeVoid,
            TokKind::TypeInt64,
            TokKind::TypeInt64Ptr,
            TokKind::TypeChr,
            TokKind::TypeChrPtr,
            TokKind::TypeAnyPtr,
        ];

        let tok = self.consume_token();
        if KINDS.contains(&tok.kind) {
            return tok;
        }

        panic!("{} Error: Expected type but got `{}`",  tok.pos, tok.val_str());
    }

    pub fn expect_tokens(&mut self, kinds: Vec<TokKind>) -> Tok {
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

    pub fn expect_token(&mut self, kind: TokKind) -> Tok {
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
                        self.toks.push(Tok {
                            kind: TokKind::None,
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
                            self.toks.push(Tok {
                                kind: TokKind::None,
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
                        self.toks.push(Tok {
                            kind: TokKind::None,
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
                    self.toks.push(Tok {
                        kind: TokKind::None,
                        val: lexeme.clone(),
                        pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
                    });
                    lexeme.clear();
                },
                b'\"' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Tok {
                            kind: TokKind::None,
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
                        self.toks.push(Tok {
                            kind: TokKind::None,
                            val: lexeme.clone(),
                            pos: Pos { row: self.pos.row, col: self.pos.col - lexeme.len() as u32, file: self.pos.file.clone() },
                        });
                        lexeme.clear();
                    }

                    lexeme.push(self.rune);
                    self.toks.push(Tok {
                        kind: TokKind::None,
                        val: lexeme.clone(),
                        pos: Pos { row: self.pos.row, col: self.pos.col, file: self.pos.file.clone() },
                    });
                    lexeme.clear();
                },
                b'>' | b'<' | b'!' | b'-' => {
                    if !lexeme.is_empty() {
                        self.toks.push(Tok {
                            kind: TokKind::None,
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
                            self.toks.push(Tok {
                                kind: TokKind::None,
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
                        self.toks.push(Tok {
                            kind: TokKind::None,
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
                            self.toks.push(Tok {
                                kind: TokKind::None,
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
                        self.toks.push(Tok {
                            kind: TokKind::None,
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
                    b'+' => tok.kind = TokKind::Plus,
                    b'-' => tok.kind = TokKind::Minus,
                    b'*' => tok.kind = TokKind::Mul,
                    b'/' => tok.kind = TokKind::Div,
                    b'=' => tok.kind = TokKind::Assign,
                    b'>' => tok.kind = TokKind::GT,
                    b'<' => tok.kind = TokKind::LT,
                    b'@' => tok.kind = TokKind::Deref,
                    b'(' => tok.kind = TokKind::OParen,
                    b')' => tok.kind = TokKind::CParen,
                    b'[' => tok.kind = TokKind::OSquare,
                    b']' => tok.kind = TokKind::CSquare,
                    b'{' => tok.kind = TokKind::OScope,
                    b'}' => tok.kind = TokKind::CScope,
                    b';' => tok.kind = TokKind::End,
                    b',' => tok.kind = TokKind::Separator,
                    b'%' => tok.kind = TokKind::Mod,
                    b'0'..=b'9' => tok.kind = TokKind::Int,
                    b'A'..=b'z' => tok.kind = TokKind::Ident,
                    _ => panic!("{} Error: Invalid token `{}`", tok.pos, tok.val_str()),
                }
            } else {
                match tok.val_str().as_str() {
                    // Ops
                    "=="       => tok.kind = TokKind::Equal,
                    "!="       => tok.kind = TokKind::NotEqual,
                    ">="       => tok.kind = TokKind::GE,
                    "<="       => tok.kind = TokKind::LE,
                    "&&"       => tok.kind = TokKind::LogAnd,
                    "||"       => tok.kind = TokKind::LogOr,
                    // Keywords
                    "if"       => tok.kind = TokKind::If,
                    "else"     => tok.kind = TokKind::Else,
                    "for"      => tok.kind = TokKind::For,
                    "while"    => tok.kind = TokKind::While,
                    "continue" => tok.kind = TokKind::Continue,
                    "break"    => tok.kind = TokKind::Break,
                    "return"   => tok.kind = TokKind::Return,
                    "include"  => tok.kind = TokKind::Include,
                    // Types
                    "void"     => tok.kind = TokKind::TypeVoid,
                    "i64"      => tok.kind = TokKind::TypeInt64,
                    "i64^"     => tok.kind = TokKind::TypeInt64Ptr,
                    "chr"      => tok.kind = TokKind::TypeChr,
                    "chr^"     => tok.kind = TokKind::TypeChrPtr,
                    "any^"     => tok.kind = TokKind::TypeAnyPtr,
                    // Intrinsics
                    "syscall" => tok.kind = TokKind::Syscall,
                    _ => { // Then match variable contents of words
                        // Allow for negative literals
                        if tok.val.iter().skip(1).all(|c| c.is_ascii_digit()) && (first.is_ascii_digit() || *first == b'-') {
                            tok.kind = TokKind::Int;
                        } else if *first == b'\'' {
                            tok.kind = TokKind::Char;
                            // Strip off speech marks
                            tok.val.pop();
                            tok.val.remove(0);
                        } else if *first == b'\"' {
                            tok.kind = TokKind::String;
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
                            tok.kind = TokKind::Ident;
                        } else {
                            panic!("{} Error: Invalid token `{}`", tok.pos, tok.val_str());
                        }
                    }
                }
            }
        }
    }
}
