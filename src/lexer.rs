//! Lexer: turns Fenneth source text into a flat list of tokens.

use crate::diagnostic::{Diagnostic, Pos};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // keywords
    Let,
    Const,
    Fn,
    Return,
    If,
    Else,
    While,
    True,
    False,
    Struct,
    Extern,
    Import,
    For,
    In,
    Break,
    Continue,
    // literals / names
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    // operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Bang,
    Assign,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    Arrow,
    // punctuation
    Colon,
    Semi,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Dot,
    DotDot,
    Eof,
}

impl TokenKind {
    /// Human-readable description used in parser error messages.
    pub fn describe(&self) -> String {
        use TokenKind as T;
        match self {
            T::Let => "'let'".into(),
            T::Const => "'const'".into(),
            T::Fn => "'fn'".into(),
            T::Return => "'return'".into(),
            T::If => "'if'".into(),
            T::Else => "'else'".into(),
            T::While => "'while'".into(),
            T::True => "'true'".into(),
            T::False => "'false'".into(),
            T::Struct => "'struct'".into(),
            T::Extern => "'extern'".into(),
            T::Import => "'import'".into(),
            T::For => "'for'".into(),
            T::In => "'in'".into(),
            T::Break => "'break'".into(),
            T::Continue => "'continue'".into(),
            T::Ident(s) => format!("identifier '{}'", s),
            T::Int(v) => format!("integer '{}'", v),
            T::Float(v) => format!("float '{}'", v),
            T::Str(_) => "string literal".into(),
            T::Plus => "'+'".into(),
            T::Minus => "'-'".into(),
            T::Star => "'*'".into(),
            T::Slash => "'/'".into(),
            T::Percent => "'%'".into(),
            T::EqEq => "'=='".into(),
            T::NotEq => "'!='".into(),
            T::Lt => "'<'".into(),
            T::Le => "'<='".into(),
            T::Gt => "'>'".into(),
            T::Ge => "'>='".into(),
            T::AndAnd => "'&&'".into(),
            T::OrOr => "'||'".into(),
            T::Bang => "'!'".into(),
            T::Assign => "'='".into(),
            T::PlusEq => "'+='".into(),
            T::MinusEq => "'-='".into(),
            T::StarEq => "'*='".into(),
            T::SlashEq => "'/='".into(),
            T::PercentEq => "'%='".into(),
            T::Arrow => "'->'".into(),
            T::Colon => "':'".into(),
            T::Semi => "';'".into(),
            T::Comma => "','".into(),
            T::LParen => "'('".into(),
            T::RParen => "')'".into(),
            T::LBrace => "'{'".into(),
            T::RBrace => "'}'".into(),
            T::LBracket => "'['".into(),
            T::RBracket => "']'".into(),
            T::Dot => "'.'".into(),
            T::DotDot => "'..'".into(),
            T::Eof => "end of file".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: Pos,
}

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

    fn ident(&mut self) -> TokenKind {
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

    fn number(&mut self, pos: Pos) -> Result<TokenKind, Diagnostic> {
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

    fn string(&mut self, pos: Pos) -> Result<TokenKind, Diagnostic> {
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

    fn symbol(&mut self, pos: Pos) -> Result<TokenKind, Diagnostic> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        Lexer::new(src)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn lexes_let_statement() {
        assert_eq!(
            kinds("let x: i32 = 10;"),
            vec![
                TokenKind::Let,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::Ident("i32".into()),
                TokenKind::Assign,
                TokenKind::Int(10),
                TokenKind::Semi,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_floats_strings_and_comments() {
        let k = kinds("// hi\n2.75 /* x */ \"a\\nb\"");
        assert_eq!(
            k,
            vec![
                TokenKind::Float(2.75),
                TokenKind::Str("a\nb".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn lexes_ranges_and_fields() {
        assert_eq!(
            kinds("a.b 1..3 [x]"),
            vec![
                TokenKind::Ident("a".into()),
                TokenKind::Dot,
                TokenKind::Ident("b".into()),
                TokenKind::Int(1),
                TokenKind::DotDot,
                TokenKind::Int(3),
                TokenKind::LBracket,
                TokenKind::Ident("x".into()),
                TokenKind::RBracket,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn reports_unterminated_string() {
        assert!(Lexer::new("\"abc").tokenize().is_err());
    }
}
