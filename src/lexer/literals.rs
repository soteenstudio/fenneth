//! Identifiers, numbers and string literals.

use crate::diagnostic::{Diagnostic, Pos};

use super::{Lexer, TokenKind};

impl Lexer {
    pub(super) fn ident(&mut self) -> TokenKind {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        match s.as_str() {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "struct" => TokenKind::Struct,
            "extern" => TokenKind::Extern,
            "import" => TokenKind::Import,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            _ => TokenKind::Ident(s),
        }
    }

    fn digits(&mut self, s: &mut String) {
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
    }

    pub(super) fn number(&mut self, pos: Pos) -> Result<TokenKind, Diagnostic> {
        let mut s = String::new();
        self.digits(&mut s);
        let is_float = self.peek() == Some('.') && self.peek2().is_some_and(|c| c.is_ascii_digit());
        if is_float {
            s.push('.');
            self.bump();
            self.digits(&mut s);
            s.parse::<f64>()
                .map(TokenKind::Float)
                .map_err(|_| Diagnostic::new(pos, format!("invalid float literal '{}'", s)))
        } else {
            s.parse::<i64>()
                .map(TokenKind::Int)
                .map_err(|_| Diagnostic::new(pos, format!("integer literal '{}' is too large", s)))
        }
    }

    pub(super) fn string(&mut self, pos: Pos) -> Result<TokenKind, Diagnostic> {
        self.bump(); // opening quote
        let mut s = String::new();
        loop {
            let here = self.pos();
            match self.bump() {
                None | Some('\n') => {
                    return Err(Diagnostic::new(pos, "unterminated string literal"));
                }
                Some('"') => return Ok(TokenKind::Str(s)),
                Some('\\') => match self.bump() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(other) => {
                        return Err(Diagnostic::new(
                            here,
                            format!("unknown escape sequence '\\{}'", other),
                        ));
                    }
                    None => return Err(Diagnostic::new(pos, "unterminated string literal")),
                },
                Some(c) => s.push(c),
            }
        }
    }
}
