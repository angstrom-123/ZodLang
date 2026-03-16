use crate::lex::Lexer;
use crate::lex::Pos;
use crate::lex::Tok;
use crate::lex::TokKind;
use crate::types::Datatype;

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
// <Additive>       : <Term>       [ ( "+" | "-" | "%" ) <Term> ]
// <Term>           : <Factor>     [ ( "*" | "/" ) <Factor> ]
// <Factor>         : <Literal-Int> 
//                  | <Literal-String> 
//                  | <Literal-Char>
//                  | <Func-Call> 
//                  | <Syscall> 
//                  | "(" <Union> ")" 
//                  | <Unary> <Factor> 
//                  | <Ident>

static OP_PRECEDENCE: &[&[TokKind]; Precedence::_Count as usize] = &[
    &[TokKind::LogOr],
    &[TokKind::LogAnd],
    &[TokKind::Equal, TokKind::NotEqual],
    &[TokKind::GT, TokKind::GE, TokKind::LT, TokKind::LE],
    &[TokKind::Plus, TokKind::Minus, TokKind::Mod],
    &[TokKind::Mul, TokKind::Div],
    &[]
];

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum NodeKind {
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

#[derive(Clone, PartialEq)]
pub struct Node {
    pub kind:     NodeKind,
    pub datatype: Datatype,
    pub tok:      Tok,
    pub children: Vec<Node>,
}
impl Node {
    pub fn is_null(&self) -> bool {
        self.kind == NodeKind::Null
    }

    pub fn dump(&self, _depth: usize) {
        eprintln!("{}: {:padding$}\x1b[94m* {:?}\x1b[0m (\x1b[92m{:?}::{:?}\x1b[0m: `{}`)", self.tok.pos, "", self.kind, self.tok.kind, self.datatype, self.tok.val_str(), padding = _depth);

        for child in &self.children {
            child.dump(_depth + 4);
        }
    }

    pub fn in_order(&self) -> Vec<Node> {
        let mut res: Vec<Node> = Vec::new();
        res.push(self.clone());
        for node in &self.children {
            res.append(&mut node.in_order());
        }

        res
    }

    pub fn exclusive_post_order(&self) -> Vec<Node> {
        let mut res: Vec<Node> = self.post_order();
        res.pop();
        res
    }

    pub fn post_order(&self) -> Vec<Node> {
        let mut res: Vec<Node> = Vec::new();
        for node in &self.children {
            res.append(&mut node.post_order());
        }
        res.push(self.clone());
        res
    }

    fn zero_literal() -> Self {
        let mut tok: Tok = Tok::null();
        tok.val = vec![b'0'];
        Node {
            kind: NodeKind::LiteralInt,
            datatype: Datatype::I64,
            tok,
            children: vec![],
        }
    }

    fn null() -> Self {
        Node {
            kind: NodeKind::Null,
            datatype: Datatype::None,
            tok: Tok::null(),
            children: vec![],
        }
    }

    fn group(nodes: Vec<Node>) -> Self {
        let tok: Tok = if let Some(node) = nodes.first() {
            Tok::null_at(node.tok.pos.clone())
        } else {
            Tok::null()
        };
        Node {
            kind: NodeKind::Group,
            datatype: Datatype::None,
            tok,
            children: nodes
        }
    }

    fn literal_int(tok: Tok) -> Self {
        Node {
            kind: NodeKind::LiteralInt,
            datatype: Datatype::I64,
            tok,
            children: vec![]
        }
    }

    fn literal_string(tok: Tok) -> Self {
        Node {
            kind: NodeKind::LiteralString,
            datatype: Datatype::ChrPtr,
            tok,
            children: vec![]
        }
    }

    fn literal_char(tok: Tok) -> Self {
        Node {
            kind: NodeKind::LiteralChar,
            datatype: Datatype::Chr,
            tok,
            children: vec![]
        }
    }

    fn include(tok: Tok, rhs: Node) -> Self {
        Node {
            kind: NodeKind::Include,
            datatype: Datatype::None,
            tok,
            children: vec![rhs],
        }
    }

    fn continuee(tok: Tok) -> Self {
        Node {
            kind: NodeKind::Continue,
            datatype: Datatype::None,
            tok,
            children: vec![]
        }
    }
    
    fn breakk(tok: Tok) -> Self {
        Node {
            kind: NodeKind::Break,
            datatype: Datatype::None,
            tok,
            children: vec![]
        }
    }

    fn returnn(tok: Tok, rhs: Node) -> Self {
        Node {
            kind: NodeKind::Return,
            datatype: Datatype::Unknown,
            tok,
            children: vec![rhs]
        }
    }

    fn var_decl(datatype: Datatype, ident: Tok) -> Self {
        Node {
            kind: NodeKind::VarDecl,
            datatype,
            tok: ident,
            children: vec![],
        }
    }

    fn var(ident: Tok) -> Self {
        Node {
            kind: NodeKind::Var,
            datatype: Datatype::Unknown,
            tok: ident,
            children: vec![],
        }
    }

    fn func_decl(datatype: Datatype, ident: Tok, args: Vec<Node>, body: Vec<Node>) -> Self {
        Node {
            kind: NodeKind::FuncDecl,
            datatype,
            tok: ident,
            children: vec![Self::group(args), Self::group(body)],
        }
    }

    fn syscall(ident: Tok, args: Vec<Node>) -> Self {
        Node {
            kind: NodeKind::Syscall,
            datatype: Datatype::I64,
            tok: ident,
            children: args,
        }
    }

    fn func_call(ident: Tok, args: Vec<Node>) -> Self {
        Node {
            kind: NodeKind::FuncCall,
            datatype: Datatype::Unknown,
            tok: ident,
            children: args,
        }
    }

    fn bin_op(operator: Tok, lhs: Node, rhs: Node) -> Self {
        Node {
            kind: NodeKind::BinaryOp,
            datatype: Datatype::None,
            tok: operator,
            children: vec![lhs, rhs],
        }
    }

    fn un_op(operator: Tok, rhs: Node) -> Self {
        Node {
            kind: NodeKind::UnaryOp,
            datatype: Datatype::None,
            tok: operator,
            children: vec![rhs],
        }
    }

    fn assign(operator: Tok, lhs: Node, rhs: Node) -> Self {
        Node {
            kind: NodeKind::Assign,
            datatype: Datatype::None,
            tok: operator,
            children: vec![lhs, rhs],
        }
    }

    fn conditional(ident: Tok, cond: Node, if_body: Vec<Node>, else_body: Vec<Node>) -> Self {
        let mut res: Node = Node {
            kind: NodeKind::Conditional,
            datatype: Datatype::None,
            tok: ident,
            children: vec![cond, Self::group(if_body)],
        };

        if !else_body.is_empty() {
            res.children.push(Self::group(else_body));
        }

        res
    }

    fn forr(ident: Tok, decl: Option<Node>, init: Option<Node>, cond: Option<Node>, post: Option<Node>, body: Vec<Node>) -> Self {
        Node {
            kind: NodeKind::ForLoop,
            datatype: Datatype::None,
            tok: ident,
            children: vec![
                decl.unwrap_or(Self::null()),
                init.unwrap_or(Self::null()),
                cond.unwrap_or(Self::null()),
                post.unwrap_or(Self::null()),
                Self::group(body)
            ],
        }
    }

    fn whilee(ident: Tok, cond: Node, body: Vec<Node>) -> Self {
        Node {
            kind: NodeKind::WhileLoop,
            datatype: Datatype::None,
            tok: ident,
            children: vec![cond, Self::group(body)],
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

pub struct AST {
    pub root: Node,
}
impl Default for AST {
    fn default() -> Self {
        Self::new()
    }
}
impl AST {
    pub fn new() -> Self {
        AST {
            root: Node::group(Vec::new())
        }
    }

    pub fn find_node(&self, ident: &Tok) -> Node {
        for node in &self.root.post_order() {
            if node.tok == *ident {
                return node.clone();
            }
        }

        panic!("{} Error: Couldn't find node with token {}", ident.pos, ident.val_str());
    }

    pub fn construct(&mut self, lexer: &mut Lexer) {
        let mut children: Vec<Node> = Vec::new();
        while lexer.has_token() {
            let tok: Tok = lexer.peek_token();
            match tok.kind {
                // <Func-Decl> : <Type> <Var> "(" [ <Type> <Var> { "," <Type> <Var> } ] ")" "{" { <Statement> } "}"
                TokKind::TypeVoid
                | TokKind::TypeInt64 
                | TokKind::TypeInt64Ptr 
                | TokKind::TypeChr 
                | TokKind::TypeChrPtr 
                | TokKind::TypeAnyPtr => {
                    children.push(self.parse_func_decl(lexer));
                },
                // <Include> : "include" <Literal-String>
                TokKind::Include => {
                    let tok: Tok = lexer.consume_token();
                    let file: Tok = lexer.expect_token(TokKind::String);
                    children.push(Node::include(tok, Node::literal_string(file)));
                },
                _ => panic!("{} Error: Unexpected top level statement `{}`", tok.pos, tok.val_str())
            }
        }
        self.root.children = children;
    }

    pub fn dump(&self) {
        self.root.dump(0);
    }

    pub fn post_order(&self) -> Vec<Node> {
        self.root.post_order()
    }

    // <Ident> : [ "@" ] <Name> { "[" <Additive "]" }
    fn parse_ident(&mut self, lexer: &mut Lexer) -> Node {
        let mut ident: Node;
        if lexer.peek_token().kind == TokKind::Deref {
            let deref: Tok = lexer.consume_token();
            let name: Tok = lexer.expect_token(TokKind::Ident);
            ident = Node::un_op(deref, Node::var(name));
        } else {
            let name: Tok = lexer.expect_token(TokKind::Ident);
            ident = Node::var(name);
        }

        if lexer.peek_token().kind == TokKind::OSquare {
            // Subscript
            let tok = lexer.consume_token();
            let subscript: Tok = Tok {
                kind: TokKind::Subscript,
                val: Vec::from("[_]"),
                pos: tok.pos,
            };
            let expr: Node = self.parse_expression(lexer, Precedence::Additive as usize);
            lexer.expect_token(TokKind::CSquare);
            ident = Node::bin_op(subscript, ident, expr);
        }

        ident
    }

    // <Assign> : [ "@" ] <Ident> "=" <Union> 
    //          | "@" "(" <Additive> ")" "=" <Union>
    fn parse_assign(&mut self, lexer: &mut Lexer) -> Node {
        if lexer.peek_token().kind == TokKind::Deref {
            let deref: Tok = lexer.consume_token();
            if lexer.peek_token().kind == TokKind::OParen {
                // Assign to address expression
                lexer.consume_token();
                let lhs: Node = self.parse_expression(lexer, Precedence::Additive as usize);
                lexer.expect_token(TokKind::CParen);
                let assign: Tok = lexer.consume_token();
                let rhs: Node = self.parse_expression(lexer, Precedence::Union as usize);
                Node::assign(assign, Node::un_op(deref, lhs), rhs)
            } else {
                // Assign to variable address
                let lhs: Node = self.parse_ident(lexer);
                let assign: Tok = lexer.expect_token(TokKind::Assign);
                let rhs: Node = self.parse_expression(lexer, Precedence::Union as usize);
                Node::assign(assign, Node::un_op(deref, lhs), rhs)
            }
        } else {
            // Regular assign
            let lhs: Node = self.parse_ident(lexer);
            let assign: Tok = lexer.expect_token(TokKind::Assign);
            let rhs: Node = self.parse_expression(lexer, Precedence::Union as usize);
            Node::assign(assign, lhs, rhs)
        }
    }

    // <Func-Decl> : <Type> <Var> "(" [ <Type> <Var> { "," <Type> <Var> } ] ")" "{" { <Statement> } "}"
    fn parse_func_decl(&mut self, lexer: &mut Lexer) -> Node {
        let datatype: Datatype = Datatype::from_token(&lexer.expect_type());
        let ident: Tok = lexer.expect_token(TokKind::Ident);

        // Function arguments between parens
        lexer.expect_token(TokKind::OParen);

        let mut args: Vec<Node> = Vec::new();
        while lexer.peek_token().kind != TokKind::CParen {
            if !args.is_empty() {
                lexer.expect_token(TokKind::Separator);
            }
            let arg_type: Datatype = Datatype::from_token(&lexer.expect_type());
            let arg_ident: Tok = lexer.expect_token(TokKind::Ident);
            args.push(Node::var_decl(arg_type, arg_ident));
        }

        lexer.expect_token(TokKind::CParen);

        // Function body between scopes
        lexer.expect_token(TokKind::OScope);

        let mut body: Vec<Node> = Vec::new();
        while lexer.peek_token().kind != TokKind::CScope {
            body.extend(self.parse_statement(lexer));
        }

        lexer.expect_token(TokKind::CScope);

        Node::func_decl(datatype, ident, args, body)
    }

    // <Syscall> : <Ident> "(" [ <Union> { "," <Union> } ] ")"
    fn parse_syscall(&mut self, lexer: &mut Lexer) -> Node {
        let ident: Tok = lexer.expect_token(TokKind::Syscall);

        // Function arguments between parens
        lexer.expect_token(TokKind::OParen);

        let mut args: Vec<Node> = Vec::new();
        while lexer.peek_token().kind != TokKind::CParen {
            if !args.is_empty() {
                lexer.expect_token(TokKind::Separator);
            }
            args.push(self.parse_expression(lexer, Precedence::Union as usize));
        }

        lexer.expect_token(TokKind::CParen);

        args.reverse();
        Node::syscall(ident, args)
    }

    // <Func-Call> : <Ident> "(" [ <Union> { "," <Union> } ] ")"
    fn parse_func_call(&mut self, lexer: &mut Lexer) -> Node {
        let ident: Tok = lexer.expect_token(TokKind::Ident);

        // Function arguments between parens
        lexer.expect_token(TokKind::OParen);

        let mut args: Vec<Node> = Vec::new();
        while lexer.peek_token().kind != TokKind::CParen {
            if !args.is_empty() {
                lexer.expect_token(TokKind::Separator);
            }
            args.push(self.parse_expression(lexer, Precedence::Union as usize));
        }

        lexer.expect_token(TokKind::CParen);

        args.reverse();
        Node::func_call(ident, args)
    }

    fn parse_var_decl(&mut self, lexer: &mut Lexer) -> (Node, Node) {
        let datatype: Datatype = Datatype::from_token(&lexer.expect_type());
        let ident: Tok = lexer.peek_token();
        let lhs: Node = Node::var_decl(datatype, ident.clone());
        if ident.kind != TokKind::Ident {
            panic!("{} Error: Expected `{}` but got `{}`",  ident.pos, TokKind::Ident.val_str(), ident.val_str());
        }

        // Either explicit assignment, or implicit zero assignment
        if lexer.peek_next_token().kind == TokKind::Assign {
            (lhs, self.parse_assign(lexer))
        } else {
            lexer.consume_token();
            let operator: Tok = Tok {
                kind: TokKind::Assign,
                val: Vec::from("="),
                pos: Pos { row: u32::MAX, col: u32::MAX, file: String::from("Implicit") },
            };
            (lhs, Node::assign(operator, Node::var(ident), Node::zero_literal()))
        }
    }

    // <Union>          : <Intersection> [ "||" <Intersection> ]
    // <Intersection>   : <Equality>     [ "&&" <Equality> ]
    // <Equality>       : <Relational>   [ ( "==" | "!=" ) <Relational> ]
    // <Relational>     : <Additive>     [ ( "<" | ">" | "<=" | ">=" ) <Additive> ]
    // <Additive>       : <Term>         [ ( "+" | "-" ) <Term> ]
    // <Term>           : <Factor>       [ ( "*" | "/" ) <Factor> ]
    fn parse_expression(&mut self, lexer: &mut Lexer, level: usize) -> Node {
        if level == Precedence::Factor as usize {
            return self.parse_factor(lexer);
        }
        let mut lhs = self.parse_expression(lexer, level + 1);

        while OP_PRECEDENCE[level].contains(&lexer.peek_token().kind) {
            let operator: Tok = lexer.consume_token();
            lhs = Node::bin_op(operator, lhs, self.parse_expression(lexer, level + 1));
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
    fn parse_factor(&mut self, lexer: &mut Lexer) -> Node {
        match lexer.peek_token().kind {
            TokKind::Int => Node::literal_int(lexer.consume_token()),
            TokKind::String => Node::literal_string(lexer.consume_token()),
            TokKind::Char => Node::literal_char(lexer.consume_token()),
            TokKind::Syscall => self.parse_syscall(lexer),
            TokKind::Ident => {
                if lexer.peek_next_token().kind == TokKind::OParen {
                    self.parse_func_call(lexer)
                } else {
                    self.parse_ident(lexer)
                }
            },
            TokKind::OParen => {
                lexer.consume_token();
                let expr: Node = self.parse_expression(lexer, Precedence::Union as usize);
                lexer.expect_token(TokKind::CParen);
                expr
            },
            TokKind::Minus | TokKind::Deref => {
                Node::un_op(lexer.consume_token(), self.parse_factor(lexer))
            },
            _ => panic!("{} Error: Expected factor but got `{}`", lexer.peek_token().pos, lexer.peek_token().val_str())
        }
    }

    fn parse_statement(&mut self, lexer: &mut Lexer) -> Vec<Node> {
        match lexer.peek_token().kind {
            // "return" <Additive> ";"
            TokKind::Return => {
                let expr: Node = Node::returnn(lexer.consume_token(), self.parse_expression(lexer, Precedence::Union as usize));
                lexer.expect_token(TokKind::End);
                vec![expr]
            },
            // "break" ";"
            TokKind::Break => {
                let expr: Node = Node::breakk(lexer.consume_token());
                lexer.expect_token(TokKind::End);
                vec![expr]
            },
            // "continue" ";"
            TokKind::Continue => {
                let expr: Node = Node::continuee(lexer.consume_token());
                lexer.expect_token(TokKind::End);
                vec![expr]
            },
            // "for" [ ( <Var-Decl> | <Assign> ) ]  ";" [ <Union> ] ";" ( <Assign> | Func-Call ) "{" { <Statement> } "}"
            TokKind::For => {
                let ident: Tok = lexer.consume_token();

                let mut decl: Option<Node> = None;
                let mut init: Option<Node> = None;
                if matches!(lexer.peek_token().kind, TokKind::TypeVoid
                                                     | TokKind::TypeInt64Ptr
                                                     | TokKind::TypeInt64
                                                     | TokKind::TypeChrPtr
                                                     | TokKind::TypeChr
                                                     | TokKind::TypeAnyPtr) {
                    let (d, i) = self.parse_var_decl(lexer);
                    decl = Some(d);
                    init = Some(i);
                } else if lexer.peek_token().kind != TokKind::End {
                    init = Some(self.parse_assign(lexer));
                }

                lexer.expect_token(TokKind::End);

                let mut cond: Option<Node> = None;
                if lexer.peek_token().kind != TokKind::End {
                    cond = Some(self.parse_expression(lexer, Precedence::Union as usize));
                }

                lexer.expect_token(TokKind::End);

                let mut post: Option<Node> = None;
                if lexer.peek_token().kind != TokKind::OScope {
                    if lexer.peek_next_token().kind == TokKind::OParen {
                        post = Some(self.parse_func_call(lexer));
                    } else {
                        post = Some(self.parse_assign(lexer));
                    }
                }

                // For body between scopes
                lexer.expect_token(TokKind::OScope);

                let mut body: Vec<Node> = Vec::new();
                while lexer.peek_token().kind != TokKind::CScope {
                    body.extend(self.parse_statement(lexer));
                }

                lexer.expect_token(TokKind::CScope);

                vec![Node::forr(ident, decl, init, cond, post, body)]
            },
            // "while" <Union> "{" { <Statement> } "}"
            TokKind::While => {
                let ident: Tok = lexer.consume_token();
                let cond: Node = self.parse_expression(lexer, Precedence::Union as usize);

                // While body between scopes
                lexer.expect_token(TokKind::OScope);

                let mut body: Vec<Node> = Vec::new();
                while lexer.peek_token().kind != TokKind::CScope {
                    body.extend(self.parse_statement(lexer));
                }

                lexer.expect_token(TokKind::CScope);

                vec![Node::whilee(ident, cond, body)]
            },
            // "if" <Union> "{" { <Statement> } "}" [ "else" "{" { <Statement> } "}" ]
            TokKind::If => {
                let ident: Tok = lexer.consume_token();
                let cond: Node = self.parse_expression(lexer, Precedence::Union as usize);

                // If body between scopes
                lexer.expect_token(TokKind::OScope);

                let mut if_body: Vec<Node> = Vec::new();
                while lexer.peek_token().kind != TokKind::CScope {
                    if_body.extend(self.parse_statement(lexer));
                }

                lexer.expect_token(TokKind::CScope);

                let mut else_body: Vec<Node> = Vec::new();
                if lexer.peek_token().kind == TokKind::Else {
                    lexer.consume_token();
                    // Else body between scopes
                    lexer.expect_token(TokKind::OScope);

                    while lexer.peek_token().kind != TokKind::CScope {
                        else_body.extend(self.parse_statement(lexer));
                    }

                    lexer.expect_token(TokKind::CScope);
                }

                vec![Node::conditional(ident, cond, if_body, else_body)]
            },
            TokKind::Syscall => {
                let expr: Node = self.parse_syscall(lexer);
                lexer.expect_token(TokKind::End);
                vec![expr]
            },
            // <Func-Call> ";"
            // <Assign> ";"
            TokKind::Ident => {
                let expr: Node = match lexer.peek_next_token().kind {
                    TokKind::OParen => self.parse_func_call(lexer),
                    TokKind::Assign 
                    | TokKind::OSquare => self.parse_assign(lexer),
                    _ => panic!("{} Error: Expected assignment or function call but got `{}`",
                                lexer.peek_token().pos, lexer.peek_token().val_str())
                };
                lexer.expect_token(TokKind::End);
                vec![expr]
            },
            // <Var-Decl> ";"
            TokKind::TypeVoid 
            | TokKind::TypeInt64
            | TokKind::TypeInt64Ptr
            | TokKind::TypeChr
            | TokKind::TypeChrPtr
            | TokKind::TypeAnyPtr => {
                let (decl, init) = self.parse_var_decl(lexer);
                lexer.expect_token(TokKind::End);
                vec![decl, init]
            },
            // <Assign> ";"
            TokKind::Deref => {
                let expr: Node = self.parse_assign(lexer);
                lexer.expect_token(TokKind::End);
                vec![expr]
            },
            _ => panic!("{} Error: Expected statement but got `{}`", lexer.peek_token().pos, lexer.peek_token().val_str())
        }
    }
}
