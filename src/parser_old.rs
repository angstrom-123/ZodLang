use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenType;
use crate::lexer::Pos;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum DataType {
    None,
    Unknown,
    Int64,
    Int64Ptr,
}
impl DataType {
    pub fn from_tok_type(tok_type: &TokenType) -> Self {
        match tok_type {
            TokenType::TypeInt64 => DataType::Int64,
            TokenType::TypeInt64Ptr => DataType::Int64Ptr,
            _ => panic!("Error: Failed to convert token type `{:?}` to data type", tok_type)
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum NodeType {
    Null,
    Program,
    Block,
    Return,
    Exit,
    MMap,
    FuncDecl,
    FuncCall,
    VarDecl,
    Var,
    DerefAssign,
    Assign,
    DebugDump,
    BinOp,
    UnOp,
    Conditional,
    ForLoop,
    WhileLoop,
    LiteralInt,
    Pointer,
    Break,
    Continue,
}

#[derive(Clone)]
pub struct ParseNode {
    pub kind:     NodeType,
    pub datatype: DataType,
    pub tok:      Token,
    pub children: Vec<ParseNode>,
}
impl ParseNode {
    pub fn dump(&self, _depth: usize) {
        let tok_str: String = String::from_utf8(self.tok.val.clone()).expect("Error: Failed to convert token value to string");
        eprintln!("{}: {:padding$}\x1b[94m* {:?}\x1b[0m (\x1b[92m{:?}::{:?}\x1b[0m: `{}`)", self.tok.pos, "", self.kind, self.tok.kind, self.datatype, tok_str, padding = _depth);

        for child in &self.children {
            child.dump(_depth + 4);
        }
    }

    pub fn exclusive_post_order(&self) -> Vec<ParseNode> {
        let mut res: Vec<ParseNode> = Vec::new();
        for node in &self.children {
            res.append(&mut node.post_order());
        }
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

    fn new_program(prog_name: String, prog: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::Program,
            datatype: DataType::None,
            tok: Token {
                kind: TokenType::None,
                val: prog_name.into_bytes(),
                pos: Pos { row: usize::MAX, col: usize::MAX },
            },
            children: prog
        }
    }

    fn new_mmap(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::MMap,
            datatype: DataType::None,
            tok,
            children: vec![rhs],
        }
    }

    fn new_exit(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::Exit,
            datatype: DataType::None,
            tok,
            children: vec![rhs],
        }
    }
    
    fn new_return(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::Return,
            datatype: DataType::None,
            tok,
            children: vec![rhs],
        }
    }

