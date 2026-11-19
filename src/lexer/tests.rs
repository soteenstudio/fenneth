use super::{Lexer, TokenKind};

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
