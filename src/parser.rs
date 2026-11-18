//! Recursive-descent parser: tokens -> AST.

use crate::ast::*;
use crate::diagnostic::{Diagnostic, Pos};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    toks: Vec<Token>,
    i: usize,
    /// When true, `Name {` is not parsed as a struct literal. Set while
    /// parsing `if`/`while`/`for` headers so `if x { ... }` stays unambiguous.
    no_struct: bool,
}

type PResult<T> = Result<T, Diagnostic>;

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self {
        Parser {
            toks,
            i: 0,
            no_struct: false,
        }
    }

    fn peek(&self) -> &TokenKind {
        &self.toks[self.i].kind
    }

    fn pos(&self) -> Pos {
        self.toks[self.i].pos
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.i].clone();
        if t.kind != TokenKind::Eof {
            self.i += 1;
        }
        t
    }

    fn check(&self, k: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(k)
    }

    fn eat(&mut self, k: &TokenKind) -> bool {
        if self.check(k) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, k: TokenKind, context: &str) -> PResult<Token> {
        if self.check(&k) {
            Ok(self.advance())
        } else {
            Err(Diagnostic::new(
                self.pos(),
                format!(
                    "expected {} {}, found {}",
                    k.describe(),
                    context,
                    self.peek().describe()
                ),
            ))
        }
    }

    fn expect_ident(&mut self, context: &str) -> PResult<(String, Pos)> {
        let pos = self.pos();
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.advance();
                Ok((name, pos))
            }
            other => Err(Diagnostic::new(
                pos,
                format!(
                    "expected identifier {}, found {}",
                    context,
                    other.describe()
                ),
            )),
        }
    }

    // ---- items -----------------------------------------------------------

    pub fn parse_program(&mut self) -> PResult<Program> {
        let mut prog = Program::default();
        while !self.check(&TokenKind::Eof) {
            let pos = self.pos();
            match self.peek() {
                TokenKind::Fn => prog.functions.push(self.parse_function(false)?),
                TokenKind::Extern => {
                    self.advance();
                    prog.functions.push(self.parse_function(true)?);
                }
                TokenKind::Struct => prog.structs.push(self.parse_struct()?),
                TokenKind::Let | TokenKind::Const => prog.globals.push(self.parse_global()?),
                TokenKind::Import => {
                    self.advance();
                    let path = match self.peek().clone() {
                        TokenKind::Str(s) => {
                            self.advance();
                            s
                        }
                        other => {
                            return Err(Diagnostic::new(
                                self.pos(),
                                format!(
                                    "expected a string path after 'import', found {}",
                                    other.describe()
                                ),
                            ))
                        }
                    };
                    self.expect(TokenKind::Semi, "after import")?;
                    prog.imports.push(Import { path, pos });
                }
                other => {
                    return Err(Diagnostic::new(
                        pos,
                        format!(
                            "expected 'fn', 'struct', 'const', 'let', 'extern' or 'import' \
                             at top level, found {}",
                            other.describe()
                        ),
                    ));
                }
            }
        }
        Ok(prog)
    }

    fn parse_type(&mut self) -> PResult<Type> {
        if self.eat(&TokenKind::LBracket) {
            let inner = self.parse_type()?;
            self.expect(TokenKind::RBracket, "to close the array type")?;
            return Ok(Type::Array(Box::new(inner)));
        }
        let (name, _) = self.expect_ident("as a type name")?;
        // Unknown names are reported by the type checker.
        Ok(Type::from_name(&name).unwrap_or(Type::Struct(name)))
    }

    fn parse_function(&mut self, is_extern: bool) -> PResult<Function> {
        let pos = self.expect(TokenKind::Fn, "")?.pos;
        let (name, _) = self.expect_ident("after 'fn'")?;
        self.expect(TokenKind::LParen, "after function name")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let (pname, ppos) = self.expect_ident("as parameter name")?;
                self.expect(TokenKind::Colon, "after parameter name")?;
                let ty = self.parse_type()?;
                params.push(Param {
                    name: pname,
                    ty,
                    pos: ppos,
                });
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "after parameters")?;
        let ret = if self.eat(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };
        let body = if is_extern {
            self.expect(TokenKind::Semi, "after extern declaration")?;
            Vec::new()
        } else {
            self.parse_block()?
        };
        Ok(Function {
            name,
            params,
            ret,
            body,
            pos,
            is_extern,
            from_std: false,
        })
    }

    fn parse_struct(&mut self) -> PResult<StructDef> {
        let pos = self.expect(TokenKind::Struct, "")?.pos;
        let (name, _) = self.expect_ident("after 'struct'")?;
        self.expect(TokenKind::LBrace, "after struct name")?;
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            let (fname, fpos) = self.expect_ident("as field name")?;
            self.expect(TokenKind::Colon, "after field name")?;
            let ty = self.parse_type()?;
            fields.push(Field {
                name: fname,
                ty,
                pos: fpos,
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RBrace, "to close the struct")?;
        Ok(StructDef { name, fields, pos })
    }

    fn parse_global(&mut self) -> PResult<Global> {
        let tok = self.advance();
        let is_const = tok.kind == TokenKind::Const;
        let (name, _) = self.expect_ident("after 'let'/'const'")?;
        self.expect(TokenKind::Colon, "(a type annotation is required)")?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Assign, "(globals must be initialized)")?;
        let init = self.parse_expr()?;
        self.expect(TokenKind::Semi, "after declaration")?;
        Ok(Global {
            name,
            ty,
            init,
            is_const,
            pos: tok.pos,
        })
    }

    // ---- statements --------------------------------------------------------

    fn parse_block(&mut self) -> PResult<Block> {
        self.expect(TokenKind::LBrace, "to start a block")?;
        let saved = self.no_struct;
        self.no_struct = false;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            if self.check(&TokenKind::Eof) {
                return Err(Diagnostic::new(
                    self.pos(),
                    "expected '}' before end of file",
                ));
            }
            stmts.push(self.parse_stmt()?);
        }
        self.no_struct = saved;
        self.expect(TokenKind::RBrace, "to end a block")?;
        Ok(stmts)
    }

    fn parse_header_expr(&mut self) -> PResult<Expr> {
        let saved = self.no_struct;
        self.no_struct = true;
        let r = self.parse_expr();
        self.no_struct = saved;
        r
    }

    fn parse_nested_expr(&mut self) -> PResult<Expr> {
        let saved = self.no_struct;
        self.no_struct = false;
        let r = self.parse_expr();
        self.no_struct = saved;
        r
    }

    fn assign_op(&self) -> Option<Option<BinOp>> {
        Some(match self.peek() {
            TokenKind::Assign => None,
            TokenKind::PlusEq => Some(BinOp::Add),
            TokenKind::MinusEq => Some(BinOp::Sub),
            TokenKind::StarEq => Some(BinOp::Mul),
            TokenKind::SlashEq => Some(BinOp::Div),
            TokenKind::PercentEq => Some(BinOp::Rem),
            _ => return None,
        })
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        let pos = self.pos();
        match self.peek().clone() {
            TokenKind::Let | TokenKind::Const => {
                let is_const = self.advance().kind == TokenKind::Const;
                let (name, _) = self.expect_ident("after 'let'/'const'")?;
                self.expect(TokenKind::Colon, "(a type annotation is required)")?;
                let ty = self.parse_type()?;
                self.expect(TokenKind::Assign, "(variables must be initialized)")?;
                let init = self.parse_expr()?;
                self.expect(TokenKind::Semi, "after declaration")?;
                Ok(Stmt {
                    kind: StmtKind::Let {
                        name,
                        ty,
                        init,
                        is_const,
                    },
                    pos,
                })
            }
            TokenKind::Return => {
                self.advance();
                let value = if self.check(&TokenKind::Semi) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect(TokenKind::Semi, "after 'return'")?;
                Ok(Stmt {
                    kind: StmtKind::Return(value),
                    pos,
                })
            }
            TokenKind::Break => {
                self.advance();
                self.expect(TokenKind::Semi, "after 'break'")?;
                Ok(Stmt {
                    kind: StmtKind::Break,
                    pos,
                })
            }
            TokenKind::Continue => {
                self.advance();
                self.expect(TokenKind::Semi, "after 'continue'")?;
                Ok(Stmt {
                    kind: StmtKind::Continue,
                    pos,
                })
            }
            TokenKind::If => self.parse_if(),
            TokenKind::While => {
                self.advance();
                let cond = self.parse_header_expr()?;
                let body = self.parse_block()?;
                Ok(Stmt {
                    kind: StmtKind::While { cond, body },
                    pos,
                })
            }
            TokenKind::For => {
                self.advance();
                let (var, _) = self.expect_ident("as the loop variable")?;
                self.expect(TokenKind::In, "after the loop variable")?;
                let first = self.parse_header_expr()?;
                if self.eat(&TokenKind::DotDot) {
                    let end = self.parse_header_expr()?;
                    let body = self.parse_block()?;
                    Ok(Stmt {
                        kind: StmtKind::ForRange {
                            var,
                            start: first,
                            end,
                            body,
                        },
                        pos,
                    })
                } else {
                    let body = self.parse_block()?;
                    Ok(Stmt {
                        kind: StmtKind::ForEach {
                            var,
                            iter: first,
                            body,
                        },
                        pos,
                    })
                }
            }
            TokenKind::LBrace => {
                let b = self.parse_block()?;
                Ok(Stmt {
                    kind: StmtKind::Block(b),
                    pos,
                })
            }
            _ => {
                let e = self.parse_expr()?;
                if let Some(op) = self.assign_op() {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.expect(TokenKind::Semi, "after assignment")?;
                    Ok(Stmt {
                        kind: StmtKind::Assign {
                            target: e,
                            op,
                            value,
                        },
                        pos,
                    })
                } else {
                    self.expect(TokenKind::Semi, "after expression")?;
                    Ok(Stmt {
                        kind: StmtKind::Expr(e),
                        pos,
                    })
                }
            }
        }
    }

    fn parse_if(&mut self) -> PResult<Stmt> {
        let pos = self.expect(TokenKind::If, "")?.pos;
        let cond = self.parse_header_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.eat(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                Some(vec![self.parse_if()?])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt {
            kind: StmtKind::If {
                cond,
                then_block,
                else_block,
            },
            pos,
        })
    }

    // ---- expressions -------------------------------------------------------

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_binary(1)
    }

    fn binop_info(&self) -> Option<(BinOp, u8)> {
        Some(match self.peek() {
            TokenKind::OrOr => (BinOp::Or, 1),
            TokenKind::AndAnd => (BinOp::And, 2),
            TokenKind::EqEq => (BinOp::Eq, 3),
            TokenKind::NotEq => (BinOp::Ne, 3),
            TokenKind::Lt => (BinOp::Lt, 4),
            TokenKind::Le => (BinOp::Le, 4),
            TokenKind::Gt => (BinOp::Gt, 4),
            TokenKind::Ge => (BinOp::Ge, 4),
            TokenKind::Plus => (BinOp::Add, 5),
            TokenKind::Minus => (BinOp::Sub, 5),
            TokenKind::Star => (BinOp::Mul, 6),
            TokenKind::Slash => (BinOp::Div, 6),
            TokenKind::Percent => (BinOp::Rem, 6),
            _ => return None,
        })
    }

    fn parse_binary(&mut self, min_prec: u8) -> PResult<Expr> {
        let mut lhs = self.parse_unary()?;
        while let Some((op, prec)) = self.binop_info() {
            if prec < min_prec {
                break;
            }
            let pos = self.pos();
            self.advance();
            let rhs = self.parse_binary(prec + 1)?;
            lhs = Expr::new(ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)), pos);
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        let pos = self.pos();
        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                let e = self.parse_unary()?;
                Ok(Expr::new(ExprKind::Unary(UnaryOp::Neg, Box::new(e)), pos))
            }
            TokenKind::Bang => {
                self.advance();
                let e = self.parse_unary()?;
                Ok(Expr::new(ExprKind::Unary(UnaryOp::Not, Box::new(e)), pos))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            let pos = self.pos();
            if self.eat(&TokenKind::Dot) {
                let (name, _) = self.expect_ident("after '.'")?;
                e = Expr::new(ExprKind::Field(Box::new(e), name), pos);
            } else if self.eat(&TokenKind::LBracket) {
                let idx = self.parse_nested_expr()?;
                self.expect(TokenKind::RBracket, "after index")?;
                e = Expr::new(ExprKind::Index(Box::new(e), Box::new(idx)), pos);
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let pos = self.pos();
        match self.peek().clone() {
            TokenKind::Int(v) => {
                self.advance();
                Ok(Expr::new(ExprKind::Int(v), pos))
            }
            TokenKind::Float(v) => {
                self.advance();
                Ok(Expr::new(ExprKind::Float(v), pos))
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr::new(ExprKind::Str(s), pos))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::new(ExprKind::Bool(true), pos))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::new(ExprKind::Bool(false), pos))
            }
            TokenKind::Ident(name) => {
                self.advance();
                if self.eat(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_nested_expr()?);
                            if !self.eat(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen, "after call arguments")?;
                    Ok(Expr::new(ExprKind::Call(name, args), pos))
                } else if !self.no_struct
                    && self.check(&TokenKind::LBrace)
                    && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                {
                    self.advance(); // '{'
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) {
                        let (fname, _) = self.expect_ident("as field name")?;
                        self.expect(TokenKind::Colon, "after field name")?;
                        let value = self.parse_nested_expr()?;
                        fields.push((fname, value));
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace, "to close the struct literal")?;
                    Ok(Expr::new(ExprKind::StructLit(name, fields), pos))
                } else {
                    Ok(Expr::new(ExprKind::Ident(name), pos))
                }
            }
            TokenKind::LParen => {
                self.advance();
                let e = self.parse_nested_expr()?;
                self.expect(TokenKind::RParen, "to close parenthesis")?;
                Ok(e)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                while !self.check(&TokenKind::RBracket) {
                    items.push(self.parse_nested_expr()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket, "to close the array literal")?;
                Ok(Expr::new(ExprKind::ArrayLit(items), pos))
            }
            other => Err(Diagnostic::new(
                pos,
                format!("expected expression, found {}", other.describe()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> PResult<Program> {
        let toks = Lexer::new(src).tokenize()?;
        Parser::new(toks).parse_program()
    }

    #[test]
    fn parses_main_with_let() {
        let p = parse("fn main() -> i32 { let x: i32 = 1 + 2 * 3; return x; }").unwrap();
        assert_eq!(p.functions.len(), 1);
        assert_eq!(p.functions[0].body.len(), 2);
        assert_eq!(p.functions[0].ret, Type::I32);
    }

    #[test]
    fn rejects_missing_type_annotation() {
        assert!(parse("fn main() { let x = 1; }").is_err());
    }

    #[test]
    fn parses_else_if_chain() {
        let src = "fn f(a: i32) { if a > 1 { } else if a > 0 { } else { } }";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_compound_assignment() {
        let p = parse("fn f() { let x: i32 = 1; x += 2; }").unwrap();
        match &p.functions[0].body[1].kind {
            StmtKind::Assign { op, .. } => assert_eq!(*op, Some(BinOp::Add)),
            _ => panic!("expected assignment"),
        }
    }

    #[test]
    fn parses_items() {
        let src = r#"
            import "util";
            struct Point { x: i32, y: i32, }
            const LIMIT: i32 = 10;
            extern fn sqrt(x: f64) -> f64;
            fn main() -> i32 { return 0; }
        "#;
        let p = parse(src).unwrap();
        assert_eq!(p.imports.len(), 1);
        assert_eq!(p.structs[0].fields.len(), 2);
        assert_eq!(p.globals.len(), 1);
        assert!(p.functions[0].is_extern);
        assert!(!p.functions[1].is_extern);
    }

    #[test]
    fn parses_array_types_and_literals() {
        let src = "fn f() { let a: [[i32]] = [[1, 2], []]; a[0][1] = 3; }";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn struct_literal_vs_block_ambiguity() {
        // `Point {` is a struct literal in expressions, but `if x {` is a block.
        let src = "fn f(x: bool) { let p: Point = Point { x: 1, y: 2 }; if x { } }";
        assert!(parse(src).is_ok());
        let src = "fn f() { if Flag { } }";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_for_loops() {
        let src = "fn f(v: [i32]) { for i in 0..10 { } for x in v { break; } }";
        let p = parse(src).unwrap();
        assert!(matches!(
            p.functions[0].body[0].kind,
            StmtKind::ForRange { .. }
        ));
        assert!(matches!(
            p.functions[0].body[1].kind,
            StmtKind::ForEach { .. }
        ));
    }
}