    fn new_debug_dump(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::DebugDump,
            datatype: DataType::None,
            tok,
            children: vec![rhs],
        }
    }

    fn new_bin_op(tok: Token, lhs: ParseNode, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::BinOp,
            datatype: DataType::None,
            tok,
            children: vec![lhs, rhs],
        }
    }

    fn new_un_op(tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::UnOp,
            datatype: DataType::None,
            tok,
            children: vec![rhs],
        }
    }

    fn new_literal(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::LiteralInt,
            datatype: DataType::None,
            tok,
            children: Vec::new(),
        }
    }

    fn new_func_decl(ident_tok: Token, params: Vec<ParseNode>, body: ParseNode) -> Self {
        let mut children: Vec<ParseNode> = params.clone();
        children.push(body);
        ParseNode {
            kind: NodeType::FuncDecl,
            datatype: DataType::None,
            tok: ident_tok,
            children,
        }
    }

    fn new_func_call(tok: Token, params: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::FuncCall,
            datatype: DataType::None,
            tok,
            children: params,
        }
    }

    fn new_deref_assign(ident_tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::DerefAssign,
            datatype: DataType::None,
            tok: ident_tok,
            children: vec![rhs],
        }
    }

    fn new_assign(ident_tok: Token, rhs: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::Assign,
            datatype: DataType::None,
            tok: ident_tok,
            children: vec![rhs],
        }
    }

    fn new_var(ident_tok: Token, datatype: DataType) -> Self {
        ParseNode {
            kind: NodeType::Var,
            datatype,
            tok: ident_tok,
            children: vec![],
        }
    }

    fn new_var_decl(ident_tok: Token, rhs: Option<(ParseNode, DataType)>) -> Self {
        match rhs {
            None => {
                let init_tok: Token = Token {
                    kind: TokenType::Int,
                    val: vec![b'0'],
                    pos: Pos { col: usize::MAX, row: usize::MAX },
                };
                ParseNode {
                    kind: NodeType::VarDecl,
                    datatype: DataType::None,
                    tok: ident_tok,
                    children: vec![ParseNode::new_literal(init_tok)],
                }
            },
            Some(rhs) => {
                ParseNode {
                    kind: NodeType::VarDecl,
                    datatype: rhs.1,
                    tok: ident_tok,
                    children: vec![rhs.0],
                }
            }
        }
    }

    fn new_block(body: Vec<ParseNode>) -> Self {
        ParseNode {
            kind: NodeType::Block,
            datatype: DataType::None,
            tok: Token::new_null(),
            children: body,
        }
    }

    fn new_conditional(tok: Token, cond: ParseNode, if_block: ParseNode, else_block: Option<ParseNode>) -> Self {
        match else_block {
            None => {
                ParseNode {
                    kind: NodeType::Conditional,
                    datatype: DataType::None,
                    tok,
                    children: vec![cond, if_block],
                }
            },
            Some(else_block) => {
                ParseNode {
                    kind: NodeType::Conditional,
                    datatype: DataType::None,
                    tok,
                    children: vec![cond, if_block, else_block],
                }
            }
        }
    }

    fn new_null() -> Self {
        ParseNode {
            kind: NodeType::Null,
            datatype: DataType::None,
            tok: Token::new_null(),
            children: vec![],
        }
    }

    fn new_for_loop(tok: Token, init: Option<ParseNode>, cond: Option<ParseNode>, post: Option<ParseNode>, body_block: ParseNode) -> Self {
        let children: Vec<ParseNode> = vec![
            init.unwrap_or(ParseNode::new_null()),
            cond.unwrap_or(ParseNode::new_null()),
            post.unwrap_or(ParseNode::new_null()),
            body_block,
        ];
        ParseNode {
            kind: NodeType::ForLoop,
            datatype: DataType::None,
            tok,
            children,
        }
    }

    fn new_while_loop(tok: Token, cond: ParseNode, body_block: ParseNode) -> Self {
        ParseNode {
            kind: NodeType::WhileLoop,
            datatype: DataType::None,
            tok,
            children: vec![cond, body_block],
        }
    }

    fn new_break(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::Break,
            datatype: DataType::None,
            tok,
            children: vec![],
        }
    }

    fn new_continue(tok: Token) -> Self {
        ParseNode {
            kind: NodeType::Continue,
            datatype: DataType::None,
            tok,
            children: vec![],
        }
    }
}

pub struct ParseTree {
    pub root: ParseNode
}
impl ParseTree {
    pub fn new(prog_name: String) -> Self {
        ParseTree { root: ParseNode::new_program(prog_name, Vec::new()) }
    }

    pub fn construct(&mut self, lexer: &mut Lexer) {
        let mut children: Vec<ParseNode> = Vec::new();
        while lexer.has_token() {
            children.push(self.parse_function(lexer));
        }
        self.root.children = children;
    }

    pub fn dump(&self) {
        self.root.dump(0);
    }

    pub fn post_order(&self) -> Vec<ParseNode> {
        self.root.post_order()
    }

    /* Production Rules:
     *
     * <program>    ::= { <func_decl> }
     * <block_item> ::= <statement> | <var_decl>
     * <statement>  ::= "dump" <add_expr> ";" 
     *                | "exit" <add_expr> ";" 
     *                | "return" <add_expr> ";" 
     *                | "break" ";"
     *                | "continue" ";"
     *                | <func_call> ";"
     *                | <id> "=" <add_expr> ";"
     *                | "@" <id> "=" <add_expr> ";"
     *                | "if" <or_expr> "{" { <block_item> } "}" [ "else" "{" { <block_item> } "}"
     *                | "for" [ <var_decl> | <or_expr> ] ";" [ <or_expr> ] ";" [ <or_expr> ] "{" { <block_item> } "}"
     *                | "while" <or_expr> "{" { <block_item> } "}"
     * <type>       ::= "i64" | "i64^"
     * <func_decl>  ::= <type> <id> "(" [ <type> <id> { "," <type> <id> } ] ")" "{" { <block_item } "}"
     * <func_call>  ::= <id> "(" [ <or_expr> { "," <or_expr> } ] ")" | "mmap" "(" <add_expr> ")"
     * <var_decl>   ::= <type> <id> [ "=" <add_expr> ] ";"
     * <or_expr>    ::= <and_expr> { "||" <and_expr> }
     * <and_expr>   ::= <equ_expr> { "&&" <equ_expr> }
     * <equ_expr>   ::= <rel_expr> { ("==" | "~=") <rel_expr> }
     * <rel_expr>   ::= <add_expr> { ("<" | ">" | "<=" | ">=") <add_expr> }
     * <add_expr>   ::= <term> { ("+" | "-") <term> }
     * <term>       ::= <factor> { ("*" | "/") <factor> }
     * <factor>     ::= <func_call> | "(" <or_expr> ")" | <unary_op> <factor> | <int> | <id>
     * <unary_op>   ::= "-" | "@"
     */

