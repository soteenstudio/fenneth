use crate::ast::*;
use crate::lexer::Lexer;

use super::{PResult, Parser};

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
