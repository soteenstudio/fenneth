//! Recursive-descent parser: tokens -> AST.

mod expr;
mod items;
mod stmt;

#[cfg(test)]
mod tests;

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

pub(super) type PResult<T> = Result<T, Diagnostic>;

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
}