    fn parse_factor(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut tok: Token = lexer.consume_token();
        match tok.kind {
            TokenType::Int => ParseNode::new_literal(tok),
            TokenType::Identifier => {
                if lexer.peek_token().kind == TokenType::OpenParen {
                    lexer.consume_token();

                    let mut params: Vec<ParseNode> = Vec::new();
                    let mut next_tok: Token = lexer.peek_token();
                    while next_tok.kind != TokenType::CloseParen {
                        if next_tok.kind == TokenType::Separator {
                            if params.is_empty() {
                                panic!("{} Error: Expected `,` but got `{}`", next_tok.pos, next_tok.val_str());
                            } else {
                                lexer.consume_token();
                            }
                        }

                        params.push(self.parse_or_expr(lexer));
                        next_tok = lexer.peek_token();
                    }
                    lexer.consume_token();
                    params.reverse();
                    ParseNode::new_func_call(tok, params)
                } else {
                    ParseNode::new_var(tok, DataType::Unknown)
                }
            },
            TokenType::KeywordMMap => {
                let mut next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::OpenParen {
                    panic!("{} Error: Expected `(` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                let expression: ParseNode = self.parse_add_expr(lexer);

                next_tok = lexer.consume_token();
                if next_tok.kind != TokenType::CloseParen {
                    panic!("{} Error: Expected `)` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                ParseNode::new_mmap(tok, expression)
            },
            TokenType::OpMinus | TokenType::OpDereference => {
                let factor: ParseNode = self.parse_factor(lexer);
                ParseNode::new_un_op(tok, factor)
            },
            TokenType::OpenParen => {
                let expression: ParseNode = self.parse_or_expr(lexer);
                tok = lexer.consume_token();
                if tok.kind != TokenType::CloseParen {
                    panic!("{} Error: Expected `)` but got `{}`", tok.pos, tok.val_str());
                }
                expression
            },
            _ => panic!("{} Error: Invalid factor `{}`", tok.pos, tok.val_str())
        }
    }

    fn parse_term(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut factor: ParseNode = self.parse_factor(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpMul | TokenType::OpDiv) {
            lexer.consume_token();
            let next_factor: ParseNode = self.parse_factor(lexer);
            factor = ParseNode::new_bin_op(tok, factor, next_factor);
            tok = lexer.peek_token();
        }

        factor
    }

    fn parse_add_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut term: ParseNode = self.parse_term(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpPlus | TokenType::OpMinus) {
            lexer.consume_token();
            let next_term: ParseNode = self.parse_term(lexer);
            term = ParseNode::new_bin_op(tok, term, next_term);
            tok = lexer.peek_token();
        }

        term
    }

    fn parse_or_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut and: ParseNode = self.parse_and_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpLogicalOr) {
            lexer.consume_token();
            let next_and: ParseNode = self.parse_and_expr(lexer);
            and = ParseNode::new_bin_op(tok, and, next_and);
            tok = lexer.peek_token();
        }

        and
    }

    fn parse_and_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut equ: ParseNode = self.parse_equ_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpLogicalAnd) {
            lexer.consume_token();
            let next_equ: ParseNode = self.parse_equ_expr(lexer);
            equ = ParseNode::new_bin_op(tok, equ, next_equ);
            tok = lexer.peek_token();
        }

