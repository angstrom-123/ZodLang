use crate::lexer::Lexer;
use crate::lexer::Pos;
use crate::lexer::Token;
use crate::lexer::TokenType;

// <Prog>           : { ( <Func-Decl> | <Include> ) }
// <Include>        : "include" <Literal-String>
// <Var-Decl>       : <Type> <Name> [ "=" <Additive> ]
// <Func-Decl>      : <Type> <Name> "(" [ <Type> <Name> { "," <Type> <Name> } ] ")" "{" { <Statement> } "}"
// <Func-Call>      : <Name> "(" [ <Union> { "," <Union> } ] ")"
// <Ident>          : [ "@" ] <Name> { "[" <Additive "]" }
// <Type>           : "i64" | "i64^" | "chr" | "chr^" | "any^"
// <Assign>         : <Ident> "=" <Union> 
//                  | "@" "(" <Additive> ")" "=" <Union>
// <Unary>          : "-" | "@"
// <Statement>      : "return" <Additive> ";"
//                  | "break" ";"
//                  | "continue" ";"
//                  | "for" [ ( <Var-Decl> | <Assign> ) ] ";" [ <Union> ] ";" ( <Assign> | Func-Call ) "{" { <Statement> } "}"
//                  | "while" <Union> "{" { <Statement> } "}"
//                  | "if" <Union> "{" { <Statement> } "}" [ "else" "{" { <Statement> } "}" ]
//                  | <Var-Decl> ";"
//                  | <Func-Call> ";"
//                  | <Assign> ";"
// <Union>          : <Intersection> [ "||" <Intersection> ]
// <Intersection>   : <Equality>     [ "&&" <Equality> ]
// <Equality>       : <Relational> [ ( "==" | "!=" ) <Relational> ]
// <Relational>     : <Additive>   [ ( "<" | ">" | "<=" | ">=" ) <Additive> ]
// <Additive>       : <Term>       [ ( "+" | "-" ) <Term> ]
// <Term>           : <Factor>     [ ( "*" | "/" ) <Factor> ]
// <Factor>         : <Literal-Int> 
//                  | <Literal-String> 
//                  | <Literal-Char>
//                  | <Func-Call> 
//                  | <Syscall> 
//                  | "(" <Union> ")" 
//                  | <Unary> <Factor> 
//                  | <Ident>

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
pub enum DataType {
    None,
    Unknown,
    Void,
    I64,
    I64Ptr,
    Chr,
    ChrPtr,
    AnyPtr,
}
impl DataType {
    pub fn from_token(tok: &Token) -> DataType {
        match tok.kind {
            TokenType::TypeVoid => DataType::Void,
            TokenType::TypeInt64 => DataType::I64,
            TokenType::TypeInt64Ptr => DataType::I64Ptr,
            TokenType::TypeChr => DataType::Chr,
            TokenType::TypeChrPtr => DataType::ChrPtr,
            TokenType::TypeAnyPtr => DataType::AnyPtr,
            _ => panic!("{} Error: Failed to convert token `{}` to datatype", tok.pos, tok.val_str())
        }
    }

    pub fn size(&self) -> i64 {
        match self {
            DataType::I64    => 8,
            DataType::I64Ptr => 8,
            DataType::Chr    => 1,
            DataType::ChrPtr => 8,
            DataType::AnyPtr => 8,
            _ => -1,
        }
    }

    pub fn is_assignable(&self, other: &DataType) -> bool {
        match self {
            DataType::I64    => matches!(other, DataType::I64),
            DataType::I64Ptr => matches!(other, DataType::I64Ptr | DataType::AnyPtr),
            DataType::Chr    => matches!(other, DataType::Chr),
            DataType::ChrPtr => matches!(other, DataType::ChrPtr | DataType::AnyPtr),
            DataType::AnyPtr => matches!(other, DataType::ChrPtr | DataType::I64Ptr | DataType::AnyPtr),
            _ => false
        }
    }

