use crate::diagnostic::Diagnostic;
use crate::lexer::Lexer;
use crate::parser::Parser;

use super::check;

fn run(src: &str) -> Result<(), Vec<Diagnostic>> {
    let toks = Lexer::new(src).tokenize().unwrap();
    let mut prog = Parser::new(toks).parse_program().unwrap();
    check(&mut prog)
}

fn first_error(src: &str) -> String {
    run(src).unwrap_err()[0].message.clone()
}

#[test]
fn accepts_valid_program() {
    assert!(run("fn main() -> i32 { let x: i32 = 1; print(x); return 0; }").is_ok());
}

#[test]
fn rejects_type_mismatch() {
    let m = first_error("fn main() -> i32 { let x: i32 = \"hi\"; return 0; }");
    assert!(m.contains("cannot initialize"), "{}", m);
}

#[test]
fn rejects_mixed_arithmetic() {
    let m = first_error("fn main() -> i32 { let x: f64 = 1.0 + 2; return 0; }");
    assert!(m.contains("cannot be applied"), "{}", m);
}

#[test]
fn rejects_const_assignment() {
    let m = first_error("fn main() -> i32 { const x: i32 = 1; x = 2; return 0; }");
    assert!(m.contains("constant"), "{}", m);
}

#[test]
fn requires_main() {
    assert!(run("fn other() { }").is_err());
}

#[test]
fn requires_return_on_all_paths() {
    let m = first_error("fn main() -> i32 { if true { return 1; } }");
    assert!(m.contains("every path"), "{}", m);
}

#[test]
fn checks_structs_and_fields() {
    let ok = "struct P { x: i32, y: i32 } \
              fn main() -> i32 { let p: P = P { x: 1, y: 2 }; p.x = 5; return p.x + p.y; }";
    assert!(run(ok).is_ok());
    let m =
        first_error("struct P { x: i32 } fn main() -> i32 { let p: P = P { x: 1 }; return p.z; }");
    assert!(m.contains("no field"), "{}", m);
    let m = first_error("struct P { x: i32 } fn main() -> i32 { let p: P = P { }; return 0; }");
    assert!(m.contains("missing field"), "{}", m);
}

#[test]
fn rejects_recursive_struct_by_value() {
    let m = first_error("struct A { a: A } fn main() -> i32 { return 0; }");
    assert!(m.contains("contains itself"), "{}", m);
    // ...but arrays of itself are fine.
    assert!(run("struct A { kids: [A] } fn main() -> i32 { return 0; }").is_ok());
}

#[test]
fn checks_arrays() {
    let ok = "fn main() -> i32 { let v: [i32] = []; push(v, 3); v[0] = 4; \
              let s: i32 = 0; for x in v { s += x; } return s + len(v); }";
    assert!(run(ok).is_ok());
    let m = first_error("fn main() -> i32 { let v: [i32] = [1, \"a\"]; return 0; }");
    assert!(m.contains("array element"), "{}", m);
}

#[test]
fn extern_signatures_are_ffi_safe() {
    assert!(run("extern fn f(a: i32, s: String) -> f64; fn main() -> i32 { return 0; }").is_ok());
    let m = first_error("extern fn f(a: [i32]) -> i32; fn main() -> i32 { return 0; }");
    assert!(m.contains("extern"), "{}", m);
}

#[test]
fn rejects_empty_array_without_context() {
    let m = first_error("fn main() -> i32 { print(len([])); return 0; }");
    assert!(m.contains("empty array"), "{}", m);
}

#[test]
fn rejects_break_outside_loop_and_assign_to_loop_var() {
    let m = first_error("fn main() -> i32 { break; return 0; }");
    assert!(m.contains("outside of a loop"), "{}", m);
    let m = first_error("fn main() -> i32 { for i in 0..3 { i = 1; } return 0; }");
    assert!(m.contains("constant"), "{}", m);
}

#[test]
fn globals_must_be_literals() {
    assert!(run("const N: i32 = 5; fn main() -> i32 { return N; }").is_ok());
    let m = first_error("const N: i32 = 1 + 2; fn main() -> i32 { return N; }");
    assert!(m.contains("literal"), "{}", m);
}

#[test]
fn allows_most_negative_i32_literal() {
    assert!(run("fn main() -> i32 { let x: i32 = -2147483648; return 0; }").is_ok());
    assert!(run("fn main() -> i32 { let x: i32 = 2147483648; return 0; }").is_err());
}

#[test]
fn string_concat_and_builtins() {
    let ok = "fn main() -> i32 { let s: String = \"a\" + to_str(1); \
              let c: i32 = str_at(s, 0); s += substr(s, 0, 1); return c + len(s); }";
    assert!(run(ok).is_ok());
}