        equ
    }

    fn parse_equ_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut rel: ParseNode = self.parse_rel_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpEqual | TokenType::OpNotEqual) {
            lexer.consume_token();
            let next_rel: ParseNode = self.parse_rel_expr(lexer);
            rel = ParseNode::new_bin_op(tok, rel, next_rel);
            tok = lexer.peek_token();
        }

        rel
    }

    fn parse_rel_expr(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut add: ParseNode = self.parse_add_expr(lexer);
        let mut tok: Token = lexer.peek_token();
        while matches!(tok.kind, TokenType::OpGreaterThan | TokenType::OpGreaterEqual | TokenType::OpLessThan | TokenType::OpLessEqual) {
            lexer.consume_token();
            let next_add: ParseNode = self.parse_add_expr(lexer);
            add = ParseNode::new_bin_op(tok, add, next_add);
            tok = lexer.peek_token();
        }

        add
    }

    fn parse_function(&mut self, lexer: &mut Lexer) -> ParseNode {
        let mut tok: Token = lexer.consume_token();
        if tok.kind != TokenType::TypeInt64 {
            panic!("{} Error: Expected function type but got `{}`", tok.pos, tok.val_str());
        }
        tok = lexer.consume_token();
        if tok.kind != TokenType::Identifier {
            panic!("{} Error: Expected identifier but got `{}`", tok.pos, tok.val_str());
        }
        let mut next_tok: Token = lexer.consume_token();
        if next_tok.kind != TokenType::OpenParen {
            panic!("{} Error: Expected `(` but got `{}`", next_tok.pos, next_tok.val_str());
        }

        let mut args: Vec<ParseNode> = Vec::new();
        next_tok = lexer.peek_token();
        while next_tok.kind != TokenType::CloseParen {
            next_tok = lexer.consume_token();
            if !args.is_empty() {
                if next_tok.kind != TokenType::Separator {
                    panic!("{} Error: Expected `,` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                next_tok = lexer.consume_token();
            }

            let type_tok: Token = next_tok.clone();
            if !matches!(next_tok.kind, TokenType::TypeInt64Ptr | TokenType::TypeInt64) {
                panic!("{} Error: Expected type but got `{}`", next_tok.pos, next_tok.val_str());
            }
            next_tok = lexer.consume_token();
            if next_tok.kind != TokenType::Identifier {
                panic!("{} Error: Expected identifier but got `{}`", next_tok.pos, next_tok.val_str());
            }
            args.push(ParseNode::new_var(next_tok, DataType::from_tok_type(&type_tok.kind)));
            next_tok = lexer.peek_token();
        }
        lexer.consume_token();

        let mut next_tok: Token = lexer.consume_token();
        if next_tok.kind != TokenType::OpenScope {
            panic!("{} Error: Expected `{{` but got `{}`", next_tok.pos, next_tok.val_str());
        }
        let mut body: Vec<ParseNode> = Vec::new();
        while next_tok.kind != TokenType::CloseScope {
            body.push(self.parse_block_item(lexer));
            next_tok = lexer.peek_token();
        }
        lexer.consume_token();

        ParseNode::new_func_decl(tok, args, ParseNode::new_block(body))
    }

    fn parse_block_item(&mut self, lexer: &mut Lexer) -> ParseNode {
        let tok: Token = lexer.peek_token();
        match tok.kind {
            TokenType::TypeInt64 | TokenType::TypeInt64Ptr => {
                self.parse_decl(lexer)
            },
            TokenType::KeywordFor | TokenType::KeywordIf | TokenType::KeywordExit | TokenType::KeywordReturn |
            TokenType::KeywordDebugDump | TokenType::Identifier | TokenType::KeywordBreak | 
            TokenType::KeywordContinue | TokenType::KeywordWhile | TokenType::OpDereference => {
                self.parse_statement(lexer)
            },
            _ => panic!("{} Error: Expected block item but got `{}`", tok.pos, tok.val_str())
        }
    }

    fn parse_decl(&mut self, lexer: &mut Lexer) -> ParseNode {
        let tok: Token = lexer.consume_token();
        if !matches!(tok.kind, TokenType::TypeInt64 | TokenType::TypeInt64Ptr) {
            panic!("{} Error: Expected type but got `{}`", tok.pos, tok.val_str());
        }
        let ident_tok: Token = lexer.consume_token();
        if ident_tok.kind != TokenType::Identifier {
            panic!("{} Error: Expected identifier but got `{}`", ident_tok.pos, ident_tok.val_str());
        }
        let mut next_tok: Token = lexer.consume_token();
        if next_tok.kind == TokenType::End {
            return ParseNode::new_var_decl(ident_tok, None);
        }

        if next_tok.kind != TokenType::OpAssign {
            panic!("{} Error: Expected `=` or `;` but got `{}`", next_tok.pos, next_tok.val_str());
        } 
        let expression: ParseNode = self.parse_add_expr(lexer);
        next_tok = lexer.consume_token();
        if next_tok.kind != TokenType::End {
            panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
        }
        ParseNode::new_var_decl(ident_tok, Some((expression, DataType::from_tok_type(&tok.kind))))
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> ParseNode {
        let tok: Token = lexer.consume_token();
        match tok.kind {
            TokenType::KeywordBreak => {
                let next_tok = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_break(tok)
            },
            TokenType::KeywordContinue => {
                let next_tok = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_continue(tok)
            },
            TokenType::KeywordWhile => {
                let expression: ParseNode = self.parse_or_expr(lexer);
                let mut next_tok = lexer.consume_token();
                if next_tok.kind != TokenType::OpenScope {
                    panic!("{} Error: Expected `{{` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                let mut body: Vec<ParseNode> = Vec::new();
                next_tok = lexer.peek_token();
                while next_tok.kind != TokenType::CloseScope {
                    body.push(self.parse_block_item(lexer));
                    next_tok = lexer.peek_token();
                }
                lexer.consume_token();

                ParseNode::new_while_loop(tok, expression, ParseNode::new_block(body))
            },
            TokenType::KeywordFor => {
                let init: Option<ParseNode>;
                let mut next_tok: Token = lexer.peek_token();
                match next_tok.kind {
                    TokenType::End => {
                        lexer.consume_token();
                        init = None;
                    },
                    TokenType::TypeInt64 => init = Some(self.parse_decl(lexer)),
                    TokenType::OpDereference => {
                        lexer.consume_token();

                        let ident_tok: Token = lexer.consume_token();
                        if ident_tok.kind != TokenType::Identifier {
                            panic!("{} Error: Expected identifier but got `{}`", ident_tok.pos, ident_tok.val_str());
                        }   

                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::OpAssign {
                            panic!("{} Error: Expected `=` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        let expression: ParseNode = self.parse_add_expr(lexer);
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::End {
                            panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        init = Some(ParseNode::new_deref_assign(ident_tok, expression));
                    },
                    TokenType::Identifier => {
                        let ident_tok: Token = lexer.consume_token();
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::OpAssign {
                            panic!("{} Error: Expected `=` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        let expression: ParseNode = self.parse_add_expr(lexer);
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::End {
                            panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        init = Some(ParseNode::new_assign(ident_tok, expression));
                    },
                    _ => {
                        panic!("{} Error: Unexpected initializer in for loop `{}`", next_tok.pos, next_tok.val_str());
                    },
                }

                let cond: Option<ParseNode>;
                next_tok = lexer.peek_token();
                if next_tok.kind == TokenType::End {
                    lexer.consume_token();
                    cond = None;
                } else {
                    cond = Some(self.parse_or_expr(lexer));
                    next_tok = lexer.consume_token();
                    if next_tok.kind != TokenType::End {
                        panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                    }
                }

                let mut post: Option<ParseNode> = None;
                next_tok = lexer.consume_token();
                match next_tok.kind {
                    TokenType::OpenScope => {},
                    TokenType::OpDereference => {
                        lexer.consume_token();

                        let ident_tok: Token = lexer.consume_token();
                        if ident_tok.kind != TokenType::Identifier {
                            panic!("{} Error: Expected identifier but got `{}`", ident_tok.pos, ident_tok.val_str());
                        }   

                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::OpAssign {
                            panic!("{} Error: Expected `=` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        let expression: ParseNode = self.parse_add_expr(lexer);
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::OpenScope {
                            panic!("{} Error: Expected `{{` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        post = Some(ParseNode::new_deref_assign(ident_tok, expression));
                    },
                    TokenType::Identifier => {
                        let ident_tok: Token = next_tok.clone();
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::OpAssign {
                            panic!("{} Error: Expected `=` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        let expression: ParseNode = self.parse_add_expr(lexer);
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::OpenScope {
                            panic!("{} Error: Expected `{{` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        post = Some(ParseNode::new_assign(ident_tok, expression));
                    },
                    _ => {
                        panic!("{} Error: Unexpected initializer in for loop `{}`", next_tok.pos, next_tok.val_str());
                    },
                }

                let mut body: Vec<ParseNode> = Vec::new();
                next_tok = lexer.peek_token();
                while next_tok.kind != TokenType::CloseScope {
                    body.push(self.parse_block_item(lexer));
                    next_tok = lexer.peek_token();
                }
                lexer.consume_token();

                ParseNode::new_for_loop(tok, init, cond, post, ParseNode::new_block(body))
            },
            TokenType::KeywordIf => {
                let guard: ParseNode = self.parse_or_expr(lexer);

                let mut next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::OpenScope {
                    panic!("{} Error: Expected `{{` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                let mut if_body: Vec<ParseNode> = Vec::new();
                next_tok = lexer.peek_token();
                while next_tok.kind != TokenType::CloseScope {
                    if_body.push(self.parse_block_item(lexer));
                    next_tok = lexer.peek_token();
                }
                lexer.consume_token();

                next_tok = lexer.peek_token();
                if next_tok.kind != TokenType::KeywordElse {
                    return ParseNode::new_conditional(tok, guard, ParseNode::new_block(if_body), None);
                }
                lexer.consume_token();

                next_tok = lexer.consume_token();
                if next_tok.kind != TokenType::OpenScope {
                    panic!("{} Error: Expected `{{` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                let mut else_body: Vec<ParseNode> = Vec::new();
                next_tok = lexer.peek_token();
                while next_tok.kind != TokenType::CloseScope {
                    else_body.push(self.parse_block_item(lexer));
                    next_tok = lexer.peek_token();
                }
                lexer.consume_token();

                ParseNode::new_conditional(tok, guard, ParseNode::new_block(if_body), Some(ParseNode::new_block(else_body)))
            },
            TokenType::KeywordReturn => {
                let expression: ParseNode = self.parse_add_expr(lexer);
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_return(tok, expression)
            },
            TokenType::KeywordMMap => {
                let expression: ParseNode = self.parse_add_expr(lexer);
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_mmap(tok, expression)
            },
            TokenType::KeywordExit => {
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::OpenParen {
                    panic!("{} Error: Expected `(` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                let expression: ParseNode = self.parse_add_expr(lexer);

                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::CloseParen {
                    panic!("{} Error: Expected `)` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_exit(tok, expression)
            },
            TokenType::KeywordDebugDump => {
                let expression: ParseNode = self.parse_add_expr(lexer);
                let next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }
                ParseNode::new_debug_dump(tok, expression)
            },
            TokenType::OpDereference => {
                let ident_tok: Token = lexer.consume_token();
                if ident_tok.kind != TokenType::Identifier {
                    panic!("{} Error: Expected identifier but got `{}`", ident_tok.pos, ident_tok.val_str());
                }   

                let mut next_tok: Token = lexer.consume_token();
                if next_tok.kind != TokenType::OpAssign {
                    panic!("{} Error: Expected `=` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                let expression: ParseNode = self.parse_add_expr(lexer);
                next_tok = lexer.consume_token();
                if next_tok.kind != TokenType::End {
                    panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                }

                ParseNode::new_deref_assign(ident_tok, expression)
            },
            TokenType::Identifier => {
                let mut next_tok: Token = lexer.consume_token();
                match next_tok.kind {
                    // Function Call
                    TokenType::OpenParen => {
                        let mut params: Vec<ParseNode> = Vec::new();
                        next_tok = lexer.peek_token();
                        while next_tok.kind != TokenType::CloseParen {
                            if next_tok.kind == TokenType::Separator {
                                if params.is_empty() {
                                    panic!("{} Error: Expected `,` but got `{}`", next_tok.pos, next_tok.val_str());
                                } else {
                                    lexer.consume_token();
                                }
                            }

                            params.push(self.parse_or_expr(lexer));
                            next_tok = lexer.peek_token();
                        }
                        lexer.consume_token();
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::End {
                            panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                        }

                        params.reverse();
                        ParseNode::new_func_call(tok, params)
                    },
                    // Variable Assignment
                    TokenType::OpAssign => {
                        let expression: ParseNode = self.parse_add_expr(lexer);
                        next_tok = lexer.consume_token();
                        if next_tok.kind != TokenType::End {
                            panic!("{} Error: Expected `;` but got `{}`", next_tok.pos, next_tok.val_str());
                        }
                        ParseNode::new_assign(tok, expression)
                    },
                    _ => panic!("{} Error: Expected `(` or `=` but got `{}`", next_tok.pos, next_tok.val_str()),
                }
            },
            _ => panic!("{} Error: Expected statement but got `{}`", tok.pos, tok.val_str()),
        }
    }
}
