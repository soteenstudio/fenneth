//! Expressions and operator precedence.

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::{PResult, Parser};

impl Parser {
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
