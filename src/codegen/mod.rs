//! C code generator: type-checked AST -> C99 source text.
//!
//! Mapping:
//!   i32 -> int32_t, f64 -> double, bool -> bool, String -> const char*
//!   struct Foo  -> struct S_Foo (by value)
//!   [T]         -> Vec_<T>*     (growable array, reference semantics)
//! User functions are emitted as `fn_<name>`, variables as `v_<name>` and
//! struct fields as `f_<name>`, so Fenneth names can never clash with C
//! keywords or libc. `extern fn` keeps its plain C symbol name.
//!
//! Layout: [`names`] maps Fenneth types and identifiers to C, [`collect`]
//! walks the AST for the vector types to declare, [`program`] emits the
//! translation unit, and [`stmt`]/[`expr`] translate statements and
//! expressions. [`gen`] holds the shared output buffer.

mod collect;
mod expr;
mod gen;
mod names;
mod program;
mod stmt;

pub use program::generate;

/// Release builds set `FENNETH_VERSION` (format `0.xxx`, e.g. `0.003`) at
/// compile time; local builds fall back to the version in `Cargo.toml`.
pub const VERSION: &str = match option_env!("FENNETH_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// The C runtime text spliced into every generated file.
pub(super) const RUNTIME: &str = include_str!("../runtime.c");
