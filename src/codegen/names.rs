//! Mapping Fenneth types and names onto C.

use crate::ast::Type;

pub fn mangle(t: &Type) -> String {
    match t {
        Type::I32 => "i32".into(),
        Type::F64 => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Str => "str".into(),
        Type::Void => "void".into(),
        Type::Struct(n) => format!("S_{}", n),
        Type::Array(inner) => format!("arr_{}", mangle(inner)),
    }
}

pub fn vec_name(elem: &Type) -> String {
    format!("Vec_{}", mangle(elem))
}

pub fn c_type(t: &Type) -> String {
    match t {
        Type::I32 => "int32_t".into(),
        Type::F64 => "double".into(),
        Type::Bool => "bool".into(),
        Type::Str => "const char*".into(),
        Type::Void => "void".into(),
        Type::Struct(n) => format!("S_{}", n),
        Type::Array(inner) => format!("{}*", vec_name(inner)),
    }
}

/// A C declaration of `name` with type `t`, optionally constant.
pub fn decl(t: &Type, name: &str, is_const: bool) -> String {
    match (t, is_const) {
        (Type::Str, true) => format!("const char* const {}", name),
        (Type::Array(_), true) => format!("{} const {}", c_type(t), name),
        (_, true) => format!("const {} {}", c_type(t), name),
        (_, false) => format!("{} {}", c_type(t), name),
    }
}

pub fn fn_name(name: &str) -> String {
    format!("fn_{}", name)
}

pub fn var_name(name: &str) -> String {
    format!("v_{}", name)
}

pub fn field_name(name: &str) -> String {
    format!("f_{}", name)
}

/// Escapes a string for use inside a C string literal. Non-ASCII bytes are
/// written as octal escapes so UTF-8 text round-trips exactly.
pub fn c_escape(s: &str) -> String {
    let mut out = String::new();
    let mut prev_octal = false;
    for b in s.bytes() {
        let mut octal = false;
        match b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\r' => out.push_str("\\r"),
            b'?' => out.push_str("\\?"),
            0x20..=0x7e => out.push(b as char),
            _ => {
                out.push_str(&format!("\\{:03o}", b));
                octal = true;
            }
        }
        prev_octal = octal;
    }
    let _ = prev_octal; // octal escapes are always 3 digits, so no ambiguity
    out
}