    pub fn is_compatible(&self, other: &DataType) -> bool {
        match self {
            DataType::I64    => matches!(other, DataType::AnyPtr | DataType::I64),
            DataType::I64Ptr => matches!(other, DataType::AnyPtr | DataType::I64Ptr | DataType::I64),
            DataType::Chr    => matches!(other, DataType::Chr),
            DataType::ChrPtr => matches!(other, DataType::AnyPtr | DataType::ChrPtr | DataType::I64),
            DataType::AnyPtr => matches!(other, DataType::AnyPtr | DataType::ChrPtr | DataType::I64Ptr | DataType::I64),
            DataType::Void   => matches!(other, DataType::Void),
            _ => false
        }
    }

    pub fn base_type(&self) -> Option<DataType> {
        match self {
            DataType::I64Ptr => Some(DataType::I64),
            DataType::ChrPtr => Some(DataType::Chr),
            _ => None
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum NodeType {
    Null,
    Group,
    Include,
    Var,
    LiteralInt,
    LiteralString,
    LiteralChar,
    FuncCall,
    Syscall,
    FuncDecl,
    VarDecl,
    Assign,
    Conditional,
    ForLoop,
    WhileLoop,
    Return,
    Continue,
    Break,
    BinaryOp,
    UnaryOp,
}

#[derive(Clone)]
pub struct ParseNode {
    pub kind:     NodeType,
    pub datatype: DataType,
    pub tok:      Token,
    pub children: Vec<ParseNode>,
}
impl ParseNode {
    pub fn is_null(&self) -> bool {
        self.kind == NodeType::Null
    }

    pub fn dump(&self, _depth: usize) {
        eprintln!("{}: {:padding$}\x1b[94m* {:?}\x1b[0m (\x1b[92m{:?}::{:?}\x1b[0m: `{}`)", self.tok.pos, "", self.kind, self.tok.kind, self.datatype, self.tok.val_str(), padding = _depth);

        for child in &self.children {
            child.dump(_depth + 4);
        }
    }

    pub fn exclusive_post_order(&self) -> Vec<ParseNode> {
        let mut res: Vec<ParseNode> = self.post_order();
        res.pop();
        res
    }

    pub fn post_order(&self) -> Vec<ParseNode> {
        let mut res: Vec<ParseNode> = Vec::new();
        for node in &self.children {
            res.append(&mut node.post_order());
        }
        res.push(self.clone());
        res
    }

    fn _zero_literal() -> Self {
        let mut tok: Token = Token::null();
        tok.val = vec![b'0'];
        ParseNode {
            kind: NodeType::LiteralInt,
            datatype: DataType::I64,
            tok,
            children: vec![],
        }
    }

    fn _null() -> Self {
        ParseNode {
            kind: NodeType::Null,
            datatype: DataType::None,
            tok: Token::null(),
            children: vec![],
        }
    }

    fn _group(nodes: Vec<ParseNode>) -> Self {
        let tok: Token = if let Some(node) = nodes.first() {
            Token::null_at(node.tok.pos.clone())
        } else {
            Token::null()
        };
        ParseNode {
            kind: NodeType::Group,
            datatype: DataType::None,
            tok,
            children: nodes
        }
    }

    fn _literal_int(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::LiteralInt,
            datatype: DataType::I64,
            tok,
            children: vec![]
        }
    }

    fn _literal_string(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::LiteralString,
            datatype: DataType::ChrPtr,
            tok,
            children: vec![]
        }
    }

    fn _literal_char(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::LiteralChar,
            datatype: DataType::Chr,
            tok,
            children: vec![]
        }
    }

