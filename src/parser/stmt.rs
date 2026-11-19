//! Statements and blocks.

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::{PResult, Parser};

impl Parser {
    pub(super) fn parse_block(&mut self) -> PResult<Block> {
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

    pub(super) fn parse_header_expr(&mut self) -> PResult<Expr> {
        let saved = self.no_struct;
        self.no_struct = true;
        let r = self.parse_expr();
        self.no_struct = saved;
        r
    }

    pub(super) fn parse_nested_expr(&mut self) -> PResult<Expr> {
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
}
