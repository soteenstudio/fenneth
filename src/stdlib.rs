//! The Fenneth standard library.
//!
//! Standard modules are ordinary Fenneth source files (see `stdlib/`) that
//! are embedded in the compiler binary, so they need no files on disk. A
//! program loads one with `import "std/math";`. Every name in a program lives
//! in one shared namespace, so each module prefixes its public functions with
//! its own name (`math_sqrt`, `string_split`, ...) and starts its private
//! helpers with an underscore.
//!
//! Functions that need the C library (square roots, trigonometry, fast string
//! search, ...) are `extern fn fen_*` declarations that are implemented in
//! `runtime.c`.
//!
//! Only the std functions a program actually reaches are kept in the
//! generated C (see [`prune_unused`]).

use std::collections::{HashMap, HashSet};

use crate::ast::*;

/// Import paths that start with this prefix refer to the standard library.
pub const PREFIX: &str = "std/";

/// `(module name, source)` for every standard module.
const MODULES: [(&str, &str); 7] = [
    ("array", include_str!("../stdlib/array.fen")),
    ("char", include_str!("../stdlib/char.fen")),
    ("fs", include_str!("../stdlib/fs.fen")),
    ("http", include_str!("../stdlib/http.fen")),
    ("math", include_str!("../stdlib/math.fen")),
    ("string", include_str!("../stdlib/string.fen")),
    ("zip", include_str!("../stdlib/zip.fen")),
];

/// Modules whose `fen_*` implementations live in their own C file (see
/// [`runtime_unit`]) instead of the always-present `runtime.c`. A program
/// that never reaches one of their functions does not pay for the C.
pub const RUNTIME_UNITS: [&str; 3] = ["fs", "http", "zip"];

/// The C source that implements the `extern fn fen_*` declarations of a
/// standard module: `runtime_unit("fs")` is `Some(("fs", <fs.c>))`. Modules
/// backed by the always-present runtime return `None`.
pub fn runtime_unit(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "fs" => ("fs", include_str!("runtime/fs.c")),
        "http" => ("http", include_str!("runtime/http.c")),
        "zip" => ("zip", include_str!("runtime/zip.c")),
        _ => return None,
    })
}

/// The runtime unit that provides the C symbol `symbol`, if any.
pub fn unit_of_symbol(symbol: &str) -> Option<&'static str> {
    if symbol.starts_with("fen_fs_") {
        Some("fs")
    } else if symbol.starts_with("fen_http_") || symbol.starts_with("fen_url_") {
        Some("http")
    } else if symbol.starts_with("fen_zip_") {
        Some("zip")
    } else {
        None
    }
}

/// Names of all standard modules, sorted.
pub fn module_names() -> Vec<&'static str> {
    MODULES.iter().map(|(n, _)| *n).collect()
}

/// Source text of the standard module `name` (without the `std/` prefix).
pub fn module_source(name: &str) -> Option<&'static str> {
    MODULES.iter().find(|(n, _)| *n == name).map(|(_, s)| *s)
}

/// If `import_path` names a standard module (`std/<name>`), returns `<name>`.
pub fn module_of_import(import_path: &str) -> Option<&str> {
    import_path.strip_prefix(PREFIX)
}

fn calls_in_expr(e: &Expr, out: &mut Vec<String>) {
    match &e.kind {
        ExprKind::Call(name, args) => {
            out.push(name.clone());
            args.iter().for_each(|a| calls_in_expr(a, out));
        }
        ExprKind::Unary(_, a) | ExprKind::Field(a, _) => calls_in_expr(a, out),
        ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => {
            calls_in_expr(a, out);
            calls_in_expr(b, out);
        }
        ExprKind::ArrayLit(items) => items.iter().for_each(|a| calls_in_expr(a, out)),
        ExprKind::StructLit(_, fields) => fields.iter().for_each(|(_, a)| calls_in_expr(a, out)),
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Str(_)
        | ExprKind::Ident(_) => {}
    }
}

fn calls_in_block(b: &Block, out: &mut Vec<String>) {
    for s in b {
        match &s.kind {
            StmtKind::Let { init, .. } => calls_in_expr(init, out),
            StmtKind::Assign { target, value, .. } => {
                calls_in_expr(target, out);
                calls_in_expr(value, out);
            }
            StmtKind::Expr(e) => calls_in_expr(e, out),
            StmtKind::Return(Some(e)) => calls_in_expr(e, out),
            StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                calls_in_expr(cond, out);
                calls_in_block(then_block, out);
                if let Some(eb) = else_block {
                    calls_in_block(eb, out);
                }
            }
            StmtKind::While { cond, body } => {
                calls_in_expr(cond, out);
                calls_in_block(body, out);
            }
            StmtKind::ForRange {
                start, end, body, ..
            } => {
                calls_in_expr(start, out);
                calls_in_expr(end, out);
                calls_in_block(body, out);
            }
            StmtKind::ForEach { iter, body, .. } => {
                calls_in_expr(iter, out);
                calls_in_block(body, out);
            }
            StmtKind::Block(b) => calls_in_block(b, out),
        }
    }
}

/// Removes standard library functions that no user function can reach, so
/// importing a module only costs the code that is really used.
///
/// Must run after type checking: the checker still validates every function
/// of every imported module, used or not.
pub fn prune_unused(program: &mut Program) {
    if !program.functions.iter().any(|f| f.from_std) {
        return;
    }
    let index: HashMap<&str, &Function> = program
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();

    // Everything written by the user is a root, used or not.
    let mut work: Vec<String> = Vec::new();
    for f in program.functions.iter().filter(|f| !f.from_std) {
        calls_in_block(&f.body, &mut work);
    }
    let mut reachable: HashSet<String> = HashSet::new();
    while let Some(name) = work.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }
        if let Some(f) = index.get(name.as_str()) {
            if f.from_std {
                calls_in_block(&f.body, &mut work);
            }
        }
    }
    program
        .functions
        .retain(|f| !f.from_std || reachable.contains(&f.name));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_table_is_sorted_and_complete() {
        let names = module_names();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        assert!(module_source("math").is_some());
        assert!(module_source("nope").is_none());
    }

    #[test]
    fn recognises_std_import_paths() {
        assert_eq!(module_of_import("std/math"), Some("math"));
        assert_eq!(module_of_import("geometry"), None);
        assert_eq!(module_of_import("stdx/math"), None);
    }
}
