//! Operators and punctuation.

use crate::diagnostic::{Diagnostic, Pos};

use super::{Lexer, TokenKind};

impl Lexer {
    pub(super) fn symbol(&mut self, pos: Pos) -> Result<TokenKind, Diagnostic> {
        use TokenKind as T;
        let c = self.bump().expect("symbol called at end of input");
        Ok(match c {
            '+' => {
                if self.eat('=') {
                    T::PlusEq
                } else {
                    T::Plus
                }
            }
            '-' => {
                if self.eat('>') {
                    T::Arrow
                } else if self.eat('=') {
                    T::MinusEq
                } else {
                    T::Minus
                }
            }
            '*' => {
                if self.eat('=') {
                    T::StarEq
                } else {
                    T::Star
                }
            }
            '/' => {
                if self.eat('=') {
                    T::SlashEq
                } else {
                    T::Slash
                }
            }
            '%' => {
                if self.eat('=') {
                    T::PercentEq
                } else {
                    T::Percent
                }
            }
            '=' => {
                if self.eat('=') {
                    T::EqEq
                } else {
                    T::Assign
                }
            }
            '!' => {
                if self.eat('=') {
                    T::NotEq
                } else {
                    T::Bang
                }
            }
            '<' => {
                if self.eat('=') {
                    T::Le
                } else {
                    T::Lt
                }
            }
            '>' => {
                if self.eat('=') {
                    T::Ge
                } else {
                    T::Gt
                }
            }
            '&' => {
                if self.eat('&') {
                    T::AndAnd
                } else {
                    return Err(Diagnostic::new(pos, "unexpected '&' (did you mean '&&'?)"));
                }
            }
            '|' => {
                if self.eat('|') {
                    T::OrOr
                } else {
                    return Err(Diagnostic::new(pos, "unexpected '|' (did you mean '||'?)"));
                }
            }
            ':' => T::Colon,
            ';' => T::Semi,
            ',' => T::Comma,
            '(' => T::LParen,
            ')' => T::RParen,
            '{' => T::LBrace,
            '}' => T::RBrace,
            '[' => T::LBracket,
            ']' => T::RBracket,
            '.' => {
                if self.eat('.') {
                    T::DotDot
                } else {
                    T::Dot
                }
            }
            other => {
                return Err(Diagnostic::new(
                    pos,
                    format!("unexpected character '{}'", other),
                ));
            }
        })
    }
}
