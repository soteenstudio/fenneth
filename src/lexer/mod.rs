//! Lexer: turns Fenneth source text into a flat list of tokens.

mod literals;
mod symbols;
mod token;

#[cfg(test)]
mod tests;

pub use token::{Token, TokenKind};

use crate::diagnostic::{Diagnostic, Pos};

pub struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: u32,
    col: u32,
    file: u32,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self::with_file(src, 0)
    }

    /// Creates a lexer whose positions carry the given file id.
    pub fn with_file(src: &str, file: u32) -> Self {
        Lexer {
            chars: src.chars().collect(),
            i: 0,
            line: 1,
            col: 1,
            file,
        }
    }

    fn pos(&self) -> Pos {
        Pos::in_file(self.file, self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.i + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.i).copied()?;
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Tokenizes the whole input. The last token is always `Eof`.
    pub fn tokenize(mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia()?;
            let pos = self.pos();
            let c = match self.peek() {
                Some(c) => c,
                None => {
                    out.push(Token {
                        kind: TokenKind::Eof,
                        pos,
                    });
                    break;
                }
            };
            let kind = if c.is_ascii_alphabetic() || c == '_' {
                self.ident()
            } else if c.is_ascii_digit() {
                self.number(pos)?
            } else if c == '"' {
                self.string(pos)?
            } else {
                self.symbol(pos)?
            };
            out.push(Token { kind, pos });
        }
        Ok(out)
    }

    fn skip_trivia(&mut self) -> Result<(), Diagnostic> {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek2() == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('/') if self.peek2() == Some('*') => {
                    let start = self.pos();
                    self.bump();
                    self.bump();
                    let mut closed = false;
                    while let Some(c) = self.bump() {
                        if c == '*' && self.peek() == Some('/') {
                            self.bump();
                            closed = true;
                            break;
                        }
                    }
                    if !closed {
                        return Err(Diagnostic::new(start, "unterminated block comment"));
                    }
                }
                _ => return Ok(()),
            }
        }
    }
}
