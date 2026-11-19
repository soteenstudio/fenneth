//! Types, functions, structs and globals.

use crate::ast::*;
use crate::lexer::TokenKind;

use super::{PResult, Parser};

impl Parser {
    pub(super) fn parse_type(&mut self) -> PResult<Type> {
        if self.eat(&TokenKind::LBracket) {
            let inner = self.parse_type()?;
            self.expect(TokenKind::RBracket, "to close the array type")?;
            return Ok(Type::Array(Box::new(inner)));
        }
        let (name, _) = self.expect_ident("as a type name")?;
        // Unknown names are reported by the type checker.
        Ok(Type::from_name(&name).unwrap_or(Type::Struct(name)))
    }

    pub(super) fn parse_function(&mut self, is_extern: bool) -> PResult<Function> {
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

    pub(super) fn parse_struct(&mut self) -> PResult<StructDef> {
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

    pub(super) fn parse_global(&mut self) -> PResult<Global> {
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
}