    fn _include(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::Include,
            datatype: DataType::None,
            tok,
            children: vec![rhs],
        }
    }

    fn _continue(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::Continue,
            datatype: DataType::None,
            tok,
            children: vec![]
        }
    }
    
    fn _break(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::Break,
            datatype: DataType::None,
            tok,
            children: vec![]
        }
    }

    fn _return(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::Return,
            datatype: DataType::Unknown,
            tok,
            children: vec![rhs]
        }
    }

    fn _var_decl(datatype: DataType, ident: Token) -> Self {
        ParseNode {
            kind: NodeType::VarDecl,
            datatype,
            tok: ident,
            children: vec![],
        }
    }

    fn _var(ident: Token) -> Self {
        ParseNode {
            kind: NodeType::Var,
            datatype: DataType::Unknown,
            tok: ident,
            children: vec![],
        }
    }

    fn _func_decl(datatype: DataType, ident: Token, args: Vec<ParseNode>, body: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::FuncDecl,
            datatype,
            tok: ident,
            children: vec![Self::_group(args), Self::_group(body)],
        }
    }

    fn _syscall(ident: Token, args: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::Syscall,
            datatype: DataType::I64,
            tok: ident,
            children: args,
        }
    }

    fn _func_call(ident: Token, args: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::FuncCall,
            datatype: DataType::Unknown,
            tok: ident,
            children: args,
        }
    }

    fn _binary_op(operator: Token, lhs: ParseNode, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::BinaryOp,
            datatype: DataType::None,
            tok: operator,
            children: vec![lhs, rhs],
        }
    }

    fn _unary_op(operator: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::UnaryOp,
            datatype: DataType::None,
            tok: operator,
            children: vec![rhs],
        }
    }

    fn _assign(operator: Token, lhs: ParseNode, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::Assign,
            datatype: DataType::None,
            tok: operator,
            children: vec![lhs, rhs],
        }
    }

    fn _conditional(ident: Token, cond: ParseNode, if_body: Vec<ParseNode>, else_body: Vec<ParseNode>) -> Self {
        let mut res: ParseNode = ParseNode {
            kind: NodeType::Conditional,
            datatype: DataType::None,
            tok: ident,
            children: vec![cond, Self::_group(if_body)],
        };

        if !else_body.is_empty() {
            res.children.push(Self::_group(else_body));
        }

        res
    }

    fn _for(ident: Token, decl: Option<ParseNode>, init: Option<ParseNode>, cond: Option<ParseNode>, post: Option<ParseNode>, body: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::ForLoop,
            datatype: DataType::None,
            tok: ident,
            children: vec![
                decl.unwrap_or(Self::_null()),
                init.unwrap_or(Self::_null()),
                cond.unwrap_or(Self::_null()),
                post.unwrap_or(Self::_null()),
                Self::_group(body)
            ],
        }
    }

    fn _while(ident: Token, cond: ParseNode, body: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::WhileLoop,
            datatype: DataType::None,
            tok: ident,
            children: vec![cond, Self::_group(body)],
        }
    }
}

pub enum Precedence {
    Union           = 0,
    Intersection    = 1,
    Equality        = 2,
    Relational      = 3,
    Additive        = 4,
    Term            = 5,
    Factor          = 6,
    _Count
}

pub struct ParseTree<'a> {
    pub root: ParseNode,
    pub precedence: &'a[&'a[TokenType]; Precedence::_Count as usize],
}
impl<'a> Default for ParseTree<'a> {
    fn default() -> Self {
        Self::new()
    }
}
impl <'a> ParseTree<'a> {
    pub fn new() -> Self {
        let root: ParseNode = ParseNode::_group(Vec::new());
        let precedence: &[&[TokenType]; Precedence::_Count as usize] = &[
            &[TokenType::OpLogicalOr],
            &[TokenType::OpLogicalAnd],
            &[TokenType::OpEqual, TokenType::OpNotEqual],
            &[TokenType::OpGreaterThan, TokenType::OpGreaterEqual, TokenType::OpLessThan, TokenType::OpLessEqual],
            &[TokenType::OpPlus, TokenType::OpMinus],
            &[TokenType::OpMul, TokenType::OpDiv],
            &[]
        ];
        ParseTree { root, precedence }
    }

    // NOTE: Currently hard coded intrinsice (dump, exit, mmap, munmap) are entered here as if 
    // they were regular functions, this lets the type checker pass.
    pub fn construct(&mut self, lexer: &mut Lexer) {
        let mut children: Vec<ParseNode> = Vec::new();
        let exit = ParseNode::_func_decl(DataType::Void, 
                                         Token { 
                                             kind: TokenType::Identifier, 
                                             val: Vec::from(b"exit"),
                                             pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("Hardcoded") } 
                                         },
                                         vec![ParseNode::_var_decl(DataType::I64, Token::null())], 
                                         vec![]);
        let dump = ParseNode::_func_decl(DataType::Void, 
                                         Token { 
                                             kind: TokenType::Identifier, 
                                             val: Vec::from(b"dump"),
                                             pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("Hardcoded") } 
                                         },
                                         vec![ParseNode::_var_decl(DataType::I64, Token::null())], 
                                         vec![]);
        let mmap = ParseNode::_func_decl(DataType::AnyPtr, 
                                         Token { 
                                             kind: TokenType::Identifier, 
                                             val: Vec::from(b"mmap"),
                                             pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("Hardcoded") } 
                                         },
                                         vec![ParseNode::_var_decl(DataType::I64, Token::null())], 
                                         vec![]);
        let munmap = ParseNode::_func_decl(DataType::I64, 
                                           Token { 
                                               kind: TokenType::Identifier, 
                                               val: Vec::from(b"munmap"),
                                               pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("Hardcoded") } 
                                           },
                                           vec![ParseNode::_var_decl(DataType::AnyPtr, Token::null()),
                                                ParseNode::_var_decl(DataType::I64, Token::null())], 
                                           vec![]);

        children.append(&mut vec![dump, exit, mmap, munmap]);

        while lexer.has_token() {
            let tok: Token = lexer.peek_token();
            match tok.kind {
                // <Func-Decl> : <Type> <Var> "(" [ <Type> <Var> { "," <Type> <Var> } ] ")" "{" { <Statement> } "}"
                TokenType::TypeVoid
                | TokenType::TypeInt64 
                | TokenType::TypeInt64Ptr 
                | TokenType::TypeChr 
                | TokenType::TypeChrPtr 
                | TokenType::TypeAnyPtr => {
                    children.push(self.parse_func_decl(lexer));
                },
                // <Include> : "include" <Literal-String>
                TokenType::KeywordInclude => {
                    let tok: Token = lexer.consume_token();
                    let file: Token = lexer.expect_token(TokenType::String);
                    children.push(ParseNode::_include(tok, ParseNode::_literal_string(file)));
                },
                _ => panic!("{} Error: Unexpected top level statement `{}`", tok.pos, tok.val_str())
            }
        }
        self.root.children = children;
    }

    pub fn dump(&self) {
        self.root.dump(0);
    }

    pub fn post_order(&self) -> Vec<ParseNode> {
        self.root.post_order()
    }

    // <Ident> : [ "@" ] <Name> { "[" <Additive "]" }
    fn parse_ident(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut ident: ParseNode;
        if lexer.peek_token().kind == TokenType::OpDereference {
            let deref: Token = lexer.consume_token();
            let name: Token = lexer.expect_token(TokenType::Identifier);
            ident = ParseNode::_unary_op(deref, ParseNode::_var(name));
        } else {
            let name: Token = lexer.expect_token(TokenType::Identifier);
            ident = ParseNode::_var(name);
        }

        if lexer.peek_token().kind == TokenType::OpenSquare {
            // Subscript
            let tok = lexer.consume_token();
            let subscript: Token = Token {
                kind: TokenType::OpSubscript,
                val: Vec::from("[_]"),
                pos: tok.pos,
            };
            let expr: ParseNode = self.parse_expression(lexer, Precedence::Additive as usize);
            lexer.expect_token(TokenType::CloseSquare);
            ident = ParseNode::_binary_op(subscript, ident, expr);
        }

        ident
    }

    // <Assign> : [ "@" ] <Ident> "=" <Union> 
    //          | "@" "(" <Additive> ")" "=" <Union>
    fn parse_assign(&mut self, lexer: &mut Lexer) -> ParseNode {
        if lexer.peek_token().kind == TokenType::OpDereference {
            let deref: Token = lexer.consume_token();
            if lexer.peek_token().kind == TokenType::OpenParen {
                // Assign to address expression
                lexer.consume_token();
                let lhs: ParseNode = self.parse_expression(lexer, Precedence::Additive as usize);
                lexer.expect_token(TokenType::CloseParen);
                let assign: Token = lexer.consume_token();
                let rhs: ParseNode = self.parse_expression(lexer, Precedence::Union as usize);
                ParseNode::_assign(assign, ParseNode::_unary_op(deref, lhs), rhs)
            } else {
                // Assign to variable address
                let lhs: ParseNode = self.parse_ident(lexer);
                let assign: Token = lexer.expect_token(TokenType::OpAssign);
                let rhs: ParseNode = self.parse_expression(lexer, Precedence::Union as usize);
                ParseNode::_assign(assign, ParseNode::_unary_op(deref, lhs), rhs)
            }
        } else {
            // Regular assign
            let lhs: ParseNode = self.parse_ident(lexer);
            let assign: Token = lexer.expect_token(TokenType::OpAssign);
            let rhs: ParseNode = self.parse_expression(lexer, Precedence::Union as usize);
            ParseNode::_assign(assign, lhs, rhs)
        }
    }

    // <Func-Decl> : <Type> <Var> "(" [ <Type> <Var> { "," <Type> <Var> } ] ")" "{" { <Statement> } "}"
    fn parse_func_decl(&mut self, lexer: &mut Lexer) -> ParseNode {
        let datatype: DataType = DataType::from_token(&lexer.expect_type());
        let ident: Token = lexer.expect_token(TokenType::Identifier);

        // Function arguments between parens
        lexer.expect_token(TokenType::OpenParen);

        let mut args: Vec<ParseNode> = Vec::new();
        while lexer.peek_token().kind != TokenType::CloseParen {
            if !args.is_empty() {
                lexer.expect_token(TokenType::Separator);
            }
            let arg_type: DataType = DataType::from_token(&lexer.expect_type());
            let arg_ident: Token = lexer.expect_token(TokenType::Identifier);
            args.push(ParseNode::_var_decl(arg_type, arg_ident));
        }

        lexer.expect_token(TokenType::CloseParen);

        // Function body between scopes
        lexer.expect_token(TokenType::OpenScope);

        let mut body: Vec<ParseNode> = Vec::new();
        while lexer.peek_token().kind != TokenType::CloseScope {
            body.extend(self.parse_statement(lexer));
        }

        lexer.expect_token(TokenType::CloseScope);

        ParseNode::_func_decl(datatype, ident, args, body)
    }

    // <Syscall> : <Ident> "(" [ <Union> { "," <Union> } ] ")"
    fn parse_syscall(&mut self, lexer: &mut Lexer) -> ParseNode {
        let ident: Token = lexer.expect_token(TokenType::Syscall);

        // Function arguments between parens
        lexer.expect_token(TokenType::OpenParen);

        let mut args: Vec<ParseNode> = Vec::new();
        while lexer.peek_token().kind != TokenType::CloseParen {
            if !args.is_empty() {
                lexer.expect_token(TokenType::Separator);
            }
            args.push(self.parse_expression(lexer, Precedence::Union as usize));
        }

        lexer.expect_token(TokenType::CloseParen);

        args.reverse();
        ParseNode::_syscall(ident, args)
    }

    // <Func-Call> : <Ident> "(" [ <Union> { "," <Union> } ] ")"
    fn parse_func_call(&mut self, lexer: &mut Lexer) -> ParseNode {
        let ident: Token = lexer.expect_token(TokenType::Identifier);

        // Function arguments between parens
        lexer.expect_token(TokenType::OpenParen);

        let mut args: Vec<ParseNode> = Vec::new();
        while lexer.peek_token().kind != TokenType::CloseParen {
            if !args.is_empty() {
                lexer.expect_token(TokenType::Separator);
            }
            args.push(self.parse_expression(lexer, Precedence::Union as usize));
        }

        lexer.expect_token(TokenType::CloseParen);

        args.reverse();
        ParseNode::_func_call(ident, args)
    }

    fn parse_var_decl(&mut self, lexer: &mut Lexer) -> (ParseNode, ParseNode) {
        let datatype: DataType = DataType::from_token(&lexer.expect_type());
        let ident: Token = lexer.peek_token();
        let lhs: ParseNode = ParseNode::_var_decl(datatype, ident.clone());
        if ident.kind != TokenType::Identifier {
            panic!("{} Error: Expected `{}` but got `{}`",  ident.pos, TokenType::Identifier.val_str(), ident.val_str());
        }

        // Either explicit assignment, or implicit zero assignment
        if lexer.peek_next_token().kind == TokenType::OpAssign {
            (lhs, self.parse_assign(lexer))
        } else {
            lexer.consume_token();
            let operator: Token = Token {
                kind: TokenType::OpAssign,
                val: Vec::from("="),
                pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("Implicit") },
            };
            (lhs, ParseNode::_assign(operator, ParseNode::_var(ident), ParseNode::_zero_literal()))
        }
    }

    // <Union>          : <Intersection> [ "||" <Intersection> ]
    // <Intersection>   : <Equality>     [ "&&" <Equality> ]
    // <Equality>       : <Relational>   [ ( "==" | "!=" ) <Relational> ]
    // <Relational>     : <Additive>     [ ( "<" | ">" | "<=" | ">=" ) <Additive> ]
    // <Additive>       : <Term>         [ ( "+" | "-" ) <Term> ]
    // <Term>           : <Factor>       [ ( "*" | "/" ) <Factor> ]
    fn parse_expression(&mut self, lexer: &mut Lexer, level: usize) -> ParseNode {
        if level == Precedence::Factor as usize {
            return self.parse_factor(lexer);
        }
        let mut lhs = self.parse_expression(lexer, level + 1);

        while self.precedence[level].contains(&lexer.peek_token().kind) {
            let operator: Token = lexer.consume_token();
            lhs = ParseNode::_binary_op(operator, lhs, self.parse_expression(lexer, level + 1));
        }

        lhs
    }

    // <Factor> : <Literal-Int> 
    //          | <Literal-String> 
    //          | <Func-Call> 
    //          | <Syscall> 
    //          | "(" <Union> ")" 
    //          | <Unary> <Factor> 
    //          | <Ident>
    //          | <Ident> [ "[" <Additive> "]" ]
    fn parse_factor(&mut self, lexer: &mut Lexer) -> ParseNode {
        match lexer.peek_token().kind {
            TokenType::Int => ParseNode::_literal_int(lexer.consume_token()),
            TokenType::String => ParseNode::_literal_string(lexer.consume_token()),
            TokenType::Char => ParseNode::_literal_char(lexer.consume_token()),
            TokenType::Syscall => self.parse_syscall(lexer),
            TokenType::Identifier => {
                if lexer.peek_next_token().kind == TokenType::OpenParen {
                    self.parse_func_call(lexer)
                } else {
                    self.parse_ident(lexer)
                }
            },
            TokenType::OpenParen => {
                lexer.consume_token();
                let expr: ParseNode = self.parse_expression(lexer, Precedence::Union as usize);
                lexer.expect_token(TokenType::CloseParen);
                expr
            },
            TokenType::OpMinus | TokenType::OpDereference => {
                ParseNode::_unary_op(lexer.consume_token(), self.parse_factor(lexer))
            },
            _ => panic!("{} Error: Expected factor but got `{}`", lexer.peek_token().pos, lexer.peek_token().val_str())
        }
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> Vec<ParseNode> {
        match lexer.peek_token().kind {
            // "return" <Additive> ";"
            TokenType::KeywordReturn => {
                let expr: ParseNode = ParseNode::_return(lexer.consume_token(), self.parse_expression(lexer, Precedence::Union as usize));
                lexer.expect_token(TokenType::End);
                vec![expr]
            },
            // "break" ";"
            TokenType::KeywordBreak => {
                let expr: ParseNode = ParseNode::_break(lexer.consume_token());
                lexer.expect_token(TokenType::End);
                vec![expr]
            },
            // "continue" ";"
            TokenType::KeywordContinue => {
                let expr: ParseNode = ParseNode::_continue(lexer.consume_token());
                lexer.expect_token(TokenType::End);
                vec![expr]
            },
            // "for" [ ( <Var-Decl> | <Assign> ) ]  ";" [ <Union> ] ";" ( <Assign> | Func-Call ) "{" { <Statement> } "}"
            TokenType::KeywordFor => {
                let ident: Token = lexer.consume_token();

                let mut decl: Option<ParseNode> = None;
                let mut init: Option<ParseNode> = None;
                if matches!(lexer.peek_token().kind, TokenType::TypeVoid | TokenType::TypeInt64Ptr | TokenType::TypeInt64 | TokenType::TypeChrPtr | TokenType::TypeChr | TokenType::TypeAnyPtr) {
                    let (d, i) = self.parse_var_decl(lexer);
                    decl = Some(d);
                    init = Some(i);
                } else if lexer.peek_token().kind != TokenType::End {
                    init = Some(self.parse_assign(lexer));
                }

                lexer.expect_token(TokenType::End);

                let mut cond: Option<ParseNode> = None;
                if lexer.peek_token().kind != TokenType::End {
                    cond = Some(self.parse_expression(lexer, Precedence::Union as usize));
                }

                lexer.expect_token(TokenType::End);

                let mut post: Option<ParseNode> = None;
                if lexer.peek_token().kind != TokenType::OpenScope {
                    if lexer.peek_next_token().kind == TokenType::OpenParen {
                        post = Some(self.parse_func_call(lexer));
                    } else {
                        post = Some(self.parse_assign(lexer));
                    }
                }

                // For body between scopes
                lexer.expect_token(TokenType::OpenScope);

                let mut body: Vec<ParseNode> = Vec::new();
                while lexer.peek_token().kind != TokenType::CloseScope {
                    body.extend(self.parse_statement(lexer));
                }

                lexer.expect_token(TokenType::CloseScope);

                vec![ParseNode::_for(ident, decl, init, cond, post, body)]
            },
            // "while" <Union> "{" { <Statement> } "}"
            TokenType::KeywordWhile => {
                let ident: Token = lexer.consume_token();
                let cond: ParseNode = self.parse_expression(lexer, Precedence::Union as usize);

                // While body between scopes
                lexer.expect_token(TokenType::OpenScope);

                let mut body: Vec<ParseNode> = Vec::new();
                while lexer.peek_token().kind != TokenType::CloseScope {
                    body.extend(self.parse_statement(lexer));
                }

                lexer.expect_token(TokenType::CloseScope);

                vec![ParseNode::_while(ident, cond, body)]
            },
            // "if" <Union> "{" { <Statement> } "}" [ "else" "{" { <Statement> } "}" ]
            TokenType::KeywordIf => {
                let ident: Token = lexer.consume_token();
                let cond: ParseNode = self.parse_expression(lexer, Precedence::Union as usize);

                // If body between scopes
                lexer.expect_token(TokenType::OpenScope);

                let mut if_body: Vec<ParseNode> = Vec::new();
                while lexer.peek_token().kind != TokenType::CloseScope {
                    if_body.extend(self.parse_statement(lexer));
                }

                lexer.expect_token(TokenType::CloseScope);

                let mut else_body: Vec<ParseNode> = Vec::new();
                if lexer.peek_token().kind == TokenType::KeywordElse {
                    lexer.consume_token();
                    // Else body between scopes
                    lexer.expect_token(TokenType::OpenScope);

                    while lexer.peek_token().kind != TokenType::CloseScope {
                        else_body.extend(self.parse_statement(lexer));
                    }

                    lexer.expect_token(TokenType::CloseScope);
                }

                vec![ParseNode::_conditional(ident, cond, if_body, else_body)]
            },
            TokenType::Syscall => {
                let expr: ParseNode = self.parse_syscall(lexer);
                lexer.expect_token(TokenType::End);
                vec![expr]
            },
            // <Func-Call> ";"
            // <Assign> ";"
            TokenType::Identifier => {
                let expr: ParseNode = match lexer.peek_next_token().kind {
                    TokenType::OpenParen => self.parse_func_call(lexer),
                    TokenType::OpAssign 
                    | TokenType::OpenSquare => self.parse_assign(lexer),
                    _ => panic!("{} Error: Expected assignment or function call but got `{}`",
                                lexer.peek_token().pos, lexer.peek_token().val_str())
                };
                lexer.expect_token(TokenType::End);
                vec![expr]
            },
            // <Var-Decl> ";"
            TokenType::TypeVoid 
            | TokenType::TypeInt64
            | TokenType::TypeInt64Ptr
            | TokenType::TypeChr
            | TokenType::TypeChrPtr
            | TokenType::TypeAnyPtr => {
                let (decl, init) = self.parse_var_decl(lexer);
                lexer.expect_token(TokenType::End);
                vec![decl, init]
            },
            // <Assign> ";"
            TokenType::OpDereference => {
                let expr: ParseNode = self.parse_assign(lexer);
                lexer.expect_token(TokenType::End);
                vec![expr]
            },
            _ => panic!("{} Error: Expected statement but got `{}`", lexer.peek_token().pos, lexer.peek_token().val_str())
        }
    }
}
